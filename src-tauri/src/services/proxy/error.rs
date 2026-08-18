use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

/// 代理服务错误
#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("未提供 API Key")]
    MissingApiKey,

    #[error("API Key 无效或已禁用")]
    InvalidApiKey,

    #[error("API Key 已过期")]
    ExpiredApiKey,

    #[error("配额已用尽")]
    QuotaExhausted,

    #[error("请求体格式错误: {0}")]
    BadRequest(String),

    #[error("请求被安全策略拦截: {0}")]
    Blocked(String),

    #[error("模型不允许访问: {0}")]
    ModelNotAllowed(String),

    #[error("无可用渠道: {0}")]
    NoAvailableChannel(String),

    #[error("所有渠道均失败")]
    AllChannelsFailed,

    #[error("上游请求失败: {0}")]
    UpstreamError(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

impl ProxyError {
    /// 获取对应的 HTTP 状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            ProxyError::MissingApiKey | ProxyError::InvalidApiKey | ProxyError::ExpiredApiKey => {
                StatusCode::UNAUTHORIZED
            }
            ProxyError::QuotaExhausted => StatusCode::TOO_MANY_REQUESTS,
            ProxyError::ModelNotAllowed(_) => StatusCode::FORBIDDEN,
            ProxyError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ProxyError::Blocked(_) => StatusCode::from_u16(451).unwrap(),
            ProxyError::NoAvailableChannel(_) => StatusCode::SERVICE_UNAVAILABLE,
            ProxyError::AllChannelsFailed | ProxyError::UpstreamError(_) => {
                StatusCode::BAD_GATEWAY
            }
            ProxyError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// 获取错误类型标识
    pub fn error_type(&self) -> &'static str {
        match self {
            ProxyError::MissingApiKey | ProxyError::InvalidApiKey | ProxyError::ExpiredApiKey => {
                "authentication_error"
            }
            ProxyError::QuotaExhausted => "quota_exceeded",
            ProxyError::ModelNotAllowed(_) => "model_not_allowed",
            ProxyError::BadRequest(_) => "invalid_request_error",
            ProxyError::Blocked(_) => "content_policy_violation",
            ProxyError::NoAvailableChannel(_) => "no_available_channel",
            ProxyError::AllChannelsFailed => "all_channels_failed",
            ProxyError::UpstreamError(_) => "upstream_error",
            ProxyError::Internal(_) => "internal_error",
        }
    }
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = json!({
            "error": {
                "message": self.to_string(),
                "type": self.error_type(),
                "code": status.as_u16(),
            }
        });
        (status, Json(body)).into_response()
    }
}
