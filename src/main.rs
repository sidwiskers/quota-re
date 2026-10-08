mod api;
mod engine;
mod fonts;

use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,quota_re=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Initializing Quota-Re Engine (Skia)...");

    // Initialize the font manager and skia engine context here later
    
    // Setup router
    let app = Router::new()
        .route("/health", get(|| async { axum::Json(serde_json::json!({ "status": "ok" })) }))
        .nest("/", api::routes()); // Nesting at root to match original /sticker, /quote

    let port = std::env::var("PORT").unwrap_or_else(|_| "5000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Server listening on {}", addr);
    
    axum::serve(listener, app).await.unwrap();
}
