use sqlx::{QueryBuilder, SqlitePool};

use crate::db::models::Channel;
use crate::dto::channel_dto::{CreateChannelDto, UpdateChannelDto};

pub struct ChannelRepo;

impl ChannelRepo {
    /// 创建渠道
    pub async fn create(pool: &SqlitePool, dto: CreateChannelDto) -> Result<Channel, sqlx::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query_as::<_, Channel>(
            r#"
            INSERT INTO channels (id, name, type, base_url, api_key, models, status, priority, weight, config, model_mapping, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&dto.name)
        .bind(&dto.r#type)
        .bind(&dto.base_url)
        .bind(&dto.api_key)
        .bind(&dto.models)
        .bind(dto.status)
        .bind(dto.priority)
        .bind(dto.weight)
        .bind(&dto.config)
        .bind(&dto.model_mapping)
        .bind(&now)
        .bind(&now)
        .fetch_one(pool)
        .await
    }

    /// 删除渠道
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM channels WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 获取所有渠道
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Channel>, sqlx::Error> {
        sqlx::query_as::<_, Channel>("SELECT * FROM channels ORDER BY priority DESC, created_at DESC")
            .fetch_all(pool)
            .await
    }

    /// 获取已启用的渠道（status = 1）
    pub async fn find_enabled(pool: &SqlitePool) -> Result<Vec<Channel>, sqlx::Error> {
        sqlx::query_as::<_, Channel>(
            "SELECT * FROM channels WHERE status = 1 ORDER BY priority DESC, created_at DESC",
        )
        .fetch_all(pool)
        .await
    }

    /// 更新渠道（仅更新 Some 字段）
    pub async fn update(pool: &SqlitePool, dto: UpdateChannelDto) -> Result<Channel, sqlx::Error> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new("UPDATE channels SET ");
        let mut first = true;

        macro_rules! push_set {
            ($col:literal, $val:expr) => {{
                if !first { builder.push(", "); }
                builder.push(concat!($col, " = ")).push_bind($val);
                first = false;
            }};
        }

        if let Some(ref v) = dto.name { push_set!("name", v); }
        if let Some(ref v) = dto.r#type { push_set!("type", v); }
        if let Some(ref v) = dto.base_url { push_set!("base_url", v); }
        if let Some(ref v) = dto.api_key { push_set!("api_key", v); }
        if let Some(ref v) = dto.models { push_set!("models", v); }
        if let Some(v) = dto.status { push_set!("status", v); }
        if let Some(v) = dto.priority { push_set!("priority", v); }
        if let Some(v) = dto.weight { push_set!("weight", v); }
        if let Some(ref v) = dto.config { push_set!("config", v); }
        if let Some(ref v) = dto.model_mapping { push_set!("model_mapping", v); }
        push_set!("updated_at", &now);

        builder.push(" WHERE id = ").push_bind(&dto.id);
        builder.push(" RETURNING *");

        builder.build_query_as::<Channel>().fetch_one(pool).await
    }

    /// 统计已启用的渠道数量
    pub async fn count_enabled(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COUNT(*) FROM channels WHERE status = 1")
            .fetch_one(pool)
            .await
    }
}
