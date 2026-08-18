pub mod claude;
pub mod custom;
pub mod dashscope;
pub mod deepseek;
pub mod gemini;
pub mod openai;

use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

use async_trait::async_trait;
use axum::body::Body;
use serde::{Deserialize, Serialize};

/// 流式用量接收器——用于在流结束后异步获取 Token 用量
pub type UsageReceiver = tokio::sync::oneshot::Receiver<TokenUsage>;

/// 渠道配置——从数据库 Channel 转换而来
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub base_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    pub model_mapping: serde_json::Value,
    pub extra: serde_json::Value,
}

/// 代理请求——统一的上游请求抽象（body 为 OpenAI 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRequest {
    pub model: String,
    pub body: serde_json::Value,
    pub stream: bool,
}

/// 渠道连通性测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub success: bool,
    pub message: String,
    pub latency_ms: u64,
    #[serde(default)]
    pub models: Vec<String>,
}

/// Token 用量——统一各家的计费格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

/// 统一渠道适配器：外部统一以 OpenAI 格式收发，内部转换各家渠道格式
#[async_trait]
pub trait Adaptor: Send + Sync {
    /// 渠道类型标识
    fn channel_type(&self) -> &'static str;
    /// 默认支持的模型列表
    fn default_models(&self) -> Vec<&'static str>;
    /// 默认 API 地址
    fn default_base_url(&self) -> &str;

    /// 测试渠道连通性
    async fn test(&self, config: &ChannelConfig) -> Result<TestResult, anyhow::Error>;

    /// 非流式转发：返回 (状态码, OpenAI 格式响应体, Token用量)
    async fn forward(
        &self,
        request: &ProxyRequest,
        config: &ChannelConfig,
    ) -> Result<(u16, serde_json::Value, Option<TokenUsage>), anyhow::Error>;

    /// 流式转发：返回已转换为 OpenAI 格式的 SSE Body + 可选的流式用量接收器
    async fn forward_stream(
        &self,
        request: &ProxyRequest,
        config: &ChannelConfig,
    ) -> Result<(Body, Option<UsageReceiver>), anyhow::Error>;
}

/// 适配器注册表——按渠道类型分发到具体实现
pub struct AdaptorRegistry {
    adaptors: HashMap<&'static str, Box<dyn Adaptor>>,
    order: Vec<&'static str>,
}

impl AdaptorRegistry {
    /// 注册全部内置渠道适配器
    pub fn new() -> Self {
        let mut registry = Self {
            adaptors: HashMap::new(),
            order: Vec::new(),
        };
        for a in [
            Box::new(openai::OpenAIAdaptor) as Box<dyn Adaptor>,
            Box::new(claude::ClaudeAdaptor),
            Box::new(gemini::GeminiAdaptor),
            Box::new(deepseek::DeepSeekAdaptor),
            Box::new(dashscope::DashscopeAdaptor),
            Box::new(custom::CustomAdaptor),
        ] {
            registry.order.push(a.channel_type());
            registry.adaptors.insert(a.channel_type(), a);
        }
        registry
    }

    /// 按渠道类型获取适配器
    pub fn get(&self, channel_type: &str) -> Option<&dyn Adaptor> {
        self.adaptors.get(channel_type).map(|b| b.as_ref())
    }

    /// 获取全部适配器（用于前端展示各渠道类型的默认配置）
    pub fn all(&self) -> Vec<&dyn Adaptor> {
        self.order.iter().filter_map(|t| self.get(t)).collect()
    }

    /// 各渠道类型的默认配置（按注册顺序返回，供创建渠道表单自动填充）
    pub fn defaults(&self) -> Vec<crate::dto::channel_dto::ChannelDefaultsDto> {
        self.all()
            .into_iter()
            .map(|a| crate::dto::channel_dto::ChannelDefaultsDto {
                channel_type: a.channel_type().to_string(),
                base_url: a.default_base_url().to_string(),
                models: a.default_models().into_iter().map(String::from).collect(),
            })
            .collect()
    }
}

impl Default for AdaptorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== 共享工具 ====================

