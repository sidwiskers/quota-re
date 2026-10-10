use crate::engine::{
    images::{decode_safe_image, draw_cropped_image},
    layout::BubbleLayout,
    theme::Theme,
};
use crate::fonts::manager::FontManager;
use skia_safe::{
    textlayout::TextAlign, Canvas, Color, EncodedImageFormat, Paint, Rect, RRect, Surface,
    TileMode,
};

const STICKER_SIDE: i32 = 512;
const MAX_QUOTE_HEIGHT: f32 = 3_000.0;

pub fn render_sticker(
    font_mgr: &FontManager,
    username: &str,
    message: &str,
    reply: Option<(&str, &str)>,
    theme: &Theme,
    avatar_bytes: Option<&[u8]>,
) -> Option<Vec<u8>> {
    if message.trim().is_empty() {
        return None;
    }

    let avatar = avatar_bytes.and_then(decode_safe_image);
    let has_avatar = avatar.is_some();
    let mut layout =
        BubbleLayout::calculate(font_mgr, username, message, reply, theme, 1.0, has_avatar);
    let available_height = 464.0;
    if layout.bubble_height > available_height {
        let scale = (available_height / layout.bubble_height).clamp(0.25, 1.0);
        layout =
            BubbleLayout::calculate(font_mgr, username, message, reply, theme, scale, has_avatar);
    }

    let mut surface = Surface::new_raster_n32_premul((STICKER_SIDE, STICKER_SIDE))?;
    let canvas = surface.canvas();
    canvas.clear(Color::from_argb(0, 0, 0, 0));

    let left = (STICKER_SIDE as f32 - layout.bubble_width) / 2.0;
    let top = (STICKER_SIDE as f32 - layout.bubble_height) / 2.0;
    let scale = layout.render_scale;
    let horizontal_padding = 18.0 * scale;
    let vertical_padding = 13.0 * scale;
    let inner_width = (layout.bubble_width - horizontal_padding * 2.0).max(1.0);

    let bubble_rect = Rect::from_xywh(left, top, layout.bubble_width, layout.bubble_height);
    let mut bubble_paint = Paint::default();
    bubble_paint.set_anti_alias(true);
    bubble_paint.set_color(theme.bubble_bg);
    canvas.draw_rrect(
        RRect::new_rect_xy(bubble_rect, 24.0 * scale, 24.0 * scale),
        &bubble_paint,
    );
    canvas.save();
    canvas.clip_rrect(
        RRect::new_rect_xy(bubble_rect, 24.0 * scale, 24.0 * scale),
        None,
        true,
    );

    let name_height = layout
        .name_paragraph
        .height()
        .max(29.0 * scale)
        .max(if has_avatar { 36.0 * scale } else { 0.0 });
    let name_y = top + vertical_padding;
    let mut name_x = left + horizontal_padding;
    if has_avatar {
        let avatar_size = 36.0 * scale;
        let avatar_y = name_y + ((name_height - avatar_size) / 2.0).max(0.0);
        let avatar_rect = Rect::from_xywh(name_x, avatar_y, avatar_size, avatar_size);
        draw_avatar_image(canvas, avatar.as_ref(), avatar_rect, theme.avatar_border);
        name_x += 46.0 * scale;
    }
    layout.name_paragraph.paint(canvas, (name_x, name_y));

    let mut content_y = name_y + name_height + 6.0 * scale;

    if layout.has_reply {
        let reply_height = layout.reply_block_height;
        let reply_rect = Rect::from_xywh(
            left + horizontal_padding,
            content_y,
            inner_width,
            reply_height,
        );
        let mut reply_bg = Paint::default();
        reply_bg.set_anti_alias(true);
        reply_bg.set_color(theme.reply_bg);
        canvas.draw_rrect(
            RRect::new_rect_xy(reply_rect, 8.0 * scale, 8.0 * scale),
            &reply_bg,
        );

        let bar_rect = Rect::from_xywh(
            reply_rect.left() + 7.0 * scale,
            reply_rect.top() + 7.0 * scale,
            3.0 * scale,
            (reply_height - 14.0 * scale).max(1.0),
        );
        let mut bar_paint = Paint::default();
        bar_paint.set_anti_alias(true);
        bar_paint.set_color(theme.reply_bar);
        canvas.draw_rrect(
            RRect::new_rect_xy(bar_rect, 1.5 * scale, 1.5 * scale),
            &bar_paint,
        );

        if let Some(paragraph) = layout.reply_name_paragraph.as_ref() {
            paragraph.paint(
                canvas,
                (reply_rect.left() + 16.0 * scale, reply_rect.top() + 5.0 * scale),
            );
        }
        if let Some(paragraph) = layout.reply_msg_paragraph.as_ref() {
            let name_height = layout
                .reply_name_paragraph
                .as_ref()
                .map(|p| p.height().max(17.0 * scale))
                .unwrap_or(17.0 * scale);
            paragraph.paint(
                canvas,
                (
                    reply_rect.left() + 16.0 * scale,
                    reply_rect.top() + 7.0 * scale + name_height,
                ),
            );
        }

        content_y += reply_height + 6.0 * scale;
    }

    layout
        .msg_paragraph
        .paint(canvas, (left + horizontal_padding, content_y));
    canvas.restore();

    let snapshot = surface.image_snapshot();
    let data = snapshot.encode_to_data_with_quality(EncodedImageFormat::WEBP, 90)?;
    Some(data.as_bytes().to_vec())
}

