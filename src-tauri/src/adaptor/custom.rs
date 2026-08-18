use async_trait::async_trait;
use axum::body::Body;

use super::openai::{
    forward_openai_compatible, forward_stream_openai_compatible, test_openai_compatible,
};
use super::{Adaptor, ChannelConfig, ProxyRequest, TestResult, TokenUsage, UsageReceiver};

/// 自定义渠道适配器（任意 OpenAI 兼容服务：Ollama / vLLM / LM Studio 等）
pub struct CustomAdaptor;

#[async_trait]
impl Adaptor for CustomAdaptor {
    fn channel_type(&self) -> &'static str {
        "custom"
    }

    fn default_models(&self) -> Vec<&'static str> {
        vec![]
    }

    fn default_base_url(&self) -> &str {
        ""
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
