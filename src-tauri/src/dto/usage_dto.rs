use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 每日用量数据点（DB 查询结果，仅包含有数据的日期）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DailyUsageData {
    pub date: String,
    pub requests: i64,
    pub tokens: i64,
}

/// 渠道用量项（DB 查询结果，percentage 由 Service 层计算）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ChannelUsageItem {
    pub channel_name: Option<String>,
    pub channel_type: String,
    pub requests: i64,
    pub tokens: i64,
    pub percentage: f64,
}

/// 模型用量项（DB 查询结果，percentage 由 Service 层计算）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsageItem {
    pub model: String,
    pub requests: i64,
    pub tokens: i64,
    pub percentage: f64,
}

/// 用量总览完整响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageOverviewResponse {
    pub total_requests: i64,
    pub total_tokens: i64,
    pub daily_data: Vec<DailyUsageData>,
    pub channel_usage: Vec<ChannelUsageItem>,
    pub model_usage: Vec<ModelUsageItem>,
}
