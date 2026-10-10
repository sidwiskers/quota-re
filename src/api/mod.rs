use axum::{
    extract::{DefaultBodyLimit, Multipart},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use bytes::Bytes;
use std::sync::{Arc, OnceLock};
use tokio::sync::Semaphore;
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};

use crate::{
    engine::{
        cards::{render_audio_card, render_file_card},
        renderer::{render_quote, render_sticker},
        theme::Theme,
    },
    fonts::manager::FontManager,
};

type ApiError = (StatusCode, String);

const MAX_REQUEST_BYTES: usize = 12 * 1024 * 1024;
const MAX_STICKER_MESSAGE_CHARS: usize = 4_096;
const MAX_QUOTE_CHARS: usize = 4_000;
const MAX_CARD_TEXT_CHARS: usize = 256;

// Keep the number of expensive native render operations bounded. When both
// slots are busy, fail quickly rather than accumulating a blocking-task queue.
static RENDER_SEMAPHORE: OnceLock<Arc<Semaphore>> = OnceLock::new();

pub fn routes() -> Router {
    Router::new()
        .route("/sticker", post(sticker_handler))
        .route("/quote", post(quote_handler))
        .route("/audio-card", post(audio_card_handler))
        .route("/file-card", post(file_card_handler))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
        .layer(RequestBodyLimitLayer::new(MAX_REQUEST_BYTES))
        .layer(TraceLayer::new_for_http())
}

async fn render_blocking<F>(render: F) -> Result<Vec<u8>, ApiError>
where
    F: FnOnce() -> Option<Vec<u8>> + Send + 'static,
{
    let semaphore = RENDER_SEMAPHORE
        .get_or_init(|| Arc::new(Semaphore::new(2)))
        .clone();
    let permit = semaphore.try_acquire_owned().map_err(|_| {
        (
            StatusCode::TOO_MANY_REQUESTS,
            "Renderer is busy; retry the request shortly".to_string(),
        )
    })?;

    let output = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        render()
    })
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Rendering task failed".to_string(),
        )
    })?;

    output.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Image encoding or rendering failed".to_string(),
    ))
}

fn malformed_form(error: impl std::fmt::Display) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        format!("Malformed multipart form: {error}"),
    )
}

fn validate_text(
    value: &str,
    field_name: &str,
    max_chars: usize,
    required: bool,
) -> Result<(), ApiError> {
    if required && value.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Missing required field: {field_name}"),
        ));
    }
    if value.chars().count() > max_chars {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("{field_name} exceeds the {max_chars}-character limit"),
        ));
    }
    Ok(())
}

fn parse_quote_output_format(value: &str) -> Result<bool, ApiError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "png" => Ok(true),
        "jpg" | "jpeg" => Ok(false),
        _ => Err((
            StatusCode::BAD_REQUEST,
            "output_format must be png, jpg, or jpeg".to_string(),
        )),
    }
}

