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
    engine::{renderer::render_sticker, renderer::render_quote, theme::Theme},
    fonts::manager::FontManager,
};

#[derive(Clone)]
pub struct AppState {
    pub font_mgr: Arc<FontManager>,
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/sticker", post(sticker_handler))
        .route("/quote", post(quote_handler))
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

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, format!("Malformed form data: {}", e))
    })? {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "username" => { username = field.text().await.unwrap_or_default(); }
            "message" => { message = field.text().await.unwrap_or_default(); }
            "theme" => { theme_str = field.text().await.unwrap_or_default(); }
            "avatar_file" => {
                if let Ok(data) = field.bytes().await {
                    if !data.is_empty() { avatar_bytes = Some(data); }
                }
            }
            _ => {}
        }
    }

    if username.is_empty() || message.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Missing 'username' or 'message'".to_string()));
    }

    let theme = if theme_str == "light" { Theme::light() } else { Theme::dark() };

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
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Thread panicked: {}", e)))?
    .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode WebP".to_string()))?;

    Ok(([(header::CONTENT_TYPE, "image/webp")], img_data))
}

async fn quote_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut quote_text = String::new();
    let mut author_name = String::new();
    let mut quote_style = 0; // 0=horiz-left
    let mut avatar_bytes: Option<Bytes> = None;
    let mut bg_bytes: Option<Bytes> = None;
    let mut output_format = String::from("jpg");

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, format!("Malformed form data: {}", e))
    })? {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "quote_text" => { quote_text = field.text().await.unwrap_or_default(); }
            "author_name" => { author_name = field.text().await.unwrap_or_default(); }
            "quote_style" => {
                if let Ok(val) = field.text().await.unwrap_or_default().parse::<i32>() {
                    quote_style = val;
                }
            }
            "output_format" => { output_format = field.text().await.unwrap_or_default(); }
            "avatar_file" => {
                if let Ok(data) = field.bytes().await {
                    if !data.is_empty() { avatar_bytes = Some(data); }
                }
            }
            "bg_file" => {
                if let Ok(data) = field.bytes().await {
                    if !data.is_empty() { bg_bytes = Some(data); }
                }
            }
            _ => {}
        }
    }

    if quote_text.is_empty() || author_name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Missing 'quote_text' or 'author_name'".to_string()));
    }

    let is_png = output_format.to_lowercase() == "png";

    let img_data = tokio::task::spawn_blocking(move || {
        // Theme is irrelevant for classic quote background, it uses custom settings,
        // but we pass dark for default text colors.
        render_quote(
            &state.font_mgr,
            &quote_text,
            &author_name,
            &Theme::dark(),
            avatar_bytes.as_deref(),
            bg_bytes.as_deref(),
            quote_style,
        )
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Thread panicked: {}", e)))?
    .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode image".to_string()))?;

    let content_type = if is_png { "image/png" } else { "image/jpeg" };
    Ok(([(header::CONTENT_TYPE, content_type)], img_data))
}
