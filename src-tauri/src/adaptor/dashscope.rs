use async_trait::async_trait;
use axum::body::Body;

use super::{openai, Adaptor, ChannelConfig, ProxyRequest, TestResult, UsageReceiver};

/// 阿里云百炼（Dashscope）适配器
///
/// Dashscope 完全兼容 OpenAI 格式，因此直接复用 openai 模块的共享函数。
/// 区别仅在于：
/// - 默认 base_url 不同
/// - 默认模型列表不同（Qwen 系列）
pub struct DashscopeAdaptor;

#[async_trait]
impl Adaptor for DashscopeAdaptor {
    fn channel_type(&self) -> &'static str {
        "dashscope"
    }

    fn default_models(&self) -> Vec<&'static str> {
        vec![
            "qwen3.8-max",
            "qwen3.7-plus",
            "qwen3.7-flash",
            "qwen3.5-omni-plus",
        ]
    }

    fn default_base_url(&self) -> &str {
        "https://dashscope.aliyuncs.com/compatible-mode/v1"
    }

    async fn test(&self, config: &ChannelConfig) -> Result<TestResult, anyhow::Error> {
        openai::test_openai_compatible(config).await
    }

    async fn forward(
        &self,
        request: &ProxyRequest,
        config: &ChannelConfig,
    ) -> Result<(u16, serde_json::Value, Option<super::TokenUsage>), anyhow::Error> {
        openai::forward_openai_compatible(request, config).await
    }

    async fn forward_stream(
        &self,
        request: &ProxyRequest,
        config: &ChannelConfig,
    ) -> Result<(Body, Option<UsageReceiver>), anyhow::Error> {
        openai::forward_stream_openai_compatible(request, config).await
    }
}