async fn sticker_handler(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let mut username = String::new();
    let mut message = String::new();
    let mut theme_str = String::from("dark");
    let mut reply_username = String::new();
    let mut reply_message = String::new();
    let mut avatar_bytes: Option<Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(malformed_form)? {
        let name = field.name().unwrap_or_default().to_owned();
        match name.as_str() {
            "username" => username = field.text().await.map_err(malformed_form)?,
            "message" => message = field.text().await.map_err(malformed_form)?,
            "theme" => theme_str = field.text().await.map_err(malformed_form)?,
            "reply_username" => {
                reply_username = field.text().await.map_err(malformed_form)?
            }
            "reply_message" => {
                reply_message = field.text().await.map_err(malformed_form)?
            }
            "avatar_file" => {
                let data = field.bytes().await.map_err(malformed_form)?;
                if !data.is_empty() {
                    avatar_bytes = Some(data);
                }
            }
            _ => {
                let _ = field.bytes().await.map_err(malformed_form)?;
            }
        }
    }

    validate_text(&username, "username", 128, true)?;
    validate_text(
        &message,
        "message",
        MAX_STICKER_MESSAGE_CHARS,
        true,
    )?;
    validate_text(&reply_username, "reply_username", 128, false)?;
    validate_text(&reply_message, "reply_message", 1_000, false)?;

    if reply_username.trim().is_empty() != reply_message.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "reply_username and reply_message must be supplied together".to_string(),
        ));
    }

    let theme_is_light = match theme_str.trim().to_ascii_lowercase().as_str() {
        "dark" => false,
        "light" => true,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "theme must be dark or light".to_string(),
            ))
        }
    };

    let img_data = render_blocking(move || {
        FontManager::with_thread_local(|font_mgr| {
            let theme = if theme_is_light {
                Theme::light()
            } else {
                Theme::dark()
            };
            let reply = if reply_username.trim().is_empty() {
                None
            } else {
                Some((reply_username.as_str(), reply_message.as_str()))
            };
            render_sticker(
                font_mgr,
                &username,
                &message,
                reply,
                &theme,
                avatar_bytes.as_deref(),
            )
        })
    })
    .await?;

    Ok(([(header::CONTENT_TYPE, "image/webp")], img_data))
}

async fn quote_handler(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let mut quote_text = String::new();
    let mut author_name = String::new();
    let mut quote_style = 0_i32;
    let mut avatar_bytes: Option<Bytes> = None;
    let mut bg_bytes: Option<Bytes> = None;
    let mut output_format = String::from("jpg");

    while let Some(field) = multipart.next_field().await.map_err(malformed_form)? {
        let name = field.name().unwrap_or_default().to_owned();
        match name.as_str() {
            "quote_text" => quote_text = field.text().await.map_err(malformed_form)?,
            "author_name" => author_name = field.text().await.map_err(malformed_form)?,
            "quote_style" => {
                let raw = field.text().await.map_err(malformed_form)?;
                quote_style = raw.trim().parse::<i32>().map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        "quote_style must be an integer".to_string(),
                    )
                })?;
            }
            "output_format" => {
                output_format = field.text().await.map_err(malformed_form)?
            }
            "avatar_file" => {
                let data = field.bytes().await.map_err(malformed_form)?;
                if !data.is_empty() {
                    avatar_bytes = Some(data);
                }
            }
            "bg_file" => {
                let data = field.bytes().await.map_err(malformed_form)?;
                if !data.is_empty() {
                    bg_bytes = Some(data);
                }
            }
            _ => {
                let _ = field.bytes().await.map_err(malformed_form)?;
            }
        }
    }

    validate_text(&quote_text, "quote_text", MAX_QUOTE_CHARS, true)?;
    validate_text(&author_name, "author_name", 256, true)?;
    if !matches!(quote_style, 0 | 2 | 3) {
        return Err((
            StatusCode::BAD_REQUEST,
            "quote_style must be 0, 2, or 3".to_string(),
        ));
    }
    let output_png = parse_quote_output_format(&output_format)?;

    let img_data = render_blocking(move || {
        FontManager::with_thread_local(|font_mgr| {
            render_quote(
                font_mgr,
                &quote_text,
                &author_name,
                &Theme::dark(),
                avatar_bytes.as_deref(),
                bg_bytes.as_deref(),
                quote_style,
                output_png,
            )
        })
    })
    .await?;

    let content_type = if output_png { "image/png" } else { "image/jpeg" };
    Ok(([(header::CONTENT_TYPE, content_type)], img_data))
}

