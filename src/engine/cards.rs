use crate::engine::images::{decode_safe_image, draw_cropped_image};
use crate::fonts::manager::FontManager;
use skia_safe::{Color, EncodedImageFormat, Paint, Point, Rect, RRect, Surface};

pub fn render_audio_card(
    font_mgr: &FontManager,
    title: &str,
    performer: &str,
    duration: i32, // seconds
    progress: f32, // 0.0 to 1.0
    thumb_bytes: Option<&[u8]>,
) -> Option<Vec<u8>> {
    let canvas_w = 600.0;
    let canvas_h = 220.0;
    
    let mut surface = Surface::new_raster_n32_premul((canvas_w as i32, canvas_h as i32))?;
    let canvas = surface.canvas();

    // 1. Draw Background (Dark Theme)
    let mut bg_paint = Paint::default();
    bg_paint.set_color(Color::from_rgb(29, 30, 44));
    let bg_rect = Rect::from_xywh(0.0, 0.0, canvas_w, canvas_h);
    canvas.draw_rrect(RRect::new_rect_xy(bg_rect, 20.0, 20.0), &bg_paint);

    // 2. Draw Thumbnail
    let thumb_size = 140.0;
    let thumb_x = 40.0;
    let thumb_y = 40.0;
    let thumb_rect = Rect::from_xywh(thumb_x, thumb_y, thumb_size, thumb_size);

    if let Some(image) = thumb_bytes.and_then(decode_safe_image) {
        canvas.save();
        canvas.clip_rrect(RRect::new_rect_xy(thumb_rect, 15.0, 15.0), None, true);

        let mut img_paint = Paint::default();
        img_paint.set_anti_alias(true);
        draw_cropped_image(canvas, &image, thumb_rect, &img_paint);
        canvas.restore();
    } else {
        let mut p = Paint::default();
        p.set_anti_alias(true);
        p.set_color(Color::from_rgb(60, 65, 75));
        canvas.draw_rrect(RRect::new_rect_xy(thumb_rect, 15.0, 15.0), &p);
    }

    // 3. Draw Text (Title & Performer)
    let text_x = thumb_x + thumb_size + 30.0;
    let max_text_w = canvas_w - text_x - 40.0;
    
    let title_para = font_mgr.build_paragraph(title, 32.0, "Roboto", max_text_w, Color::WHITE, Some(1));
    title_para.paint(canvas, (text_x, thumb_y + 10.0));

    let perf_para = font_mgr.build_paragraph(performer, 22.0, "Roboto", max_text_w, Color::from_rgb(170, 170, 170), Some(1));
    perf_para.paint(canvas, (text_x, thumb_y + 50.0));

    // 4. Progress Bar
    let bar_y = thumb_y + 110.0;
    let bar_w = max_text_w;
    let bar_h = 6.0;
    
    let mut bar_bg = Paint::default();
    bar_bg.set_color(Color::from_argb(50, 255, 255, 255));
    canvas.draw_rrect(RRect::new_rect_xy(Rect::from_xywh(text_x, bar_y, bar_w, bar_h), 3.0, 3.0), &bar_bg);

    let progress_clamped = progress.max(0.0).min(1.0);
    let mut bar_fg = Paint::default();
    bar_fg.set_color(Color::from_rgb(100, 190, 255)); // Telegram Audio Blue
    canvas.draw_rrect(RRect::new_rect_xy(Rect::from_xywh(text_x, bar_y, bar_w * progress_clamped, bar_h), 3.0, 3.0), &bar_fg);

    if progress_clamped > 0.0 {
        canvas.draw_circle(Point::new(text_x + (bar_w * progress_clamped), bar_y + (bar_h / 2.0)), 8.0, &bar_fg);
    }

    // 5. Time Labels
    let current_sec = ((duration as f32) * progress_clamped) as i32;
    let format_time = |secs: i32| format!("{:02}:{:02}", secs / 60, secs % 60);

    let time_curr_para = font_mgr.build_paragraph(&format_time(current_sec), 18.0, "Roboto", 100.0, Color::from_rgb(150, 150, 150), Some(1));
    time_curr_para.paint(canvas, (text_x, bar_y + 15.0));

    let time_tot_para = font_mgr.build_paragraph(&format_time(duration), 18.0, "Roboto", 100.0, Color::from_rgb(150, 150, 150), Some(1));
    time_tot_para.paint(canvas, (text_x + bar_w - time_tot_para.max_intrinsic_width(), bar_y + 15.0));

    let snapshot = surface.image_snapshot();
    let data = snapshot.encode_to_data_with_quality(EncodedImageFormat::PNG, 100)?;
    
    Some(data.as_bytes().to_vec())
}

