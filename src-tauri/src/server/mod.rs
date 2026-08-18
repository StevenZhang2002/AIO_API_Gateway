pub mod routes;
pub mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};

use crate::adaptor::AdaptorRegistry;

use self::state::AppState;

/// 启动 HTTP 服务器
pub async fn start_http_server(port: u16, pool: SqlitePool, registry: Arc<AdaptorRegistry>) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(routes::proxy_routes())
        .layer(cors)
        .with_state(AppState { pool, registry });

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("🚀 Proxy server listening on http://0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("绑定端口失败");

    axum::serve(listener, app)
        .await
        .expect("HTTP 服务器启动失败");
}