/// 构建上游请求 URL（base_url + 路径，兼容尾斜杠）
pub(crate) fn build_url(config: &ChannelConfig, path: &str) -> Result<reqwest::Url, anyhow::Error> {
    let base = config.base_url.trim_end_matches('/');
    Ok(reqwest::Url::parse(&format!("{}/{}", base, path))?)
}

/// 非流式请求客户端（60s 总超时）
pub(crate) fn http_client() -> reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("初始化 HTTP 客户端失败")
        })
        .clone()
}

/// 流式请求客户端（仅连接超时，避免切断长连接 SSE）
pub(crate) fn stream_http_client() -> reqwest::Client {
    static STREAM_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    STREAM_CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .build()
                .expect("初始化流式 HTTP 客户端失败")
        })
        .clone()
}

/// 读取响应体并解析为 JSON（非 JSON 内容兜底为 raw 字段）
pub(crate) async fn parse_json(resp: reqwest::Response) -> (u16, serde_json::Value) {
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    let body = serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({ "raw": text }));
    (status, body)
}

/// 将上游错误响应包装为 OpenAI 格式的 error 响应
pub(crate) fn wrap_error(status: u16, body: &serde_json::Value) -> serde_json::Value {
    let message = body
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| body.to_string());
    serde_json::json!({
        "error": {
            "message": message,
            "type": "upstream_error",
            "code": status,
        }
    })
}

/// 根据 model_mapping 将外部模型名映射为渠道内部模型名
pub(crate) fn apply_model_mapping(body: &mut serde_json::Value, config: &ChannelConfig) {
    if let (Some(model), Some(mapping)) = (
        body.get("model").and_then(|m| m.as_str()),
        config.model_mapping.as_object(),
    ) {
        if let Some(mapped) = mapping.get(model).and_then(|v| v.as_str()) {
            body["model"] = serde_json::json!(mapped);
        }
    }
}

/// 从 OpenAI 兼容响应中提取 Token 用量
pub(crate) fn extract_usage(body: &serde_json::Value) -> Option<TokenUsage> {
    let usage = body.get("usage")?;
    Some(TokenUsage {
        prompt_tokens: usage
            .get("prompt_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        completion_tokens: usage
            .get("completion_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        total_tokens: usage
            .get("total_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
    })
}

/// 收集消息中的文本内容（system 消息 / text 块），用于各家格式的 system 字段
pub(crate) fn collect_text(msg: &serde_json::Value, out: &mut String) {
    match msg.get("content") {
        Some(serde_json::Value::String(s)) => out.push_str(s),
        Some(serde_json::Value::Array(parts)) => {
            for part in parts {
                if part.get("type").and_then(|t| t.as_str()) == Some("text") {
                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                        out.push_str(text);
                    }
                }
            }
        }
        _ => {}
    }
}

/// 截断长文本（用于错误消息展示，按字符截断避免 UTF-8 边界问题）
pub(crate) fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        format!("{}...", text.chars().take(max).collect::<String>())
    }
}

// ==================== SSE 流转换工具 ====================

