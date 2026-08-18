use async_trait::async_trait;
use axum::body::Body;

use super::openai::{
    forward_openai_compatible, forward_stream_openai_compatible, test_openai_compatible,
};
use super::{Adaptor, ChannelConfig, ProxyRequest, TestResult, TokenUsage, UsageReceiver};

/// DeepSeek 渠道适配器（OpenAI 兼容格式，复用 OpenAI 实现）
pub struct DeepSeekAdaptor;

#[async_trait]
impl Adaptor for DeepSeekAdaptor {
    fn channel_type(&self) -> &'static str {
        "deepseek"
    }

    fn default_models(&self) -> Vec<&'static str> {
        vec!["deepseek-v4-flash", "deepseek-v4-pro"]
    }

    fn default_base_url(&self) -> &str {
        "https://api.deepseek.com/v1"
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