pub fn render_quote(
    font_mgr: &FontManager,
    quote_text: &str,
    author_name: &str,
    theme: &Theme,
    avatar_bytes: Option<&[u8]>,
    bg_bytes: Option<&[u8]>,
    quote_style: i32,
    output_png: bool,
) -> Option<Vec<u8>> {
    if quote_text.trim().is_empty() || !matches!(quote_style, 0 | 2 | 3) {
        return None;
    }

    let avatar_image = avatar_bytes.and_then(decode_safe_image);
    let has_avatar = avatar_image.is_some();
    let canvas_width = 1_200.0;
    let padding = 50.0;
    let horizontal = quote_style == 0;
    let avatar_size = match quote_style {
        0 if has_avatar => 400.0,
        2 if has_avatar => 300.0,
        3 if has_avatar => 128.0,
        _ => 0.0,
    };
    let text_width = if horizontal {
        canvas_width - padding * 2.0 - (if has_avatar { avatar_size + padding } else { 0.0 })
    } else if quote_style == 2 {
        canvas_width - 140.0
    } else {
        canvas_width - 180.0
    };

    let quote_font_size = if quote_style == 3 { 42.0 } else { 36.0 };
    let author_font_size = 28.0;
    let alignment = if horizontal {
        TextAlign::Start
    } else {
        TextAlign::Center
    };

    let quote_paragraph = font_mgr.build_paragraph_aligned(
        quote_text,
        quote_font_size,
        "Roboto",
        text_width,
        theme.text,
        Some(if horizontal { 60 } else { 45 }),
        alignment,
    );
    let quote_height = quote_paragraph.height();

    let author_text = format!("— {}", author_name.trim());
    let author_paragraph = font_mgr.build_paragraph_aligned(
        &author_text,
        author_font_size,
        "Roboto",
        text_width,
        Color::from_rgb(190, 190, 190),
        Some(1),
        alignment,
    );
    let author_height = author_paragraph.height();
    let content_height = quote_height + 16.0 + author_height;

    let canvas_height = match quote_style {
        0 => (content_height + padding * 2.0).max(600.0).min(MAX_QUOTE_HEIGHT),
        2 => (content_height + avatar_size + 170.0).max(900.0).min(MAX_QUOTE_HEIGHT),
        _ => (content_height + 320.0).max(700.0).min(MAX_QUOTE_HEIGHT),
    };

    let mut surface = Surface::new_raster_n32_premul((
        canvas_width as i32,
        canvas_height.ceil() as i32,
    ))?;
    let canvas = surface.canvas();
    draw_background(canvas, bg_bytes, canvas_width, canvas_height);

    match quote_style {
        0 => {
            let avatar_rect = Rect::from_xywh(
                canvas_width - padding - avatar_size,
                (canvas_height - avatar_size) / 2.0,
                avatar_size,
                avatar_size,
            );
            if let Some(image) = avatar_image.as_ref() {
                draw_avatar_image(canvas, image, avatar_rect);
            }

            let text_x = padding;
            let content_y = ((canvas_height - content_height) / 2.0).max(padding);
            let text_rect = Rect::from_xywh(text_x, content_y, text_width, content_height);
            canvas.save();
            canvas.clip_rect(text_rect, None, true);
            quote_paragraph.paint(canvas, (text_x, content_y));
            author_paragraph.paint(canvas, (text_x, content_y + quote_height + 16.0));
            canvas.restore();
        }
        2 => {
            let avatar_rect = Rect::from_xywh(
                (canvas_width - avatar_size) / 2.0,
                55.0,
                avatar_size,
                avatar_size,
            );
            if let Some(image) = avatar_image.as_ref() {
                draw_avatar_image(canvas, image, avatar_rect);
            }

            let text_x = (canvas_width - text_width) / 2.0;
            let text_y = avatar_rect.bottom() + 45.0;
            let text_rect = Rect::from_xywh(text_x, text_y, text_width, content_height);
            canvas.save();
            canvas.clip_rect(text_rect, None, true);
            quote_paragraph.paint(canvas, (text_x, text_y));
            author_paragraph.paint(canvas, (text_x, text_y + quote_height + 16.0));
            canvas.restore();
        }
        3 => {
            let card_rect = Rect::from_xywh(
                55.0,
                55.0,
                canvas_width - 110.0,
                canvas_height - 110.0,
            );
            let mut card_paint = Paint::default();
            card_paint.set_anti_alias(true);
            card_paint.set_color(theme.bubble_bg);
            canvas.draw_rrect(
                RRect::new_rect_xy(card_rect, 34.0, 34.0),
                &card_paint,
            );

            let text_x = (canvas_width - text_width) / 2.0;
            let text_y = if has_avatar {
                let avatar_rect = Rect::from_xywh(
                    (canvas_width - avatar_size) / 2.0,
                    card_rect.top() + 28.0,
                    avatar_size,
                    avatar_size,
                );
                if let Some(image) = avatar_image.as_ref() {
                draw_avatar_image(canvas, image, avatar_rect);
            }
                avatar_rect.bottom() + 24.0
            } else {
                card_rect.top() + 48.0
            };
            let text_rect = Rect::from_xywh(text_x, text_y, text_width, content_height);
            canvas.save();
            canvas.clip_rect(text_rect, None, true);
            quote_paragraph.paint(canvas, (text_x, text_y));
            author_paragraph.paint(canvas, (text_x, text_y + quote_height + 16.0));
            canvas.restore();
        }
        _ => return None,
    }

    let snapshot = surface.image_snapshot();
    let format = if output_png {
        EncodedImageFormat::PNG
    } else {
        EncodedImageFormat::JPEG
    };
    let quality = if output_png { 100 } else { 92 };
    let data = snapshot.encode_to_data_with_quality(format, quality)?;
    Some(data.as_bytes().to_vec())
}

