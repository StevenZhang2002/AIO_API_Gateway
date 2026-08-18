use crate::db::repository::log_repo::LogRepo;
use crate::dto::usage_dto::{
    ChannelUsageItem, DailyUsageData, ModelUsageItem, UsageOverviewResponse,
};
use sqlx::SqlitePool;

pub struct UsageService;

impl UsageService {
    /// 获取用量总览（近 30 天）
    pub async fn get_usage_overview(pool: &SqlitePool) -> Result<UsageOverviewResponse, sqlx::Error> {
        // 总量
        let (total_requests, total_tokens) = LogRepo::get_total_requests_and_tokens(pool).await?;

        // 每日趋势（DB 仅返回有数据的日期）
        let raw_daily = LogRepo::get_daily_usage(pool).await?;

        // 填充完整 30 天，缺失日期补零
        let daily_data = Self::fill_daily_gaps(raw_daily);

        // 渠道用量
        let raw_channels = LogRepo::get_channel_usage(pool).await?;
        let channel_usage = Self::calc_percentages(raw_channels, total_requests);

        // 模型用量
        let raw_models = LogRepo::get_model_usage(pool).await?;
        let model_usage = Self::calc_percentages(raw_models, total_requests);

        Ok(UsageOverviewResponse {
            total_requests,
            total_tokens,
            daily_data,
            channel_usage,
            model_usage,
        })
    }

    /// 填充 30 天日期空洞（无数据日期补零）
    fn fill_daily_gaps(raw: Vec<DailyUsageData>) -> Vec<DailyUsageData> {
        use std::collections::HashMap;

        let map: HashMap<String, DailyUsageData> =
            raw.into_iter().map(|d| (d.date.clone(), d)).collect();

        let today = chrono::Local::now().date_naive();
        (0..30)
            .rev()
            .map(|i| {
                let date = today - chrono::Duration::days(i);
                let date_str = date.format("%Y-%m-%d").to_string();
                map.get(&date_str).cloned().unwrap_or(DailyUsageData {
                    date: date_str,
                    requests: 0,
                    tokens: 0,
                })
            })
            .collect()
    }

    /// 计算各项占比（百分比保留一位小数）
    fn calc_percentages<T: HasRequests>(items: Vec<T>, total: i64) -> Vec<T> {
        items
            .into_iter()
            .map(|mut item| {
                let pct = if total > 0 {
                    (*item.requests() as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                *item.percentage() = (pct * 10.0).round() / 10.0;
                item
            })
            .collect()
    }
}

/// 辅助 trait：统一渠道/模型用量的百分比计算
trait HasRequests {
    fn requests(&self) -> &i64;
    fn percentage(&mut self) -> &mut f64;
}

impl HasRequests for ChannelUsageItem {
    fn requests(&self) -> &i64 {
        &self.requests
    }
    fn percentage(&mut self) -> &mut f64 {
        &mut self.percentage
    }
}

impl HasRequests for ModelUsageItem {
    fn requests(&self) -> &i64 {
        &self.requests
    }
    fn percentage(&mut self) -> &mut f64 {
        &mut self.percentage
    }
}
