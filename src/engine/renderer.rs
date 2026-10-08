use crate::engine::layout::BubbleLayout;
use crate::engine::theme::Theme;
use crate::fonts::manager::FontManager;
use skia_safe::{
    BlendMode, Color, Data, EncodedImageFormat, Image, Paint, Path, Point, RRect, Rect, Shader,
    Surface, TileMode,
};

pub fn render_sticker(
    font_mgr: &FontManager,
    username: &str,
    message: &str,
    reply: Option<(&str, &str)>,
    theme: &Theme,
    avatar_bytes: Option<&[u8]>,
) -> Option<Vec<u8>> {
    let render_scale = 3.0;
    let target = 512.0;

    let layout = BubbleLayout::calculate(font_mgr, username, message, reply, theme, render_scale);

    let ava_sz = 52.0 * render_scale;
    let ava_gap = 8.0 * render_scale;
    let ava_total = ava_sz + (4.0 * render_scale); // Avatar border
    
    let bubble_x = ava_total + ava_gap;
    let canvas_w = bubble_x + layout.bubble_width;
    let bottom_pad = 56.0 * render_scale;
    let canvas_h = layout.bubble_height + bottom_pad;

    // We calculate the scale matrix to draw directly to a 512px canvas, achieving 
    // perfect vector downsampling without massive memory allocation.
    let target_scale = target / canvas_w.max(canvas_h);
    
    let final_w = (canvas_w * target_scale).round() as i32;
    let final_h = (canvas_h * target_scale).round() as i32;

    let mut surface = Surface::new_raster_n32_premul((final_w, final_h))?;
    let canvas = surface.canvas();

    // Scale canvas so we can draw using high-res coordinates
    canvas.scale((target_scale, target_scale));

    // 1. Draw the Bubble
    let mut bubble_paint = Paint::default();
    bubble_paint.set_anti_alias(true);
    bubble_paint.set_color(theme.bubble_bg);

    let bubble_rect = Rect::from_xywh(bubble_x, 0.0, layout.bubble_width, layout.bubble_height);
    let bubble_rrect = RRect::new_rect_xy(bubble_rect, 22.0 * render_scale, 22.0 * render_scale);
    canvas.draw_rrect(bubble_rrect, &bubble_paint);

    // 2. Draw the Tail
    let mut path = Path::new();
    let tw = 20.0 * render_scale;
    let th = 22.0 * render_scale;
    let bx = bubble_x;
    let by = layout.bubble_height;
    
    path.move_to((bx + 2.0, by - th));
    path.line_to((bx + 2.0, by));
    path.line_to((bx - tw + 2.0, by - 3.0 * render_scale));
    path.close();
    canvas.draw_path(&path, &bubble_paint);

    // 3. Gradient Shine
    let shine_colors = [
        Color::from_argb(18, 255, 255, 255),
        Color::from_argb(0, 255, 255, 255),
    ];
    if let Some(shader) = Shader::linear_gradient(
        (bubble_x, 0.0),
        (bubble_x, layout.bubble_height),
        &shine_colors,
        None,
        TileMode::Clamp,
        None,
        None,
    ) {
        let mut shine_paint = Paint::default();
        shine_paint.set_anti_alias(true);
        shine_paint.set_shader(shader);
        shine_paint.set_blend_mode(BlendMode::Screen);
        canvas.draw_rrect(bubble_rrect, &shine_paint);
    }

    // 4. Draw Text & Reply Block
    let iph = 18.0 * render_scale;
    let mut cur_y = 13.0 * render_scale; // ipv
    let tx0 = bubble_x + iph;

    layout.name_paragraph.paint(canvas, (tx0, cur_y));
    cur_y += layout.name_paragraph.height() + (4.0 * render_scale);

    if layout.has_reply {
        let rp_w = layout.bubble_width - (iph * 2.0);
        let rp_h = layout.reply_block_height - (4.0 * render_scale);
        let rp_y0 = cur_y + (2.0 * render_scale);

        let mut rp_bg_paint = Paint::default();
        rp_bg_paint.set_color(theme.reply_bg);
        rp_bg_paint.set_anti_alias(true);
        canvas.draw_rrect(
            RRect::new_rect_xy(Rect::from_xywh(tx0, rp_y0, rp_w, rp_h), 7.0 * render_scale, 7.0 * render_scale),
            &rp_bg_paint,
        );

        let mut rp_bar_paint = Paint::default();
        rp_bar_paint.set_color(theme.reply_bar);
        rp_bar_paint.set_anti_alias(true);
        canvas.draw_rrect(
            RRect::new_rect_xy(
                Rect::from_xywh(tx0 + 3.0 * render_scale, rp_y0 + 5.0 * render_scale, 3.0 * render_scale, rp_h - 10.0 * render_scale),
                2.0 * render_scale, 2.0 * render_scale,
            ),
            &rp_bar_paint,
        );

        let text_left = tx0 + 12.0 * render_scale;
        if let Some(p) = &layout.reply_name_paragraph {
            p.paint(canvas, (text_left, rp_y0 + 6.0 * render_scale));
        }
        if let Some(p) = &layout.reply_msg_paragraph {
            p.paint(canvas, (text_left, rp_y0 + (10.0 + 17.0) * render_scale));
        }
        cur_y += layout.reply_block_height;
    }

    layout.msg_paragraph.paint(canvas, (tx0, cur_y));

    // 5. Draw Avatar
    let ava_x = 0.0;
    let ava_y = 2.0 * render_scale;
    
    let mut border_paint = Paint::default();
    border_paint.set_color(theme.avatar_border);
    border_paint.set_anti_alias(true);

    let center = Point::new(ava_x + ava_total / 2.0, ava_y + ava_total / 2.0);
    let radius = ava_total / 2.0;
    canvas.draw_circle(center, radius, &border_paint);

    let ava_inner_radius = ava_sz / 2.0;
    let inner_rect = Rect::from_xywh(
        center.x - ava_inner_radius,
        center.y - ava_inner_radius,
        ava_sz,
        ava_sz,
    );

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
    } else {
        // Fallback: draw initials in circle
        let mut placeholder_paint = Paint::default();
        placeholder_paint.set_color(Color::from_rgb(150, 206, 180)); // Deterministic color logic goes here in future
        placeholder_paint.set_anti_alias(true);
        canvas.draw_circle(center, ava_inner_radius, &placeholder_paint);
        
        let initial = username.chars().next().unwrap_or('?').to_uppercase().to_string();
        let mut initial_paragraph = font_mgr.build_paragraph(
            &initial,
            ava_inner_radius * 1.2,
            "Roboto",
            ava_sz,
            Color::WHITE,
            Some(1)
        );
        
        let ix = center.x - (initial_paragraph.max_intrinsic_width() / 2.0);
        let iy = center.y - (initial_paragraph.height() / 2.0);
        initial_paragraph.paint(canvas, (ix, iy));
    }

    // 6. Encode to WebP
    let snapshot = surface.image_snapshot();
    let data = snapshot.encode_to_data_with_quality(EncodedImageFormat::WEBP, 92)?;
    
    Some(data.as_bytes().to_vec())
}
