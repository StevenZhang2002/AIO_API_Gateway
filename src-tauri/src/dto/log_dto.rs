use serde::{Deserialize, Serialize};

// ==================== Log DTO ====================

/// 创建日志 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogDto {
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
    pub request_body: Option<String>,
    pub response_body: Option<String>,
}

/// 日志搜索 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchLogDto {
    /// 关键词模糊搜索（匹配 model / api_key_name / channel_name）
    pub keyword: Option<String>,
    /// API Key ID
    pub api_key_id: Option<String>,
    /// 渠道 ID
    pub channel_id: Option<String>,
    /// 模型名称（模糊匹配）
    pub model: Option<String>,
    /// HTTP 状态码
    pub status_code: Option<i32>,
    /// 是否流式请求
    pub is_stream: Option<bool>,
    /// 是否重试请求
    pub is_retry: Option<bool>,
    /// 开始时间（ISO 8601）
    pub start_time: Option<String>,
    /// 结束时间（ISO 8601）
    pub end_time: Option<String>,
    /// 页码，默认 1
    #[serde(default = "default_page")]
    pub page: i64,
    /// 每页数量，默认 20
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

/// 分页结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}
