use sqlx::{QueryBuilder, SqlitePool};

use crate::db::models::ApiKey;
use crate::dto::api_key_dto::{CreateApiKeyDto, UpdateApiKeyDto};

pub struct ApiKeyRepo;

impl ApiKeyRepo {
    /// 创建 API Key
    pub async fn create(pool: &SqlitePool, dto: CreateApiKeyDto) -> Result<ApiKey, sqlx::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let key = format!("sk-aio-{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (id, name, key, status, allowed_models, allowed_channels, quota_limit, quota_used, expires_at, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&dto.name)
        .bind(&key)
        .bind(dto.status)
        .bind(&dto.allowed_models)
        .bind(&dto.allowed_channels)
        .bind(dto.quota_limit)
        .bind(&dto.expires_at)
        .bind(&now)
        .bind(&now)
        .fetch_one(pool)
        .await
    }

    /// 删除 API Key
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM api_keys WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 更新 API Key（仅更新 Some 字段）
    pub async fn update(pool: &SqlitePool, dto: UpdateApiKeyDto) -> Result<ApiKey, sqlx::Error> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new("UPDATE api_keys SET ");
        let mut first = true;

        macro_rules! push_set {
            ($col:literal, $val:expr) => {{
                if !first { builder.push(", "); }
                builder.push(concat!($col, " = ")).push_bind($val);
                first = false;
            }};
        }

        if let Some(ref v) = dto.name { push_set!("name", v); }
        if let Some(v) = dto.status { push_set!("status", v); }
        if let Some(ref v) = dto.allowed_models { push_set!("allowed_models", v); }
        if let Some(ref v) = dto.allowed_channels { push_set!("allowed_channels", v); }
        if let Some(v) = dto.quota_limit { push_set!("quota_limit", v); }
        if let Some(ref v) = dto.expires_at { push_set!("expires_at", v); }
        push_set!("updated_at", &now);

        builder.push(" WHERE id = ").push_bind(&dto.id);
        builder.push(" RETURNING *");

        builder.build_query_as::<ApiKey>().fetch_one(pool).await
    }

    /// 获取所有 API Key
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<ApiKey>, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
    }

    /// 根据 key 查询已启用的 API Key（用于鉴权）
    pub async fn find_by_key(pool: &SqlitePool, key: &str) -> Result<Option<ApiKey>, sqlx::Error> {
        sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE key = ? AND status = 1",
        )
        .bind(key)
        .fetch_optional(pool)
        .await
    }
}
