use crate::engine::theme::Theme;
use crate::fonts::manager::FontManager;
use skia_safe::textlayout::Paragraph;

/// Represents the calculated dimensions and layout for a QuotLy-style sticker bubble
pub struct BubbleLayout {
    pub bubble_width: f32,
    pub bubble_height: f32,
    pub name_paragraph: Paragraph,
    pub msg_paragraph: Paragraph,
    pub has_reply: bool,
    pub reply_name_paragraph: Option<Paragraph>,
    pub reply_msg_paragraph: Option<Paragraph>,
    pub reply_block_height: f32,
}

impl BubbleLayout {
    pub fn calculate(
        font_mgr: &FontManager,
        username: &str,
        message: &str,
        reply: Option<(&str, &str)>,
        theme: &Theme,
        render_scale: f32,
    ) -> Self {
        let max_bub_w = 440.0 * render_scale;
        let min_bub_w = 180.0 * render_scale;
        let iph = 18.0 * render_scale;
        let ipv = 13.0 * render_scale;

        let name_fs = 25.0 * render_scale;
        let msg_fs = 25.0 * render_scale;
        let rep_fs = 17.0 * render_scale;

        let name_paragraph = font_mgr.build_paragraph(
            if username.is_empty() { "A" } else { username },
            name_fs,
            "Roboto",
            max_bub_w,
            theme.username,
            Some(1),
        );
        let name_w = name_paragraph.max_intrinsic_width();
        let name_h = name_paragraph.height().max(name_fs + 4.0 * render_scale);

        let msg_single = font_mgr.build_paragraph(message, msg_fs, "Roboto", f32::MAX, theme.text, None);
        let msg_single_w = msg_single.max_intrinsic_width();

        let content_max_w = name_w.max(msg_single_w);
        let mut bubble_w = min_bub_w.max(content_max_w + iph * 2.0 + 12.0 * render_scale);
        bubble_w = bubble_w.min(max_bub_w);
        let inner_w = bubble_w - iph * 2.0;

        let msg_paragraph = font_mgr.build_paragraph(message, msg_fs, "Roboto", inner_w, theme.text, None);
        let msg_h = msg_paragraph.height();

        let mut has_reply = false;
        let mut reply_name_paragraph = None;
        let mut reply_msg_paragraph = None;
        let mut reply_block_height = 0.0;

        if let Some((rep_user, rep_msg)) = reply {
            has_reply = true;
            reply_block_height = rep_fs * 3.6;

            reply_name_paragraph = Some(font_mgr.build_paragraph(
                rep_user, rep_fs, "Roboto", inner_w - 20.0 * render_scale, theme.reply_name, Some(1),
            ));

            reply_msg_paragraph = Some(font_mgr.build_paragraph(
                rep_msg, rep_fs, "Roboto", inner_w - 20.0 * render_scale, theme.reply_text, Some(1),
            ));
        }

        let mut bubble_h = ipv + name_h + (6.0 * render_scale) + reply_block_height + msg_h + ipv + (6.0 * render_scale);
        bubble_h = bubble_h.max(60.0 * render_scale);

        Self {
            bubble_width: bubble_w,
            bubble_height: bubble_h,
            name_paragraph,
            msg_paragraph,
            has_reply,
            reply_name_paragraph,
            reply_msg_paragraph,
            reply_block_height,
        }
    }
}
