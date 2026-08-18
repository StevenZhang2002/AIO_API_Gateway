mod adaptor;
mod db;
mod dto;
mod server;
mod services;

use std::sync::Arc;

use adaptor::AdaptorRegistry;
use adaptor::ChannelConfig;
use services::api_key_service::ApiKeyService;
use services::channel_service::ChannelService;
use sqlx::SqlitePool;
use tauri::{Manager, State};

use crate::db::models::{ApiKey, Channel, RequestLog};
use crate::dto::api_key_dto::{CreateApiKeyDto, UpdateApiKeyDto};
use crate::dto::channel_dto::{ChannelDefaultsDto, CreateChannelDto, TestChannelDto, UpdateChannelDto};
use crate::dto::dashboard_dto::DashboardResponse;
use crate::dto::log_dto::{PaginatedResult, SearchLogDto};
use crate::dto::usage_dto::UsageOverviewResponse;
use crate::db::repository::log_repo::LogRepo;
use services::dashboard_service::DashboardService;
use services::usage_service::UsageService;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ==================== Channel Commands ====================

#[tauri::command]
async fn create_channel(pool: State<'_, SqlitePool>, dto: CreateChannelDto) -> Result<Channel, String> {
    ChannelService::create(&*pool, dto)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_channel(pool: State<'_, SqlitePool>, id: String) -> Result<(), String> {
    ChannelService::delete(&*pool, &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_all_channels(pool: State<'_, SqlitePool>) -> Result<Vec<Channel>, String> {
    ChannelService::find_all(&*pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_enabled_channels(pool: State<'_, SqlitePool>) -> Result<Vec<Channel>, String> {
    ChannelService::find_enabled(&*pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_channel(pool: State<'_, SqlitePool>, dto: UpdateChannelDto) -> Result<Channel, String> {
    ChannelService::update(&*pool, dto)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_channel_defaults(registry: State<'_, AdaptorRegistry>) -> Vec<ChannelDefaultsDto> {
    registry.defaults()
}

#[tauri::command]
async fn test_channel(
    registry: State<'_, AdaptorRegistry>,
    dto: TestChannelDto,
) -> Result<adaptor::TestResult, String> {
    let adaptor = registry
        .get(&dto.r#type)
        .ok_or_else(|| format!("不支持的渠道类型: {}", dto.r#type))?;

    let models: Vec<String> =
        serde_json::from_str(&dto.models).unwrap_or_default();

    let config = ChannelConfig {
        base_url: dto.base_url,
        api_key: dto.api_key,
        models,
        model_mapping: serde_json::Value::Object(Default::default()),
        extra: serde_json::Value::Object(Default::default()),
    };

    adaptor
        .test(&config)
        .await
        .map_err(|e| e.to_string())
}

// ==================== API Key Commands ====================

#[tauri::command]
async fn create_api_key(pool: State<'_, SqlitePool>, dto: CreateApiKeyDto) -> Result<ApiKey, String> {
    ApiKeyService::create(&*pool, dto)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_api_key(pool: State<'_, SqlitePool>, id: String) -> Result<(), String> {
    ApiKeyService::delete(&*pool, &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_all_api_keys(pool: State<'_, SqlitePool>) -> Result<Vec<ApiKey>, String> {
    ApiKeyService::find_all(&*pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_api_key(pool: State<'_, SqlitePool>, dto: UpdateApiKeyDto) -> Result<ApiKey, String> {
    ApiKeyService::update(&*pool, dto)
        .await
        .map_err(|e| e.to_string())
}

// ==================== Log Commands ====================

#[tauri::command]
async fn search_logs(
    pool: State<'_, SqlitePool>,
    dto: SearchLogDto,
) -> Result<PaginatedResult<RequestLog>, String> {
    LogRepo::search(&*pool, dto)
        .await
        .map_err(|e| e.to_string())
}

// ==================== Dashboard Commands ====================

#[tauri::command]
async fn get_dashboard_stats(pool: State<'_, SqlitePool>) -> Result<DashboardResponse, String> {
    DashboardService::get_dashboard(&*pool)
        .await
        .map_err(|e| e.to_string())
}

// ==================== Usage Commands ====================

#[tauri::command]
async fn get_usage_overview(pool: State<'_, SqlitePool>) -> Result<UsageOverviewResponse, String> {
    UsageService::get_usage_overview(&*pool)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("获取应用数据目录失败");
            let pool = tauri::async_runtime::block_on(async {
                db::init_pool(app_data_dir).await.expect("数据库初始化失败")
            });
            let registry = Arc::new(AdaptorRegistry::new());

            app.manage(pool.clone());
            app.manage(AdaptorRegistry::new());

            // 启动代理 HTTP 服务器（后台 task）
            let server_pool = pool;
            let server_registry = registry.clone();
            let proxy_port = 8080; // 首版硬编码端口
            tauri::async_runtime::spawn(async move {
                server::start_http_server(proxy_port, server_pool, server_registry).await;
            });

            println!("🚀 AIO Gateway 启动成功");
            println!("   代理端点: http://localhost:{}/v1/chat/completions", proxy_port);
            println!("   健康检查: http://localhost:{}/health", proxy_port);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            create_channel,
            delete_channel,
            get_all_channels,
            get_enabled_channels,
            update_channel,
            get_channel_defaults,
            test_channel,
            create_api_key,
            delete_api_key,
            get_all_api_keys,
            update_api_key,
            search_logs,
            get_dashboard_stats,
            get_usage_overview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
