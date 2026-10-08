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
    engine::{renderer::render_sticker, renderer::render_quote, theme::Theme, cards::{render_audio_card, render_file_card}},
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
        .route("/audio-card", post(audio_card_handler))
        .route("/file-card", post(file_card_handler))
        .with_state(state)
}

// ... existing sticker_handler and quote_handler ...

async fn sticker_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut username = String::new();
    let mut message = String::new();
    let mut theme_str = String::from("dark");
    let mut avatar_bytes: Option<Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, format!("Malformed form data: {}", e)))? {
        match field.name().unwrap_or_default() {
            "username" => { username = field.text().await.unwrap_or_default(); }
            "message" => { message = field.text().await.unwrap_or_default(); }
            "theme" => { theme_str = field.text().await.unwrap_or_default(); }
            "avatar_file" => {
                if let Ok(data) = field.bytes().await { if !data.is_empty() { avatar_bytes = Some(data); } }
            }
            _ => {}
        }
    }

    if username.is_empty() || message.is_empty() { return Err((StatusCode::BAD_REQUEST, "Missing required fields".into())); }
    
    let theme = if theme_str == "light" { Theme::light() } else { Theme::dark() };
    
    let img_data = tokio::task::spawn_blocking(move || {
        render_sticker(&state.font_mgr, &username, &message, None, &theme, avatar_bytes.as_deref())
    }).await.unwrap().ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Encoding failed".into()))?;

    Ok(([(header::CONTENT_TYPE, "image/webp")], img_data))
}

async fn quote_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut quote_text = String::new();
    let mut author_name = String::new();
    let mut quote_style = 0;
    let mut avatar_bytes: Option<Bytes> = None;
    let mut bg_bytes: Option<Bytes> = None;
    let mut output_format = String::from("jpg");

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, format!("Malformed form data: {}", e)))? {
        match field.name().unwrap_or_default() {
            "quote_text" => { quote_text = field.text().await.unwrap_or_default(); }
            "author_name" => { author_name = field.text().await.unwrap_or_default(); }
            "quote_style" => { if let Ok(val) = field.text().await.unwrap_or_default().parse::<i32>() { quote_style = val; } }
            "output_format" => { output_format = field.text().await.unwrap_or_default(); }
            "avatar_file" => { if let Ok(data) = field.bytes().await { if !data.is_empty() { avatar_bytes = Some(data); } } }
            "bg_file" => { if let Ok(data) = field.bytes().await { if !data.is_empty() { bg_bytes = Some(data); } } }
            _ => {}
        }
    }

    if quote_text.is_empty() || author_name.is_empty() { return Err((StatusCode::BAD_REQUEST, "Missing required fields".into())); }
    
    let is_png = output_format.to_lowercase() == "png";
    let img_data = tokio::task::spawn_blocking(move || {
        render_quote(&state.font_mgr, &quote_text, &author_name, &Theme::dark(), avatar_bytes.as_deref(), bg_bytes.as_deref(), quote_style)
    }).await.unwrap().ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Encoding failed".into()))?;

    Ok(([(header::CONTENT_TYPE, if is_png { "image/png" } else { "image/jpeg" })], img_data))
}

async fn audio_card_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut title = String::new();
    let mut performer = String::new();
    let mut duration = 0;
    let mut progress = 0.0;
    let mut thumb_bytes: Option<Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, format!("Malformed form data: {}", e)))? {
        match field.name().unwrap_or_default() {
            "title" => { title = field.text().await.unwrap_or_default(); }
            "performer" => { performer = field.text().await.unwrap_or_default(); }
            "duration" => { if let Ok(val) = field.text().await.unwrap_or_default().parse::<i32>() { duration = val; } }
            "progress" => { if let Ok(val) = field.text().await.unwrap_or_default().parse::<f32>() { progress = val; } }
            "thumb_file" => {
                if let Ok(data) = field.bytes().await {
                    if !data.is_empty() { thumb_bytes = Some(data); }
                }
            }
            _ => {}
        }
    }

    if title.is_empty() || performer.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Missing 'title' or 'performer'".to_string()));
    }

    let img_data = tokio::task::spawn_blocking(move || {
        render_audio_card(&state.font_mgr, &title, &performer, duration, progress, thumb_bytes.as_deref())
    })
    .await.unwrap().ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Encoding failed".to_string()))?;

    Ok(([(header::CONTENT_TYPE, "image/png")], img_data))
}

async fn file_card_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut file_name = String::new();
    let mut file_size_str = String::new();
    let mut file_ext = String::new();
    let mut thumb_bytes: Option<Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, format!("Malformed form data: {}", e)))? {
        match field.name().unwrap_or_default() {
            "file_name" => { file_name = field.text().await.unwrap_or_default(); }
            "file_size_str" => { file_size_str = field.text().await.unwrap_or_default(); }
            "file_ext" => { file_ext = field.text().await.unwrap_or_default(); }
            "thumb_file" => {
                if let Ok(data) = field.bytes().await {
                    if !data.is_empty() { thumb_bytes = Some(data); }
                }
            }
            _ => {}
        }
    }

    if file_name.is_empty() || file_size_str.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Missing 'file_name' or 'file_size_str'".to_string()));
    }

    let img_data = tokio::task::spawn_blocking(move || {
        render_file_card(&state.font_mgr, &file_name, &file_size_str, &file_ext, thumb_bytes.as_deref())
    })
    .await.unwrap().ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Encoding failed".to_string()))?;

    Ok(([(header::CONTENT_TYPE, "image/png")], img_data))
}
