use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 仪表盘统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    /// 今日请求数
    pub today_requests: i64,
    /// 今日 Token 消耗
    pub today_tokens: i64,
    /// 活跃渠道数
    pub active_channels: i64,
    /// 平均延迟（ms）
    pub avg_latency: i64,
    /// 累计请求数
    pub total_requests: i64,
    /// 累计 Token 消耗
    pub total_tokens: i64,
    /// 错误率（百分比）
    pub error_rate: f64,
}

/// 最近活动项（简化版日志）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecentActivity {
    pub id: String,
    pub model: String,
    pub channel_name: Option<String>,
    pub status_code: i32,
    pub total_tokens: i32,
    pub duration_ms: i32,
    pub created_at: String,
}

/// 仪表盘完整响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardResponse {
    pub stats: DashboardStats,
    pub recent_activities: Vec<RecentActivity>,
}