fn draw_background(canvas: &Canvas, bytes: Option<&[u8]>, width: f32, height: f32) {
    let bounds = Rect::from_xywh(0.0, 0.0, width, height);
    let mut fallback = Paint::default();
    fallback.set_color(Color::from_rgb(24, 26, 36));
    canvas.draw_rect(bounds, &fallback);

    if let Some(image) = bytes.and_then(decode_safe_image) {
        let mut paint = Paint::default();
        if let Some(filter) = skia_safe::image_filters::blur(
            (18.0, 18.0),
            TileMode::Clamp,
            None,
            None,
        ) {
            paint.set_image_filter(filter);
        }
        draw_cropped_image(canvas, &image, bounds, &paint);
    }

    let mut dimming = Paint::default();
    dimming.set_color(Color::from_argb(92, 0, 0, 0));
    canvas.draw_rect(bounds, &dimming);
}

fn draw_avatar_image(canvas: &Canvas, image: &skia_safe::Image, destination: Rect) {
    canvas.save();
    canvas.clip_rrect(RRect::new_oval(destination), None, true);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    draw_cropped_image(canvas, image, destination, &paint);
    canvas.restore();
}

#[cfg(test)]
mod tests {
    use super::{render_quote, render_sticker};
    use crate::{engine::theme::Theme, fonts::manager::FontManager};

    #[test]
    fn sticker_output_is_webp() {
        let output = FontManager::with_thread_local(|font_mgr| {
            render_sticker(
                font_mgr,
                "Siddhartha",
                "Rendering test",
                None,
                &Theme::dark(),
                None,
            )
        })
        .expect("sticker rendering should succeed");

        assert!(output.len() > 12);
        assert_eq!(&output[0..4], b"RIFF");
        assert_eq!(&output[8..12], b"WEBP");
    }

    #[test]
    fn sticker_renders_avatar_and_reply() {
        use skia_safe::{Color, EncodedImageFormat, Surface};

        let output = FontManager::with_thread_local(|font_mgr| {
            let mut avatar_surface =
                Surface::new_raster_n32_premul((16, 16)).expect("test avatar surface");
            avatar_surface.canvas().clear(Color::from_rgb(60, 120, 200));
            let avatar_data = avatar_surface
                .image_snapshot()
                .encode_to_data_with_quality(EncodedImageFormat::PNG, 100)
                .expect("encode test avatar");

            render_sticker(
                font_mgr,
                "Siddhartha",
                "A message with an avatar and a reply",
                Some(("Original author", "The original message")),
                &Theme::dark(),
                Some(avatar_data.as_bytes()),
            )
        })
        .expect("sticker with avatar and reply should render");

        assert_eq!(&output[0..4], b"RIFF");
        assert_eq!(&output[8..12], b"WEBP");
    }

    #[test]
    fn all_quote_layouts_render() {
        FontManager::with_thread_local(|font_mgr| {
            for style in [0, 2, 3] {
                let output = render_quote(
                    font_mgr,
                    "A quote with multiple words",
                    "Author",
                    &Theme::dark(),
                    None,
                    None,
                    style,
                    false,
                )
                .expect("supported quote layout should render");
                assert!(output.starts_with(&[0xFF, 0xD8, 0xFF]));
            }
        });
    }

    #[test]
    fn quote_encoding_matches_requested_format() {
        let (png, jpeg) = FontManager::with_thread_local(|font_mgr| {
            let png = render_quote(
                font_mgr,
                "A test quote",
                "Author",
                &Theme::dark(),
                None,
                None,
                0,
                true,
            )
            .expect("PNG quote rendering should succeed");
            let jpeg = render_quote(
                font_mgr,
                "A test quote",
                "Author",
                &Theme::dark(),
                None,
                None,
                0,
                false,
            )
            .expect("JPEG quote rendering should succeed");
            (png, jpeg)
        });

        assert!(png.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]));
        assert!(jpeg.starts_with(&[0xFF, 0xD8, 0xFF]));
    }
}
