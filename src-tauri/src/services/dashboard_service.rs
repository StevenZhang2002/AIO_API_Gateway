use crate::db::repository::channel_repo::ChannelRepo;
use crate::db::repository::log_repo::LogRepo;
use crate::dto::dashboard_dto::DashboardResponse;
use sqlx::SqlitePool;

pub struct DashboardService;

impl DashboardService {
    /// 获取仪表盘完整数据
    pub async fn get_dashboard(pool: &SqlitePool) -> Result<DashboardResponse, sqlx::Error> {
        let mut stats = LogRepo::get_dashboard_stats(pool).await?;

        // 填充活跃渠道数
        let active_channels = ChannelRepo::count_enabled(pool).await?;
        stats.active_channels = active_channels;

        // 获取最近活动
        let recent_activities = LogRepo::get_recent_activities(pool, 5).await?;

        Ok(DashboardResponse {
            stats,
            recent_activities,
        })
    }
}
