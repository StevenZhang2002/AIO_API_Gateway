use serde::{Deserialize, Serialize};

// ==================== Channel DTO ====================

/// 创建渠道 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChannelDto {
    pub name: String,
    pub r#type: String,
    pub base_url: String,
    pub api_key: String,
    #[serde(default = "default_models")]
    pub models: String,
    #[serde(default = "default_status")]
    pub status: i32,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "default_weight")]
    pub weight: i32,
    #[serde(default = "default_config")]
    pub config: String,
    #[serde(default = "default_model_mapping")]
    pub model_mapping: String,
}

/// 更新渠道 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChannelDto {
    pub id: String,
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub models: Option<String>,
    pub status: Option<i32>,
    pub priority: Option<i32>,
    pub weight: Option<i32>,
    pub config: Option<String>,
    pub model_mapping: Option<String>,
}

/// 渠道类型默认配置 DTO——供前端创建渠道表单自动填充
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelDefaultsDto {
    /// 渠道类型标识（与 channels.type 一致）
    pub channel_type: String,
    /// 默认 API 地址
    pub base_url: String,
    /// 默认支持的模型列表
    pub models: Vec<String>,
}

/// 测试渠道连通性 DTO——前端发送的测试请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestChannelDto {
    pub r#type: String,
    pub base_url: String,
    pub api_key: String,
    #[serde(default = "default_models")]
    pub models: String,
}

// 默认值函数
fn default_models() -> String {
    "[]".to_string()
}

fn default_status() -> i32 {
    1
}

fn default_weight() -> i32 {
    1
}

fn default_config() -> String {
    "{}".to_string()
}

fn default_model_mapping() -> String {
    "{}".to_string()
}