pub fn render_file_card(
    font_mgr: &FontManager,
    file_name: &str,
    file_size_str: &str,
    file_ext: &str,
    thumb_bytes: Option<&[u8]>,
) -> Option<Vec<u8>> {
    let canvas_w = 600.0;
    let canvas_h = 180.0;
    
    let mut surface = Surface::new_raster_n32_premul((canvas_w as i32, canvas_h as i32))?;
    let canvas = surface.canvas();

    // 1. Draw Background
    let mut bg_paint = Paint::default();
    bg_paint.set_color(Color::from_rgb(29, 30, 44));
    let bg_rect = Rect::from_xywh(0.0, 0.0, canvas_w, canvas_h);
    canvas.draw_rrect(RRect::new_rect_xy(bg_rect, 20.0, 20.0), &bg_paint);

    // 2. Draw Thumbnail OR Extension Icon
    let thumb_size = 100.0;
    let thumb_x = 40.0;
    let thumb_y = 40.0;
    let thumb_rect = Rect::from_xywh(thumb_x, thumb_y, thumb_size, thumb_size);

    if let Some(image) = thumb_bytes.and_then(decode_safe_image) {
        canvas.save();
        canvas.clip_rrect(RRect::new_oval(thumb_rect), None, true);

        let mut img_paint = Paint::default();
        img_paint.set_anti_alias(true);
        draw_cropped_image(canvas, &image, thumb_rect, &img_paint);
        canvas.restore();
    } else {
        let mut p = Paint::default();
        p.set_anti_alias(true);
        p.set_color(Color::from_rgb(100, 190, 255)); // Telegram File Blue
        canvas.draw_rrect(RRect::new_oval(thumb_rect), &p);

        let display_ext = if file_ext.is_empty() { "FILE" } else { file_ext }.to_uppercase();
        let ext_para = font_mgr.build_paragraph(&display_ext, 28.0, "Roboto", thumb_size - 10.0, Color::WHITE, Some(1));

        let ex = thumb_x + (thumb_size - ext_para.max_intrinsic_width()) / 2.0;
        let ey = thumb_y + (thumb_size - ext_para.height()) / 2.0;
        ext_para.paint(canvas, (ex, ey));
    }

    // 3. Draw Text (File Name & Size)
    let text_x = thumb_x + thumb_size + 30.0;
    let max_text_w = canvas_w - text_x - 40.0;
    
    let name_para = font_mgr.build_paragraph(file_name, 30.0, "Roboto", max_text_w, Color::WHITE, Some(1));
    let text_block_h = name_para.height() + 10.0 + 24.0;
    let start_y = (canvas_h - text_block_h) / 2.0;

    name_para.paint(canvas, (text_x, start_y));

    let size_para = font_mgr.build_paragraph(file_size_str, 22.0, "Roboto", max_text_w, Color::from_rgb(170, 170, 170), Some(1));
    size_para.paint(canvas, (text_x, start_y + name_para.height() + 10.0));

    let snapshot = surface.image_snapshot();
    let data = snapshot.encode_to_data_with_quality(EncodedImageFormat::PNG, 100)?;
    
    Some(data.as_bytes().to_vec())
}
