use std::time::Instant;

use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::header::CONTENT_TYPE;

use super::{
    apply_model_mapping, collect_text, extract_usage, http_client, openai_sse_chunk,
    openai_sse_done, parse_json, parse_sse_data, sse_stream_convert, stream_http_client,
    truncate, wrap_error, Adaptor, ChannelConfig, ProxyRequest, TestResult,
    TokenUsage, UsageReceiver,
};
use axum::body::Body;

/// Google Gemini 渠道适配器
pub struct GeminiAdaptor;

#[async_trait]
impl Adaptor for GeminiAdaptor {
    fn channel_type(&self) -> &'static str {
        "gemini"
    }

    fn default_models(&self) -> Vec<&'static str> {
        vec![
            "gemini-2.5-pro",
            "gemini-2.5-flash",
            "gemini-2.5-flash-lite",
            "gemini-2.0-flash",
        ]
    }

    fn default_base_url(&self) -> &str {
        "https://generativelanguage.googleapis.com/v1beta"
    }

    async fn test(&self, config: &ChannelConfig) -> Result<TestResult, anyhow::Error> {
        let start = Instant::now();
        let model = config
            .models
            .first()
            .cloned()
            .unwrap_or_else(|| "gemini-2.5-flash".to_string());
        let url = match gemini_url(config, &model, false) {
            Ok(u) => u,
            Err(e) => {
                return Ok(TestResult {
                    success: false,
                    message: format!("URL 无效: {}", e),
                    latency_ms: 0,
                    models: vec![],
                })
            }
        };

        let resp = http_client()
            .post(url)
            .json(&serde_json::json!({
                "contents": [{ "role": "user", "parts": [{ "text": "ping" }] }],
                "generationConfig": { "maxOutputTokens": 1 },
            }))
            .send()
            .await;
        let latency_ms = start.elapsed().as_millis() as u64;

        match resp {
            Ok(resp) => {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                if status.is_success() {
                    Ok(TestResult {
                        success: true,
                        message: format!("连接成功（HTTP {}）", status),
                        latency_ms,
                        models: vec![],
                    })
                } else {
                    Ok(TestResult {
                        success: false,
                        message: format!(
                            "请求失败: HTTP {}: {}",
                            status,
                            truncate(&text, 200)
                        ),
                        latency_ms,
                        models: vec![],
                    })
                }
            }
            Err(e) => Ok(TestResult {
                success: false,
                message: format!("网络错误: {}", e),
                latency_ms,
                models: vec![],
            }),
        }
    }

    async fn forward(
        &self,
        request: &ProxyRequest,
        config: &ChannelConfig,
    ) -> Result<(u16, serde_json::Value, Option<TokenUsage>), anyhow::Error> {
        let (body, model) = openai_to_gemini(request, config)?;
        let url = gemini_url(config, &model, false)?;

        let resp = http_client()
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let (status, resp_body) = parse_json(resp).await;
        if (200..300).contains(&status) {
            let openai_body = gemini_to_openai(&resp_body);
            let usage = extract_usage(&openai_body);
            Ok((status, openai_body, usage))
        } else {
            Ok((status, wrap_error(status, &resp_body), None))
        }
    }

    async fn forward_stream(
        &self,
        request: &ProxyRequest,
        config: &ChannelConfig,
    ) -> Result<(Body, Option<UsageReceiver>), anyhow::Error> {
        let (body, model) = openai_to_gemini(request, config)?;
        let url = gemini_url(config, &model, true)?;

        let resp = stream_http_client()
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        sse_stream_convert(resp, gemini_sse_to_openai, Some(extract_gemini_stream_usage)).await
    }
}

// ==================== OpenAI ↔ Gemini 格式转换 ====================

/// 构建 Gemini 生成请求 URL（API Key 放 query，流式追加 alt=sse）
fn gemini_url(
    config: &ChannelConfig,
    model: &str,
    stream: bool,
) -> Result<reqwest::Url, anyhow::Error> {
    let method = if stream {
        "streamGenerateContent"
    } else {
        "generateContent"
    };
    let mut url = reqwest::Url::parse(&format!(
        "{}/models/{}:{}",
        config.base_url.trim_end_matches('/'),
        model,
        method
    ))?;
    url.query_pairs_mut().append_pair("key", &config.api_key);
    if stream {
        url.query_pairs_mut().append_pair("alt", "sse");
    }
    Ok(url)
}

/// 将 OpenAI 格式请求转换为 Gemini 格式，返回 (请求体, model 名)
fn openai_to_gemini(
    request: &ProxyRequest,
    config: &ChannelConfig,
) -> Result<(serde_json::Value, String), anyhow::Error> {
    let mut body = request.body.clone();
    apply_model_mapping(&mut body, config);
    let model = body
        .get("model")
        .and_then(|m| m.as_str())
        .ok_or_else(|| anyhow!("缺少 model 参数"))?
        .to_string();

    // system 消息拆出为 systemInstruction；user/assistant 映射为 user/model
    let mut system_text = String::new();
    let mut contents: Vec<serde_json::Value> = Vec::new();

    if let Some(arr) = body.get("messages").and_then(|m| m.as_array()) {
        for msg in arr {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            if role == "system" {
                collect_text(msg, &mut system_text);
            } else {
                let gemini_role = if role == "assistant" { "model" } else { "user" };
                let parts = convert_parts(msg.get("content"))?;
                contents.push(serde_json::json!({ "role": gemini_role, "parts": parts }));
            }
        }
    }

    let mut gemini_body = serde_json::json!({ "contents": contents });
    if !system_text.is_empty() {
        gemini_body["systemInstruction"] = serde_json::json!({ "parts": [{ "text": system_text }] });
    }

    // generationConfig 映射
    let mut gen_config = serde_json::Map::new();
    if let Some(t) = body.get("temperature") {
        gen_config.insert("temperature".to_string(), t.clone());
    }
    if let Some(t) = body.get("top_p") {
        gen_config.insert("topP".to_string(), t.clone());
    }
    if let Some(m) = body.get("max_tokens") {
        gen_config.insert("maxOutputTokens".to_string(), m.clone());
    }
    if let Some(s) = body.get("stop") {
        gen_config.insert("stopSequences".to_string(), s.clone());
    }
    if !gen_config.is_empty() {
        gemini_body["generationConfig"] = serde_json::Value::Object(gen_config);
    }

    Ok((gemini_body, model))
}

