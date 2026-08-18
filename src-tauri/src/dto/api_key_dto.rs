use serde::{Deserialize, Serialize};

// ==================== ApiKey DTO ====================

/// 创建 API Key DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyDto {
    pub name: String,
    #[serde(default = "default_status")]
    pub status: i32,
    #[serde(default = "default_allowed_models")]
    pub allowed_models: String,
    #[serde(default = "default_allowed_channels")]
    pub allowed_channels: String,
    #[serde(default = "default_quota_limit")]
    pub quota_limit: i32,
    pub expires_at: Option<String>,
}

/// 更新 API Key DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApiKeyDto {
    pub id: String,
    pub name: Option<String>,
    pub status: Option<i32>,
    pub allowed_models: Option<String>,
    pub allowed_channels: Option<String>,
    pub quota_limit: Option<i32>,
    pub expires_at: Option<String>,
}

// 默认值函数
fn default_status() -> i32 {
    1
}

fn default_allowed_models() -> String {
    "[]".to_string()
}

fn default_allowed_channels() -> String {
    "[]".to_string()
}

fn default_quota_limit() -> i32 {
    -1
}
