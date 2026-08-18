use async_trait::async_trait;

/// 请求体脱敏器 trait
///
/// 可插拔设计：首版提供 NoopDesensitizer 默认实现（不脱敏），
/// 后续可接入正则替换手机号/身份证/邮箱等 PII 信息。
#[async_trait]
pub trait Desensitizer: Send + Sync {
    /// 对请求体进行脱敏处理（如替换手机号、身份证等）
    async fn desensitize(&self, body: &mut serde_json::Value);
}

/// 默认实现：不进行任何脱敏
pub struct NoopDesensitizer;

#[async_trait]
impl Desensitizer for NoopDesensitizer {
    async fn desensitize(&self, _body: &mut serde_json::Value) {}
}