/// 转换消息 content 为 Gemini parts（图片需 base64，image_url 暂不支持，忽略）
fn convert_parts(content: Option<&serde_json::Value>) -> Result<serde_json::Value, anyhow::Error> {
    match content {
        Some(serde_json::Value::String(s)) => Ok(serde_json::json!([{ "text": s }])),
        Some(serde_json::Value::Array(parts)) => {
            let mut out = Vec::new();
            for part in parts {
                if part.get("type").and_then(|t| t.as_str()) == Some("text") {
                    out.push(part.clone());
                }
            }
            Ok(serde_json::Value::Array(out))
        }
        _ => Ok(serde_json::json!([])),
    }
}

/// 将 Gemini 响应转换为 OpenAI 格式
fn gemini_to_openai(body: &serde_json::Value) -> serde_json::Value {
    let candidate = body
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first());

    let content = candidate
        .and_then(|c| c.pointer("/content/parts"))
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    let finish_reason = match candidate
        .and_then(|c| c.get("finishReason"))
        .and_then(|f| f.as_str())
    {
        Some("MAX_TOKENS") => "length",
        Some("SAFETY") | Some("RECITATION") | Some("BLOCKLIST") => "content_filter",
        _ => "stop",
    };

    let (prompt_tokens, completion_tokens, total_tokens) = body
        .get("usageMetadata")
        .map(|u| {
            (
                u.get("promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0),
                u.get("candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0),
                u.get("totalTokenCount").and_then(|v| v.as_u64()).unwrap_or(0),
            )
        })
        .unwrap_or((0, 0, 0));

    serde_json::json!({
        "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": serde_json::Value::Null,
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": content },
            "finish_reason": finish_reason,
        }],
        "usage": {
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": total_tokens,
        },
    })
}

// ==================== Gemini SSE → OpenAI SSE 转换 ====================

/// 将单个 Gemini SSE 事件转换为 OpenAI 格式的 SSE 字节
///
/// Gemini 流式格式（alt=sse）：
/// - 每个 data 行是一个 JSON 对象，包含 candidates 和 usageMetadata
/// - 流结束时发送 data: {}
fn gemini_sse_to_openai(event_text: &str) -> Option<Vec<u8>> {
    let data_str = parse_sse_data(event_text)?;
    let data_str = data_str.trim();

    // 空对象表示流结束
    if data_str == "{}" || data_str.is_empty() {
        return Some(openai_sse_done());
    }

    let data: serde_json::Value = serde_json::from_str(data_str).ok()?;

    let candidate = data
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first());

    // 提取文本内容
    let text = candidate
        .and_then(|c| c.pointer("/content/parts"))
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    // 提取 finishReason
    let finish_reason = match candidate
        .and_then(|c| c.get("finishReason"))
        .and_then(|f| f.as_str())
    {
        Some("MAX_TOKENS") => "length",
        Some("SAFETY") | Some("RECITATION") | Some("BLOCKLIST") => "content_filter",
        Some("STOP") | Some("FINISH_REASON_UNSPECIFIED") => "stop",
        _ if !text.is_empty() => "stop",
        _ => "stop",
    };

    // 提取 usageMetadata
    let usage = data.get("usageMetadata").map(|u| {
        serde_json::json!({
            "prompt_tokens": u.get("promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0),
            "completion_tokens": u.get("candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0),
            "total_tokens": u.get("totalTokenCount").and_then(|v| v.as_u64()).unwrap_or(0),
        })
    });

    let chunk = serde_json::json!({
        "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        "object": "chat.completion.chunk",
        "created": chrono::Utc::now().timestamp(),
        "model": "",
        "choices": [{
            "index": 0,
            "delta": { "content": text },
            "finish_reason": finish_reason,
        }],
        "usage": usage,
    });

    Some(openai_sse_chunk(&chunk))
}

// ==================== Gemini 流式用量提取 ====================

/// 从 Gemini 原始 SSE 事件中提取 Token 用量
///
/// Gemini 在每个 SSE 事件的 `usageMetadata` 字段中报告用量，
/// 最后一个非空事件包含最终用量。由于 extractor 会覆盖之前的值，
/// 最终保留的就是流结束时的正确用量。
fn extract_gemini_stream_usage(event_text: &str) -> Option<TokenUsage> {
    let data_str = parse_sse_data(event_text)?;
    let data_str = data_str.trim();
    if data_str == "{}" || data_str.is_empty() {
        return None;
    }
    let data: serde_json::Value = serde_json::from_str(data_str).ok()?;
    let usage = data.get("usageMetadata")?;

    let prompt = usage.get("promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0);
    let completion = usage.get("candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0);
    let total = usage.get("totalTokenCount").and_then(|v| v.as_u64()).unwrap_or(0);

    if prompt == 0 && completion == 0 && total == 0 {
        return None;
    }

    Some(TokenUsage {
        prompt_tokens: prompt,
        completion_tokens: completion,
        total_tokens: total,
    })
}
