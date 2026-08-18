use std::time::Instant;

use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::header::CONTENT_TYPE;

use super::{
    apply_model_mapping, build_url, collect_text, extract_usage, http_client, openai_sse_chunk,
    openai_sse_done, parse_json, parse_sse_data, parse_sse_event_type, sse_stream_convert,
    stream_http_client, truncate, wrap_error, Adaptor, ChannelConfig, ProxyRequest, TestResult,
    TokenUsage, UsageReceiver,
};
use axum::body::Body;

/// Anthropic Claude 渠道适配器
pub struct ClaudeAdaptor;

#[async_trait]
impl Adaptor for ClaudeAdaptor {
    fn channel_type(&self) -> &'static str {
        "claude"
    }

    fn default_models(&self) -> Vec<&'static str> {
        vec![
            "claude-sonnet-4-5",
            "claude-opus-4-1",
            "claude-3-5-sonnet-latest",
            "claude-3-5-haiku-latest",
        ]
    }

    fn default_base_url(&self) -> &str {
        "https://api.anthropic.com/v1"
    }

    async fn test(&self, config: &ChannelConfig) -> Result<TestResult, anyhow::Error> {
        let start = Instant::now();
        let url = match build_url(config, "messages") {
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
        let model = config
            .models
            .first()
            .cloned()
            .unwrap_or_else(|| "claude-3-5-sonnet-latest".to_string());

        let resp = http_client()
            .post(url)
            .header("x-api-key", config.api_key.as_str())
            .header("anthropic-version", "2023-06-01")
            .json(&serde_json::json!({
                "model": model,
                "max_tokens": 1,
                "messages": [{ "role": "user", "content": "ping" }],
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
        let url = build_url(config, "messages")?;
        let body = openai_to_claude(request, config)?;

        let resp = http_client()
            .post(url)
            .header("x-api-key", config.api_key.as_str())
            .header("anthropic-version", "2023-06-01")
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let (status, resp_body) = parse_json(resp).await;
        if (200..300).contains(&status) {
            let openai_body = claude_to_openai(&resp_body);
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
        let url = build_url(config, "messages")?;
        let mut body = openai_to_claude(request, config)?;
        body["stream"] = serde_json::json!(true);

        let resp = stream_http_client()
            .post(url)
            .header("x-api-key", config.api_key.as_str())
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "messages-2023-12-15")
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        sse_stream_convert(resp, claude_sse_to_openai, Some(extract_claude_stream_usage)).await
    }
}



// ==================== OpenAI ↔ Claude 格式转换 ====================

/// 将 OpenAI 格式请求转换为 Claude 格式（/v1/messages）
fn openai_to_claude(
    request: &ProxyRequest,
    config: &ChannelConfig,
) -> Result<serde_json::Value, anyhow::Error> {
    let mut body = request.body.clone();
    apply_model_mapping(&mut body, config);

    // system 消息需拆出为顶层 system 字段；max_tokens 为必填，缺省给 4096
    let mut system_text = String::new();
    let mut messages: Vec<serde_json::Value> = Vec::new();

    if let Some(arr) = body.get("messages").and_then(|m| m.as_array()) {
        for msg in arr {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            if role == "system" {
                collect_text(msg, &mut system_text);
            } else {
                let claude_role = if role == "assistant" { "assistant" } else { "user" };
                let content = convert_content(msg.get("content"))?;
                messages.push(serde_json::json!({ "role": claude_role, "content": content }));
            }
        }
    }

    let model = body
        .get("model")
        .cloned()
        .unwrap_or_else(|| serde_json::json!(config.models.first().cloned().unwrap_or_default()));

    let mut claude_body = serde_json::json!({
        "model": model,
        "max_tokens": body.get("max_tokens").cloned().unwrap_or(serde_json::json!(4096)),
        "messages": messages,
        "stream": request.stream,
    });
    if !system_text.is_empty() {
        claude_body["system"] = serde_json::json!(system_text);
    }
    for key in ["temperature", "top_p", "stop"] {
        if let Some(v) = body.get(key) {
            claude_body[key] = v.clone();
        }
    }
    Ok(claude_body)
}

/// 转换消息 content：字符串 / 文本块 / 图片 URL → Claude 格式
fn convert_content(content: Option<&serde_json::Value>) -> Result<serde_json::Value, anyhow::Error> {
    match content {
        Some(serde_json::Value::String(s)) => Ok(serde_json::Value::String(s.clone())),
        Some(serde_json::Value::Array(parts)) => {
            let mut out = Vec::new();
            for part in parts {
                match part.get("type").and_then(|t| t.as_str()) {
                    Some("text") => out.push(part.clone()),
                    Some("image_url") => {
                        let url = part
                            .pointer("/image_url/url")
                            .and_then(|u| u.as_str())
                            .ok_or_else(|| anyhow!("image_url 缺少 url 字段"))?;
                        out.push(serde_json::json!({
                            "type": "image",
                            "source": { "type": "url", "url": url },
                        }));
                    }
                    _ => { /* 忽略不支持的类型 */ }
                }
            }
            Ok(serde_json::Value::Array(out))
        }
        _ => Ok(serde_json::Value::Null),
    }
}

/// 将 Claude 响应转换为 OpenAI 格式
fn claude_to_openai(body: &serde_json::Value) -> serde_json::Value {
    let content = body
        .get("content")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();

    let finish_reason = match body.get("stop_reason").and_then(|s| s.as_str()) {
        Some("max_tokens") => "length",
        Some("tool_use") => "tool_calls",
        _ => "stop",
    };

    let (prompt_tokens, completion_tokens) = body
        .get("usage")
        .map(|u| {
            (
                u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            )
        })
        .unwrap_or((0, 0));

    serde_json::json!({
        "id": body.get("id").cloned().unwrap_or_else(|| serde_json::json!("msg_unknown")),
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": body.get("model").cloned().unwrap_or(serde_json::Value::Null),
        "choices": [{
            "index": 0,
            "message": { "role": "assistant", "content": content },
            "finish_reason": finish_reason,
        }],
        "usage": {
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": prompt_tokens + completion_tokens,
        },
    })
}

// ==================== Claude SSE → OpenAI SSE 转换 ====================

/// 将单个 Claude SSE 事件转换为 OpenAI 格式的 SSE 字节
///
/// Claude 流式事件类型：
/// - message_start → 空 chunk（含 usage）
/// - content_block_delta → 文本 delta
/// - message_delta → finish_reason + 最终 usage
/// - message_stop → [DONE]
/// - ping / content_block_start / content_block_stop → 跳过
fn claude_sse_to_openai(event_text: &str) -> Option<Vec<u8>> {
    let data_str = parse_sse_data(event_text)?;

    // Claude 有时发送注释事件（以 : 开头），跳过
    if data_str.starts_with(':') {
        return None;
    }

    let event_type = parse_sse_event_type(event_text);
    let data: serde_json::Value = serde_json::from_str(&data_str).ok()?;

    match event_type.as_deref() {
        Some("message_start") => {
            // 发送一个空 chunk，携带 usage 信息
            let usage = data.get("message").and_then(|m| m.get("usage")).map(|u| {
                serde_json::json!({
                    "prompt_tokens": u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                    "completion_tokens": 0,
                    "total_tokens": u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                })
            });

            let chunk = serde_json::json!({
                "id": data.pointer("/message/id").cloned().unwrap_or_else(|| serde_json::json!("")),
                "object": "chat.completion.chunk",
                "created": chrono::Utc::now().timestamp(),
                "model": data.pointer("/message/model").cloned().unwrap_or(serde_json::Value::Null),
                "choices": [{
                    "index": 0,
                    "delta": { "role": "assistant", "content": "" },
                    "finish_reason": null,
                }],
                "usage": usage,
            });
            Some(openai_sse_chunk(&chunk))
        }

        Some("content_block_delta") => {
            // 提取文本 delta
            let text = data
                .get("delta")
                .and_then(|d| d.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("");

            if text.is_empty() {
                return None;
            }

            let chunk = serde_json::json!({
                "id": "",
                "object": "chat.completion.chunk",
                "created": chrono::Utc::now().timestamp(),
                "model": "",
                "choices": [{
                    "index": 0,
                    "delta": { "content": text },
                    "finish_reason": null,
                }],
            });
            Some(openai_sse_chunk(&chunk))
        }

        Some("message_delta") => {
            // 提取 stop_reason 和 usage
            let finish_reason = match data.get("delta").and_then(|d| d.get("stop_reason")).and_then(|s| s.as_str()) {
                Some("max_tokens") => "length",
                Some("end_turn") => "stop",
                Some("tool_use") => "tool_calls",
                _ => "stop",
            };

            let usage = data.get("usage").map(|u| {
                serde_json::json!({
                    "prompt_tokens": 0,
                    "completion_tokens": u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                    "total_tokens": u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                })
            });

            let chunk = serde_json::json!({
                "id": "",
                "object": "chat.completion.chunk",
                "created": chrono::Utc::now().timestamp(),
                "model": "",
                "choices": [{
                    "index": 0,
                    "delta": {},
                    "finish_reason": finish_reason,
                }],
                "usage": usage,
            });
            Some(openai_sse_chunk(&chunk))
        }

        Some("message_stop") => Some(openai_sse_done()),

        // ping, content_block_start, content_block_stop 等事件跳过
        _ => None,
    }
}

// ==================== Claude 流式用量提取 ====================

/// 从 Claude 原始 SSE 事件中提取 Token 用量
///
/// Claude 在 `message_delta` 事件中报告最终 output_tokens，
/// 在 `message_start` 事件中报告 input_tokens。
/// 优先采用 message_delta（包含最终 completion 用量）。
fn extract_claude_stream_usage(event_text: &str) -> Option<TokenUsage> {
    let event_type = parse_sse_event_type(event_text)?;
    let data_str = parse_sse_data(event_text)?;
    let data: serde_json::Value = serde_json::from_str(&data_str).ok()?;

    match event_type.as_str() {
        "message_delta" => {
            // 最终用量：包含 output_tokens
            let usage = data.get("usage")?;
            let completion = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            Some(TokenUsage {
                prompt_tokens: 0,
                completion_tokens: completion,
                total_tokens: completion,
            })
        }
        "message_start" => {
            // 初始用量：包含 input_tokens
            let usage = data.get("message")?.get("usage")?;
            let prompt = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            Some(TokenUsage {
                prompt_tokens: prompt,
                completion_tokens: 0,
                total_tokens: prompt,
            })
        }
        _ => None,
    }
}