async fn audio_card_handler(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let mut title = String::new();
    let mut performer = String::new();
    let mut duration = 0_i32;
    let mut progress = 0.0_f32;
    let mut thumb_bytes: Option<Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(malformed_form)? {
        let name = field.name().unwrap_or_default().to_owned();
        match name.as_str() {
            "title" => title = field.text().await.map_err(malformed_form)?,
            "performer" => performer = field.text().await.map_err(malformed_form)?,
            "duration" => {
                let raw = field.text().await.map_err(malformed_form)?;
                duration = raw.trim().parse::<i32>().map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        "duration must be an integer number of seconds".to_string(),
                    )
                })?;
            }
            "progress" => {
                let raw = field.text().await.map_err(malformed_form)?;
                progress = raw.trim().parse::<f32>().map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        "progress must be a number between 0 and 1".to_string(),
                    )
                })?;
            }
            "thumb_file" => {
                let data = field.bytes().await.map_err(malformed_form)?;
                if !data.is_empty() {
                    thumb_bytes = Some(data);
                }
            }
            _ => {
                let _ = field.bytes().await.map_err(malformed_form)?;
            }
        }
    }

    validate_text(&title, "title", MAX_CARD_TEXT_CHARS, true)?;
    validate_text(&performer, "performer", MAX_CARD_TEXT_CHARS, true)?;
    if !(0..=86_400).contains(&duration) {
        return Err((
            StatusCode::BAD_REQUEST,
            "duration must be between 0 and 86400 seconds".to_string(),
        ));
    }
    if !progress.is_finite() {
        return Err((
            StatusCode::BAD_REQUEST,
            "progress must be a finite number between 0 and 1".to_string(),
        ));
    }
    progress = progress.clamp(0.0, 1.0);

    let img_data = render_blocking(move || {
        FontManager::with_thread_local(|font_mgr| {
            render_audio_card(
                font_mgr,
                &title,
                &performer,
                duration,
                progress,
                thumb_bytes.as_deref(),
            )
        })
    })
    .await?;

    Ok(([(header::CONTENT_TYPE, "image/png")], img_data))
}

async fn file_card_handler(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let mut file_name = String::new();
    let mut file_size_str = String::new();
    let mut file_ext = String::new();
    let mut thumb_bytes: Option<Bytes> = None;

    while let Some(field) = multipart.next_field().await.map_err(malformed_form)? {
        let name = field.name().unwrap_or_default().to_owned();
        match name.as_str() {
            "file_name" => file_name = field.text().await.map_err(malformed_form)?,
            "file_size_str" => {
                file_size_str = field.text().await.map_err(malformed_form)?
            }
            "file_ext" => file_ext = field.text().await.map_err(malformed_form)?,
            "thumb_file" => {
                let data = field.bytes().await.map_err(malformed_form)?;
                if !data.is_empty() {
                    thumb_bytes = Some(data);
                }
            }
            _ => {
                let _ = field.bytes().await.map_err(malformed_form)?;
            }
        }
    }

    validate_text(&file_name, "file_name", 512, true)?;
    validate_text(&file_size_str, "file_size_str", 64, true)?;
    validate_text(&file_ext, "file_ext", 16, false)?;

    let img_data = render_blocking(move || {
        FontManager::with_thread_local(|font_mgr| {
            render_file_card(
                font_mgr,
                &file_name,
                &file_size_str,
                &file_ext,
                thumb_bytes.as_deref(),
            )
        })
    })
    .await?;

    Ok(([(header::CONTENT_TYPE, "image/png")], img_data))
}

#[cfg(test)]
mod tests {
    use super::{parse_quote_output_format, validate_text};
    use axum::http::StatusCode;

    #[test]
    fn quote_format_parser_accepts_only_supported_encodings() {
        assert_eq!(parse_quote_output_format("png").unwrap(), true);
        assert_eq!(parse_quote_output_format(" JPG ").unwrap(), false);
        assert_eq!(parse_quote_output_format("jpeg").unwrap(), false);
        assert_eq!(
            parse_quote_output_format("webp").unwrap_err().0,
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn text_limits_are_enforced_and_required_fields_are_checked() {
        assert!(validate_text("quote", "quote_text", 5, true).is_ok());
        assert_eq!(
            validate_text("quotes", "quote_text", 5, true).unwrap_err().0,
            StatusCode::PAYLOAD_TOO_LARGE
        );
        assert_eq!(
            validate_text("  ", "quote_text", 5, true).unwrap_err().0,
            StatusCode::BAD_REQUEST
        );
    }
}
