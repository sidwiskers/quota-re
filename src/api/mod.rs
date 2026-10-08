use axum::{
    extract::Multipart,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use bytes::Bytes;
use std::sync::Arc;

use crate::{
    engine::{renderer::render_sticker, theme::Theme},
    fonts::manager::FontManager,
};

#[derive(Clone)]
pub struct AppState {
    pub font_mgr: Arc<FontManager>,
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/sticker", post(sticker_handler))
        .with_state(state)
}

async fn sticker_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut username = String::new();
    let mut message = String::new();
    let mut theme_str = String::from("dark");
    let mut avatar_bytes: Option<Bytes> = None;

    // Parse the multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, format!("Malformed form data: {}", e))
    })? {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "username" => {
                username = field.text().await.unwrap_or_default();
            }
            "message" => {
                message = field.text().await.unwrap_or_default();
            }
            "theme" => {
                theme_str = field.text().await.unwrap_or_default();
            }
            "avatar_file" => {
                if let Ok(data) = field.bytes().await {
                    if !data.is_empty() {
                        avatar_bytes = Some(data);
                    }
                }
            }
            _ => {} // Ignore unknown fields like bg_color or avatar_url for now
        }
    }

    if username.is_empty() || message.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Missing 'username' or 'message'".to_string(),
        ));
    }

    let theme = if theme_str == "light" { Theme::light() } else { Theme::dark() };

    // Offload the heavy Skia rendering to a blocking thread to avoid locking the Tokio runtime
    let img_data = tokio::task::spawn_blocking(move || {
        render_sticker(
            &state.font_mgr,
            &username,
            &message,
            None,
            &theme,
            avatar_bytes.as_deref(),
        )
    })
    .await
    .map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Thread panicked: {}", e))
    })?
    .ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode WebP".to_string())
    })?;

    Ok((
        [(header::CONTENT_TYPE, "image/webp")],
        img_data,
    ))
}