/// 从 reqwest::Response 读取字节流，通过 channel 发送转换后的 SSE 数据
///
/// - `on_event` 回调将原始 SSE 事件转换为 OpenAI 格式字节
/// - `usage_extractor` 回调从原始 SSE 事件中提取 Token 用量，流结束后通过 oneshot 传递
pub(crate) async fn sse_stream_convert<F, U>(
    resp: reqwest::Response,
    on_event: F,
    usage_extractor: Option<U>,
) -> Result<(Body, Option<UsageReceiver>), anyhow::Error>
where
    F: Fn(&str) -> Option<Vec<u8>> + Send + 'static,
    U: Fn(&str) -> Option<TokenUsage> + Send + 'static,
{
    use tokio_stream::wrappers::ReceiverStream;
    use tokio_stream::StreamExt;

    let mut byte_stream = resp.bytes_stream();
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<axum::body::Bytes, std::convert::Infallible>>(64);

    // 流式用量捕获：通过 oneshot 在流结束后传递
    let (usage_tx, usage_rx) = tokio::sync::oneshot::channel::<TokenUsage>();
    let usage_shared = std::sync::Arc::new(tokio::sync::Mutex::new(None::<TokenUsage>));

    let usage_shared_clone = usage_shared.clone();
    tokio::spawn(async move {
        let mut buf = Vec::new();

        while let Some(chunk_result) = byte_stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    buf.extend_from_slice(&chunk);

                    // 按 \n\n 分割完整 SSE 事件
                    while let Some(pos) = find_double_newline(&buf) {
                        let event_bytes = buf.drain(..pos + 2).collect::<Vec<u8>>();
                        let event_text = String::from_utf8_lossy(&event_bytes);
                        let event_text = event_text.trim();

                        // 提取流式用量
                        if let Some(ref extractor) = usage_extractor {
                            if let Some(usage) = extractor(event_text) {
                                let mut guard = usage_shared_clone.lock().await;
                                *guard = Some(usage);
                            }
                        }

                        if let Some(out_bytes) = on_event(event_text) {
                            if tx.send(Ok(axum::body::Bytes::from(out_bytes))).await.is_err() {
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Ok(axum::body::Bytes::from(format!("\n\ndata: {}", e)))).await;
                    return;
                }
            }
        }

        // 处理缓冲区剩余数据
        if !buf.is_empty() {
            let event_text = String::from_utf8_lossy(&buf).trim().to_string();
            if !event_text.is_empty() {
                if let Some(ref extractor) = usage_extractor {
                    if let Some(usage) = extractor(&event_text) {
                        let mut guard = usage_shared_clone.lock().await;
                        *guard = Some(usage);
                    }
                }
                if let Some(out_bytes) = on_event(&event_text) {
                    let _ = tx.send(Ok(axum::body::Bytes::from(out_bytes))).await;
                }
            }
        }

        // 流结束，发送用量（如果有的话）
        let guard = usage_shared_clone.lock().await;
        if let Some(usage) = guard.as_ref() {
            let _ = usage_tx.send(usage.clone());
        }
    });

    Ok((Body::from_stream(ReceiverStream::new(rx)), Some(usage_rx)))
}

/// 在缓冲区中查找 \n\n 或 \r\n\r\n 的位置
///
/// 返回第一个换行符的索引（即 `\n\n` 中第一个 `\n` 的位置），
/// 调用方通过 `drain(..pos + 2)` 即可消费整个事件（含分隔符）。
fn find_double_newline(buf: &[u8]) -> Option<usize> {
    for i in 1..buf.len() {
        if buf[i] == b'\n' && buf[i - 1] == b'\n' {
            return Some(i - 1);
        }
        if buf[i] == b'\n'
            && i >= 3
            && buf[i - 1] == b'\r'
            && buf[i - 2] == b'\n'
            && buf[i - 3] == b'\r'
        {
            return Some(i - 1);
        }
    }
    None
}

/// 从 SSE 事件文本中提取 data 字段内容
pub(crate) fn parse_sse_data(event_text: &str) -> Option<String> {
    let mut data_lines = Vec::new();
    for line in event_text.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(data) = line.strip_prefix("data:") {
            let data = if data.starts_with(' ') {
                &data[1..]
            } else {
                data
            };
            data_lines.push(data);
        }
    }
    if data_lines.is_empty() {
        None
    } else {
        Some(data_lines.join("\n"))
    }
}

/// 从 SSE 事件文本中提取 event 类型
pub(crate) fn parse_sse_event_type(event_text: &str) -> Option<String> {
    for line in event_text.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(event) = line.strip_prefix("event:") {
            let event = if event.starts_with(' ') {
                event[1..].to_string()
            } else {
                event.to_string()
            };
            return Some(event);
        }
    }
    None
}

/// 构建 OpenAI 格式的 SSE chunk
pub(crate) fn openai_sse_chunk(chunk: &serde_json::Value) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"data: ");
    out.extend_from_slice(&serde_json::to_vec(chunk).unwrap_or_default());
    out.extend_from_slice(b"\n\n");
    out
}

/// OpenAI [DONE] 标记
pub(crate) fn openai_sse_done() -> Vec<u8> {
    b"data: [DONE]\n\n".to_vec()
}
