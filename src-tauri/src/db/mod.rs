pub mod models;
pub mod repository;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::fs;
use std::path::PathBuf;

/// 初始化 SQLite 数据库连接池
///
/// # Arguments
/// * `app_data_dir` - 应用数据目录，由 Tauri 的 `app.path().app_data_dir()` 提供
///
/// # Returns
/// 返回 SqlitePool 连接池实例
pub async fn init_pool(app_data_dir: PathBuf) -> Result<SqlitePool, sqlx::Error> {
    // 确保数据目录存在
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir).expect("无法创建数据目录");
    }

    let db_path = app_data_dir.join("aio_gateway.db");
    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .connect(&database_url)
        .await?;

    // 执行数据库迁移
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("数据库迁移失败");

    println!("数据库连接池初始化成功: {}", db_path.display());

    Ok(pool)
}
