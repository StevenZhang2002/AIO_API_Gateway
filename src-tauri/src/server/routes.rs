use axum::routing::{get, post};
use axum::Router;

use super::state::AppState;
use crate::services::proxy::ProxyService;

/// 代理路由
pub fn proxy_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/chat/completions", post(proxy_handler))
        .route("/health", get(health_handler))
}

/// 代理处理器：POST /v1/chat/completions
async fn proxy_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> impl axum::response::IntoResponse {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    ProxyService::handle(&state.pool, &state.registry, auth_header, body)
        .await
}

/// 健康检查：GET /health
async fn health_handler() -> &'static str {
    "OK"
}
