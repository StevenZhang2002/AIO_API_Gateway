use sqlx::SqlitePool;
use thiserror::Error;

use crate::db::models::Channel;
use crate::db::repository::channel_repo::ChannelRepo;
use crate::dto::channel_dto::{CreateChannelDto, UpdateChannelDto};

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("渠道不存在: {0}")]
    NotFound(String),

    #[error("参数校验失败: {0}")]
    ValidationError(String),

    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

pub struct ChannelService;

impl ChannelService {
    /// 创建渠道
    pub async fn create(pool: &SqlitePool, dto: CreateChannelDto) -> Result<Channel, ServiceError> {
        // 参数校验
        if dto.name.trim().is_empty() {
            return Err(ServiceError::ValidationError("渠道名称不能为空".to_string()));
        }
        if dto.base_url.trim().is_empty() {
            return Err(ServiceError::ValidationError("Base URL 不能为空".to_string()));
        }
        if dto.api_key.trim().is_empty() {
            return Err(ServiceError::ValidationError("API Key 不能为空".to_string()));
        }

        let channel = ChannelRepo::create(pool, dto).await?;
        Ok(channel)
    }

    /// 删除渠道
    pub async fn delete(pool: &SqlitePool, id: &str) -> Result<(), ServiceError> {
        // 检查渠道是否存在
        let channel = sqlx::query_as::<_, Channel>("SELECT * FROM channels WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        if channel.is_none() {
            return Err(ServiceError::NotFound(format!("渠道 {} 不存在", id)));
        }

        ChannelRepo::delete(pool, id).await?;
        Ok(())
    }

    /// 获取所有渠道
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Channel>, ServiceError> {
        let channels = ChannelRepo::find_all(pool).await?;
        Ok(channels)
    }

    /// 获取已启用的渠道
    pub async fn find_enabled(pool: &SqlitePool) -> Result<Vec<Channel>, ServiceError> {
        let channels = ChannelRepo::find_enabled(pool).await?;
        Ok(channels)
    }

    /// 更新渠道
    pub async fn update(pool: &SqlitePool, dto: UpdateChannelDto) -> Result<Channel, ServiceError> {
        // 参数校验（仅对 Some 字段）
        if let Some(ref name) = dto.name {
            if name.trim().is_empty() {
                return Err(ServiceError::ValidationError("渠道名称不能为空".to_string()));
            }
        }
        if let Some(ref base_url) = dto.base_url {
            if base_url.trim().is_empty() {
                return Err(ServiceError::ValidationError("Base URL 不能为空".to_string()));
            }
        }
        if let Some(ref api_key) = dto.api_key {
            if api_key.trim().is_empty() {
                return Err(ServiceError::ValidationError("API Key 不能为空".to_string()));
            }
        }

        // 检查渠道是否存在
        let channel = sqlx::query_as::<_, Channel>("SELECT * FROM channels WHERE id = ?")
            .bind(&dto.id)
            .fetch_optional(pool)
            .await?;

        if channel.is_none() {
            return Err(ServiceError::NotFound(format!("渠道 {} 不存在", dto.id)));
        }

        let updated = ChannelRepo::update(pool, dto).await?;
        Ok(updated)
    }
}
