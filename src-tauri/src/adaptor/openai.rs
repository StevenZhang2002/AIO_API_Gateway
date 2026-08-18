use std::time::Instant;

use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

use super::{
    apply_model_mapping, build_url, extract_usage, http_client, parse_json, parse_sse_data,
    sse_stream_convert, stream_http_client, truncate, wrap_error, Adaptor, ChannelConfig,
    ProxyRequest, TestResult, TokenUsage, UsageReceiver,
};
use axum::body::Body;

/// OpenAI 官方渠道适配器
pub struct OpenAIAdaptor;

#[async_trait]
impl Adaptor for OpenAIAdaptor {
    fn channel_type(&self) -> &'static str {
        "openai"
    }

    fn default_models(&self) -> Vec<&'static str> {
        vec!["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-3.5-turbo"]
    }

    fn default_base_url(&self) -> &str {
        "https://api.openai.com/v1"
    }

    async fn test(&self, config: &ChannelConfig) -> Result<TestResult, anyhow::Error> {
        test_openai_compatible(config).await
    }

    async fn forward(
        &self,
        request: &ProxyRequest,
        config: &ChannelConfig,
    ) -> Result<(u16, serde_json::Value, Option<TokenUsage>), anyhow::Error> {
        forward_openai_compatible(request, config).await
    }

    async fn forward_stream(
        &self,
        request: &ProxyRequest,
        config: &ChannelConfig,
    ) -> Result<(Body, Option<UsageReceiver>), anyhow::Error> {
        forward_stream_openai_compatible(request, config).await
    }
}

// ==================== OpenAI 兼容通用实现（DeepSeek / 自定义渠道复用） ====================

/// 连通性测试：GET /models 验证 API Key 有效性
pub(crate) async fn test_openai_compatible(
    config: &ChannelConfig,
) -> Result<TestResult, anyhow::Error> {
    let start = Instant::now();
    let url = match build_url(config, "models") {
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
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", config.api_key))
        .send()
        .await;
    let latency_ms = start.elapsed().as_millis() as u64;

    match resp {
        Ok(resp) => {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            if status.is_success() {
                let (model_count, models) = serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|v| {
                        v.get("data")
                            .and_then(|d| d.as_array())
                            .map(|a| {
                                let ids: Vec<String> = a.iter()
                                    .filter_map(|m| m.get("id").and_then(|id| id.as_str().map(String::from)))
                                    .collect();
                                (a.len(), ids)
                            })
                    })
                    .unwrap_or((0, vec![]));
                Ok(TestResult {
                    success: true,
                    message: format!("连接成功，获取到 {} 个模型", model_count),
                    latency_ms,
                    models,
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

/// 非流式转发：OpenAI 格式请求 → 上游 → OpenAI 格式响应
pub(crate) async fn forward_openai_compatible(
    request: &ProxyRequest,
    config: &ChannelConfig,
) -> Result<(u16, serde_json::Value, Option<TokenUsage>), anyhow::Error> {
    let url = build_url(config, "chat/completions")?;
    let mut body = request.body.clone();
    body["stream"] = serde_json::json!(false);
    apply_model_mapping(&mut body, config);

    let resp = http_client()
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {}", config.api_key))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    let (status, resp_body) = parse_json(resp).await;
    if (200..300).contains(&status) {
        let usage = extract_usage(&resp_body);
        Ok((status, resp_body, usage))
    } else {
        Ok((status, wrap_error(status, &resp_body), None))
    }
}

/// 流式转发：OpenAI 格式请求 → 上游 → 透传 SSE + 提取用量
pub(crate) async fn forward_stream_openai_compatible(
    request: &ProxyRequest,
    config: &ChannelConfig,
) -> Result<(Body, Option<UsageReceiver>), anyhow::Error> {
    let url = build_url(config, "chat/completions")?;
    let mut body = request.body.clone();
    body["stream"] = serde_json::json!(true);
    // 请求上游在最后一个 chunk 中返回 usage
    body["stream_options"] = serde_json::json!({ "include_usage": true });
    apply_model_mapping(&mut body, config);

    let resp = stream_http_client()
        .post(url)
        .header(AUTHORIZATION, format!("Bearer {}", config.api_key))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    // 透传 SSE 事件，同时提取 usage
    sse_stream_convert(resp, |event_text| {
        // OpenAI 格式本身就是目标格式，直接透传
        let data = parse_sse_data(event_text)?;
        let mut out = Vec::new();
        out.extend_from_slice(b"data: ");
        out.extend_from_slice(data.as_bytes());
        out.extend_from_slice(b"\n\n");
        Some(out)
    }, Some(extract_openai_stream_usage)).await
}

/// 从 OpenAI 流式 SSE 事件中提取 usage（仅在最后一个 chunk 中出现）
fn extract_openai_stream_usage(event_text: &str) -> Option<TokenUsage> {
    let data_str = parse_sse_data(event_text)?;
    let data_str = data_str.trim();
    if data_str == "[DONE]" {
        return None;
    }
    let data: serde_json::Value = serde_json::from_str(data_str).ok()?;
    let usage = data.get("usage")?;
    let total = usage.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
    if total == 0 {
        return None;
    }
    Some(TokenUsage {
        prompt_tokens: usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        completion_tokens: usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        total_tokens: total,
    })
}
