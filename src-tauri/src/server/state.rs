use std::sync::Arc;

use sqlx::SqlitePool;

use crate::adaptor::AdaptorRegistry;

/// 应用状态（共享给所有 handler）
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub registry: Arc<AdaptorRegistry>,
}
