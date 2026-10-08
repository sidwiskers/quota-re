use crate::engine::theme::Theme;
use crate::fonts::manager::FontManager;
use skia_safe::{
    textlayout::Paragraph, BlendMode, Color, Data, EncodedImageFormat, Image, Paint, Path, Point, RRect, Rect, Shader, Surface, TileMode
};

pub fn render_quote(
    font_mgr: &FontManager,
    quote_text: &str,
    author_name: &str,
    theme: &Theme,
    avatar_bytes: Option<&[u8]>,
    bg_bytes: Option<&[u8]>,
    _quote_style: i32, // To handle classic quote layouts later (0=horiz, 2=vertical, 3=cloud)
) -> Option<Vec<u8>> {
    let canvas_w = 1200.0;
    let padding = 50.0;
    let avatar_size = 400.0;

    let max_text_w = canvas_w - (padding * 2.0) - avatar_size - padding; // Assuming horiz-left style (style=0)

    let qfs = 36.0;
    let afs = 28.0;

    // 1. Measure Quote Text
    let q_paragraph = font_mgr.build_paragraph(
        quote_text,
        qfs,
        "Roboto",
        max_text_w,
        theme.text, // Normally user configures this, fallback to theme
        None,
    );
    let q_h = q_paragraph.height();

    // 2. Measure Author Text
    let author_text = format!("— {}", author_name.trim());
    let a_paragraph = font_mgr.build_paragraph(
        &author_text,
        afs,
        "Roboto",
        max_text_w,
        Color::from_rgb(180, 180, 180), // cfg.author_color
        Some(1),
    );
    let a_h = a_paragraph.height();

    let content_h = q_h + a_h;
    
    // Total canvas height calculation
    let canvas_h = 600.0_f32.max(content_h + (padding * 2.0)).max(avatar_size + (padding * 2.0));

    let mut surface = Surface::new_raster_n32_premul((canvas_w as i32, canvas_h as i32))?;
    let canvas = surface.canvas();

    // 3. Draw Background
    if let Some(bytes) = bg_bytes {
        if let Some(bg_image) = Image::from_encoded(Data::new_copy(bytes)) {
            // Apply a simple blur for bg_style=1
            // In Skia, creating an image filter for blur and painting it is easy:
            if let Some(blur_filter) = skia_safe::image_filters::blur(
                (60.0, 60.0), // cfg.bg_blur_radius
                TileMode::Clamp,
                None,
                None,
            ) {
                let mut bg_paint = Paint::default();
                bg_paint.set_image_filter(blur_filter);
                
                // Crop and Scale (Center Crop) logic
                let src_w = bg_image.width() as f32;
                let src_h = bg_image.height() as f32;
                let scale = (canvas_w / src_w).max(canvas_h / src_h);
                let nw = src_w * scale;
                let nh = src_h * scale;
                let left = (nw - canvas_w) / 2.0;
                let top = (nh - canvas_h) / 2.0;

                canvas.save();
                canvas.translate((-left, -top));
                canvas.scale((scale, scale));
                canvas.draw_image(bg_image, (0, 0), Some(&bg_paint));
                canvas.restore();
            }
        }
    } else {
        // Fallback random solid color background
        let mut bg_paint = Paint::default();
        bg_paint.set_color(Color::from_rgb(30, 30, 30));
        canvas.draw_rect(Rect::from_xywh(0.0, 0.0, canvas_w, canvas_h), &bg_paint);
    }

    // Apply Dimming Overlay
    let dimming_alpha = (70.0 / 100.0 * 255.0) as u8; // cfg.bg_dimming
    let mut overlay_paint = Paint::default();
    overlay_paint.set_color(Color::from_argb(dimming_alpha, 0, 0, 0));
    canvas.draw_rect(Rect::from_xywh(0.0, 0.0, canvas_w, canvas_h), &overlay_paint);

    // 4. Draw Avatar (Square/Rounded)
    let ava_x = canvas_w - padding - avatar_size;
    // Vertically center avatar
    let ava_y = (canvas_h - avatar_size) / 2.0;
    
    let inner_rect = Rect::from_xywh(ava_x, ava_y, avatar_size, avatar_size);
    let ava_radius = avatar_size / 2.0; // cfg.avatar_rounding = 100

    if let Some(bytes) = avatar_bytes {
        if let Some(image) = Image::from_encoded(Data::new_copy(bytes)) {
            canvas.save();
            let clip_rrect = RRect::new_oval(inner_rect);
            canvas.clip_rrect(clip_rrect, None, true);
            
            let mut img_paint = Paint::default();
            img_paint.set_anti_alias(true);
            
            // Draw image filling the avatar circle
            canvas.draw_image_rect(
                image,
                None,
                inner_rect,
                &img_paint,
            );
            canvas.restore();
        }
    }

    // 5. Draw Text Block
    // Vertically center text
    let mut cur_y = (canvas_h - content_h) / 2.0;
    let tx0 = padding; // Left aligned

    q_paragraph.paint(canvas, (tx0, cur_y));
    cur_y += q_h + 10.0; // cfg.line_spacing

    a_paragraph.paint(canvas, (tx0, cur_y));

    // 6. Encode to JPEG
    let snapshot = surface.image_snapshot();
    let data = snapshot.encode_to_data_with_quality(EncodedImageFormat::JPEG, 92)?;
    
    Some(data.as_bytes().to_vec())
}
