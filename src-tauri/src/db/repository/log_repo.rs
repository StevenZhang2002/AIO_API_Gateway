use sqlx::{QueryBuilder, SqlitePool};

use crate::db::models::RequestLog;
use crate::dto::log_dto::{CreateLogDto, PaginatedResult, SearchLogDto};

pub struct LogRepo;

impl LogRepo {
    /// 创建日志
    pub async fn create(pool: &SqlitePool, dto: CreateLogDto) -> Result<RequestLog, sqlx::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query_as::<_, RequestLog>(
            r#"
            INSERT INTO request_logs (id, api_key_id, api_key_name, channel_id, channel_name, model, upstream_model, mode, status_code, prompt_tokens, completion_tokens, total_tokens, duration_ms, error_message, is_stream, is_retry, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .fetch_one(pool)
        .await
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
        if dto.api_key_id.is_some() { has_where = true; }
        if has_where {
            count_builder.push(" WHERE ");
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
}
