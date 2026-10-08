mod api;
mod engine;
mod fonts;

use api::AppState;
use axum::{routing::get, Router};
use fonts::manager::FontManager;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,quota_re=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Initializing Quota-Re Engine (Skia)...");

    // Initialize FontManager once and share it across workers
    let font_mgr = Arc::new(FontManager::new());
    let state = AppState { font_mgr };
    
    // Setup router
    let app = Router::new()
        .route("/health", get(|| async { axum::Json(serde_json::json!({ "status": "ok" })) }))
        .nest("/", api::routes(state));

    let port = std::env::var("PORT").unwrap_or_else(|_| "5000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Server listening on {}", addr);
    
    axum::serve(listener, app).await.unwrap();
}
