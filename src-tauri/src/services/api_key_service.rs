use sqlx::SqlitePool;
use thiserror::Error;

use crate::db::models::ApiKey;
use crate::db::repository::api_key_repo::ApiKeyRepo;
use crate::dto::api_key_dto::{CreateApiKeyDto, UpdateApiKeyDto};

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("API Key 不存在: {0}")]
    NotFound(String),

    #[error("参数校验失败: {0}")]
    ValidationError(String),

    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

pub struct ApiKeyService;

impl ApiKeyService {
    /// 创建 API Key
    pub async fn create(pool: &SqlitePool, dto: CreateApiKeyDto) -> Result<ApiKey, ServiceError> {
        // 参数校验
        if dto.name.trim().is_empty() {
            return Err(ServiceError::ValidationError("API Key 名称不能为空".to_string()));
        }

        let api_key = ApiKeyRepo::create(pool, dto).await?;
        Ok(api_key)
    }

    /// 删除 API Key
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), ServiceError> {
        // 检查是否存在
        let api_key = sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        if api_key.is_none() {
            return Err(ServiceError::NotFound(format!("API Key {} 不存在", id)));
        }

        ApiKeyRepo::delete(pool, id).await?;
        Ok(())
    }

    /// 获取所有 API Key
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<ApiKey>, ServiceError> {
        let api_keys = ApiKeyRepo::find_all(pool).await?;
        Ok(api_keys)
    }

    /// 更新 API Key
    pub async fn update(pool: &SqlitePool, dto: UpdateApiKeyDto) -> Result<ApiKey, ServiceError> {
        // 参数校验（仅对 Some 字段）
        if let Some(ref name) = dto.name {
            if name.trim().is_empty() {
                return Err(ServiceError::ValidationError("API Key 名称不能为空".to_string()));
            }
        }

        // 检查是否存在
        let api_key = sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys WHERE id = ?")
            .bind(&dto.id)
            .fetch_optional(pool)
            .await?;

        if api_key.is_none() {
            return Err(ServiceError::NotFound(format!("API Key {} 不存在", dto.id)));
        }

        let updated = ApiKeyRepo::update(pool, dto).await?;
        Ok(updated)
    }
}
