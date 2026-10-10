mod api;
mod engine;
mod fonts;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,quota_re=debug")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Initializing Quota-Re Engine (Skia)...");

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "5000".to_string())
        .parse::<u16>()?;
    let address = SocketAddr::from(([0, 0, 0, 0], port));

    let app = Router::new()
        .route(
            "/health",
            get(|| async { axum::Json(serde_json::json!({ "status": "ok" })) }),
        )
        .merge(api::routes());

    let listener = TcpListener::bind(address).await?;
    tracing::info!("Server listening on {}", address);

    axum::serve(listener, app).await?;
    Ok(())
}
