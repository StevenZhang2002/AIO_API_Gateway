use sqlx::{QueryBuilder, SqlitePool};

use crate::db::models::RequestLog;
use crate::dto::dashboard_dto::{DashboardStats, RecentActivity};
use crate::dto::log_dto::{CreateLogDto, PaginatedResult, SearchLogDto};
use crate::dto::usage_dto::{ChannelUsageItem, DailyUsageData, ModelUsageItem};

pub struct LogRepo;

impl LogRepo {
    /// 创建日志
    pub async fn create(pool: &SqlitePool, dto: CreateLogDto) -> Result<RequestLog, sqlx::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query_as::<_, RequestLog>(
            r#"
            INSERT INTO request_logs (id, api_key_id, api_key_name, channel_id, channel_name, model, upstream_model, mode, status_code, prompt_tokens, completion_tokens, total_tokens, duration_ms, error_message, is_stream, is_retry, created_at, request_body, response_body)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&dto.api_key_id)
        .bind(&dto.api_key_name)
        .bind(&dto.channel_id)
        .bind(&dto.channel_name)
        .bind(&dto.model)
        .bind(&dto.upstream_model)
        .bind(&dto.mode)
        .bind(dto.status_code)
        .bind(dto.prompt_tokens)
        .bind(dto.completion_tokens)
        .bind(dto.total_tokens)
        .bind(dto.duration_ms)
        .bind(&dto.error_message)
        .bind(dto.is_stream)
        .bind(dto.is_retry)
        .bind(&now)
        .bind(&dto.request_body)
        .bind(&dto.response_body)
        .fetch_one(pool)
        .await
    }

    /// 流式请求结束后回写 token 用量
    pub async fn update_usage(
        pool: &SqlitePool,
        id: &str,
        prompt_tokens: i32,
        completion_tokens: i32,
        total_tokens: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE request_logs SET prompt_tokens = ?, completion_tokens = ?, total_tokens = ? WHERE id = ?",
        )
        .bind(prompt_tokens)
        .bind(completion_tokens)
        .bind(total_tokens)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 搜索日志（支持多条件动态查询和分页）
    pub async fn search(
        pool: &SqlitePool,
        dto: SearchLogDto,
    ) -> Result<PaginatedResult<RequestLog>, sqlx::Error> {
        // 构建 COUNT 查询
        let mut count_builder: QueryBuilder<sqlx::Sqlite> =
            QueryBuilder::new("SELECT COUNT(*) FROM request_logs");
        let mut first = true;

        macro_rules! push_cond {
            ($builder:expr, $cond:literal, $val:expr) => {{
                if !first { $builder.push(" AND "); }
                $builder.push($cond).push_bind($val);
                first = false;
            }};
        }

        let mut has_where = false;
        if dto.keyword.is_some() || dto.api_key_id.is_some() {
            has_where = true;
        }
        if has_where {
            count_builder.push(" WHERE ");
            if let Some(ref v) = dto.keyword {
                let like = format!("%{}%", v);
                push_cond!(count_builder, "(model LIKE ", like.clone());
                count_builder.push(" OR api_key_name LIKE ").push_bind(like.clone());
                count_builder.push(" OR channel_name LIKE ").push_bind(like);
                count_builder.push(")");
            }
            if let Some(ref v) = dto.api_key_id { push_cond!(count_builder, "api_key_id = ", v); }
            if let Some(ref v) = dto.channel_id { push_cond!(count_builder, "channel_id = ", v); }
            if let Some(ref v) = dto.model { push_cond!(count_builder, "model LIKE ", format!("%{}%", v)); }
            if let Some(v) = dto.status_code { push_cond!(count_builder, "status_code = ", v); }
            if let Some(v) = dto.is_stream { push_cond!(count_builder, "is_stream = ", v); }
            if let Some(v) = dto.is_retry { push_cond!(count_builder, "is_retry = ", v); }
            if let Some(ref v) = dto.start_time { push_cond!(count_builder, "created_at >= ", v); }
            if let Some(ref v) = dto.end_time { push_cond!(count_builder, "created_at <= ", v); }
        }

        // 获取总数
        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(pool)
            .await?;

        // 构建分页查询
        let mut query_builder: QueryBuilder<sqlx::Sqlite> =
            QueryBuilder::new("SELECT * FROM request_logs");
        first = true;
        if has_where {
            query_builder.push(" WHERE ");
            if let Some(ref v) = dto.keyword {
                let like = format!("%{}%", v);
                push_cond!(query_builder, "(model LIKE ", like.clone());
                query_builder.push(" OR api_key_name LIKE ").push_bind(like.clone());
                query_builder.push(" OR channel_name LIKE ").push_bind(like);
                query_builder.push(")");
            }
            if let Some(ref v) = dto.api_key_id { push_cond!(query_builder, "api_key_id = ", v); }
            if let Some(ref v) = dto.channel_id { push_cond!(query_builder, "channel_id = ", v); }
            if let Some(ref v) = dto.model { push_cond!(query_builder, "model LIKE ", format!("%{}%", v)); }
            if let Some(v) = dto.status_code { push_cond!(query_builder, "status_code = ", v); }
            if let Some(v) = dto.is_stream { push_cond!(query_builder, "is_stream = ", v); }
            if let Some(v) = dto.is_retry { push_cond!(query_builder, "is_retry = ", v); }
            if let Some(ref v) = dto.start_time { push_cond!(query_builder, "created_at >= ", v); }
            if let Some(ref v) = dto.end_time { push_cond!(query_builder, "created_at <= ", v); }
        }

        // 排序和分页
        let offset = (dto.page - 1) * dto.page_size;
        query_builder
            .push(" ORDER BY created_at DESC LIMIT ")
            .push_bind(dto.page_size)
            .push(" OFFSET ")
            .push_bind(offset);

        let items = query_builder
            .build_query_as::<RequestLog>()
            .fetch_all(pool)
            .await?;

        let total_pages = (total + dto.page_size - 1) / dto.page_size;

        Ok(PaginatedResult {
            items,
            total,
            page: dto.page,
            page_size: dto.page_size,
            total_pages,
        })
    }

    /// 获取仪表盘统计数据
    pub async fn get_dashboard_stats(pool: &SqlitePool) -> Result<DashboardStats, sqlx::Error> {
        let today_start = chrono::Local::now()
            .format("%Y-%m-%dT00:00:00")
            .to_string();

        // 今日统计
        let row: (i64, i64, f64) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(*) as cnt,
                COALESCE(SUM(total_tokens), 0) as tokens,
                COALESCE(AVG(duration_ms), 0.0) as avg_ms
            FROM request_logs 
            WHERE created_at >= ?
            "#,
        )
        .bind(&today_start)
        .fetch_one(pool)
        .await?;

        let (today_requests, today_tokens, avg_latency_f) = row;

        // 累计统计
        let total_row: (i64, i64) = sqlx::query_as(
            "SELECT COUNT(*), COALESCE(SUM(total_tokens), 0) FROM request_logs",
        )
        .fetch_one(pool)
        .await?;
        let (total_requests, total_tokens) = total_row;

        // 错误率
        let err_row: (i64, i64) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(*),
                COUNT(CASE WHEN status_code >= 400 THEN 1 END)
            FROM request_logs
            "#,
        )
        .fetch_one(pool)
        .await?;
        let (total_count, error_count) = err_row;
        let error_rate = if total_count > 0 {
            (error_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };

        Ok(DashboardStats {
            today_requests,
            today_tokens,
            active_channels: 0, // 由 Service 层填充
            avg_latency: avg_latency_f as i64,
            total_requests,
            total_tokens,
            error_rate,
        })
    }

    /// 获取最近活动（最近 N 条日志摘要）
    pub async fn get_recent_activities(
        pool: &SqlitePool,
        limit: i64,
    ) -> Result<Vec<RecentActivity>, sqlx::Error> {
        sqlx::query_as::<_, RecentActivity>(
            r#"
            SELECT id, model, channel_name, status_code, total_tokens, duration_ms, created_at
            FROM request_logs
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    // ==================== Usage 聚合查询 ====================

    /// 获取总请求数和总 Token 消耗
    pub async fn get_total_requests_and_tokens(
        pool: &SqlitePool,
    ) -> Result<(i64, i64), sqlx::Error> {
        let row: (i64, i64) = sqlx::query_as(
            "SELECT COUNT(*), COALESCE(SUM(total_tokens), 0) FROM request_logs",
        )
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    /// 获取近 30 天每日请求数与 Token 消耗（仅返回有数据的日期）
    pub async fn get_daily_usage(
        pool: &SqlitePool,
    ) -> Result<Vec<DailyUsageData>, sqlx::Error> {
        let thirty_days_ago = (chrono::Local::now().date_naive() - chrono::Duration::days(29))
            .format("%Y-%m-%d")
            .to_string();

        sqlx::query_as::<_, DailyUsageData>(
            r#"
            SELECT date(created_at) as date, COUNT(*) as requests, COALESCE(SUM(total_tokens), 0) as tokens
            FROM request_logs
            WHERE date(created_at) >= ?
            GROUP BY date(created_at)
            ORDER BY date
            "#,
        )
        .bind(&thirty_days_ago)
        .fetch_all(pool)
        .await
    }

    /// 获取各渠道用量分布（LEFT JOIN channels 获取渠道类型）
    pub async fn get_channel_usage(
        pool: &SqlitePool,
    ) -> Result<Vec<ChannelUsageItem>, sqlx::Error> {
        sqlx::query_as::<_, ChannelUsageItem>(
            r#"
            SELECT 
                l.channel_name,
                COALESCE(c.type, 'unknown') as channel_type,
                COUNT(*) as requests,
                COALESCE(SUM(l.total_tokens), 0) as tokens,
                0.0 as percentage
            FROM request_logs l
            LEFT JOIN channels c ON l.channel_id = c.id
            GROUP BY l.channel_id, l.channel_name, c.type
            ORDER BY requests DESC
            "#,
        )
        .fetch_all(pool)
        .await
    }

    /// 获取各模型用量分布
    pub async fn get_model_usage(
        pool: &SqlitePool,
    ) -> Result<Vec<ModelUsageItem>, sqlx::Error> {
        sqlx::query_as::<_, ModelUsageItem>(
            r#"
            SELECT 
                model,
                COUNT(*) as requests,
                COALESCE(SUM(total_tokens), 0) as tokens,
                0.0 as percentage
            FROM request_logs
            GROUP BY model
            ORDER BY requests DESC
            "#,
        )
        .fetch_all(pool)
        .await
    }
}
