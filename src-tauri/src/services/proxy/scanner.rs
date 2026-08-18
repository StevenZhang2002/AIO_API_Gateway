use async_trait::async_trait;

/// 请求/响应安全扫描器 trait
///
/// 可插拔设计：首版提供 PassThroughScanner 默认实现（不拦截），
/// 后续可接入关键词正则、敏感词库、外部审核 API 等。
#[async_trait]
pub trait RequestScanner: Send + Sync {
    /// 扫描请求体，返回 Some(原因) 表示拦截，None 表示放行
    async fn scan_request(&self, body: &serde_json::Value) -> Option<String>;

    /// 扫描响应体，返回 Some(原因) 表示拦截，None 表示放行
    async fn scan_response(&self, body: &serde_json::Value) -> Option<String>;
}

/// 默认实现：不拦截任何请求
pub struct PassThroughScanner;

#[async_trait]
impl RequestScanner for PassThroughScanner {
    async fn scan_request(&self, _body: &serde_json::Value) -> Option<String> {
        None
    }

    async fn scan_response(&self, _body: &serde_json::Value) -> Option<String> {
        None
    }
}
