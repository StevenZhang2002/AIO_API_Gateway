use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ==================== Channel ====================

/// 渠道表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub base_url: String,
    pub api_key: String,
    pub models: String,
    pub status: i32,
    pub priority: i32,
    pub weight: i32,
    pub config: String,
    pub model_mapping: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_test_at: Option<String>,
    pub last_test_ok: Option<bool>,
}

// ==================== ApiKey ====================

/// API Key 表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key: String,
    pub status: i32,
    pub allowed_models: String,
    pub allowed_channels: String,
    pub quota_limit: i32,
    pub quota_used: i32,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ==================== RequestLog ====================

/// 请求日志表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RequestLog {
    pub id: String,
    pub api_key_id: Option<String>,
    pub api_key_name: Option<String>,
    pub channel_id: Option<String>,
    pub channel_name: Option<String>,
    pub model: String,
    pub upstream_model: Option<String>,
    pub mode: String,
    pub status_code: i32,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    pub duration_ms: i32,
    pub error_message: Option<String>,
    pub is_stream: bool,
    pub is_retry: bool,
    pub created_at: String,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
}
