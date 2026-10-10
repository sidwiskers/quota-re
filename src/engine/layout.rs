use crate::engine::theme::Theme;
use crate::fonts::manager::FontManager;
use skia_safe::textlayout::Paragraph;

/// Calculated dimensions and shaped text for a QuotLy-style sticker bubble.
pub struct BubbleLayout {
    pub render_scale: f32,
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
        has_avatar: bool,
    ) -> Self {
        let scale = render_scale.clamp(0.25, 1.0);
        let max_bubble_width = 440.0 * scale;
        let min_bubble_width = 180.0 * scale;
        let horizontal_padding = 18.0 * scale;
        let vertical_padding = 13.0 * scale;
        let name_font_size = 25.0 * scale;
        let message_font_size = 25.0 * scale;
        let reply_font_size = 17.0 * scale;

        let avatar_space = if has_avatar { 46.0 * scale } else { 0.0 };
        let name_paragraph = font_mgr.build_paragraph(
            if username.trim().is_empty() { " " } else { username },
            name_font_size,
            "Roboto",
            (max_bubble_width - avatar_space).max(1.0),
            theme.username,
            Some(1),
        );
        let name_width = name_paragraph.max_intrinsic_width();
        let name_height = name_paragraph
            .height()
            .max(name_font_size + 4.0 * scale)
            .max(if has_avatar { 36.0 * scale } else { 0.0 });

        // Measure intrinsic message width separately from the final wrapped
        // paragraph, then bound the bubble to the fixed design width.
        let message_measure = font_mgr.build_paragraph(
            message,
            message_font_size,
            "Roboto",
            max_bubble_width,
            theme.text,
            None,
        );
        let content_width = (name_width + avatar_space).max(message_measure.max_intrinsic_width());
        let bubble_width = min_bubble_width
            .max(content_width + horizontal_padding * 2.0 + 12.0 * scale)
            .min(max_bubble_width);
        let inner_width = (bubble_width - horizontal_padding * 2.0).max(1.0);

        let msg_paragraph = font_mgr.build_paragraph(
            message,
            message_font_size,
            "Roboto",
            inner_width,
            theme.text,
            Some(8),
        );
        let message_height = msg_paragraph.height();

        let mut has_reply = false;
        let mut reply_name_paragraph = None;
        let mut reply_msg_paragraph = None;
        let mut reply_block_height = 0.0;

        if let Some((reply_username, reply_message)) = reply {
            has_reply = true;
            reply_block_height = reply_font_size * 4.2;

            reply_name_paragraph = Some(font_mgr.build_paragraph(
                reply_username,
                reply_font_size,
                "Roboto",
                (inner_width - 20.0 * scale).max(1.0),
                theme.reply_name,
                Some(1),
            ));
            reply_msg_paragraph = Some(font_mgr.build_paragraph(
                reply_message,
                reply_font_size,
                "Roboto",
                (inner_width - 20.0 * scale).max(1.0),
                theme.reply_text,
                Some(2),
            ));
        }

        let bubble_height = (
            vertical_padding
                + name_height
                + 6.0 * scale
                + reply_block_height
                + message_height
                + vertical_padding
                + 6.0 * scale
        )
            .max(60.0 * scale);

        Self {
            render_scale: scale,
            bubble_width,
            bubble_height,
            name_paragraph,
            msg_paragraph,
            has_reply,
            reply_name_paragraph,
            reply_msg_paragraph,
            reply_block_height,
        }
    }
}
