use crate::fonts::fallback::paragraph_direction;
use skia_safe::{
    textlayout::{
        FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextAlign, TextStyle,
    },
    FontMgr,
};
use std::cell::RefCell;

/// Skia's paragraph FontCollection is not Send/Sync in this version.
/// Keep one instance per blocking-render worker thread instead of sharing it
/// through Axum application state.
thread_local! {
    static THREAD_FONT_MANAGER: RefCell<FontManager> =
        RefCell::new(FontManager::new());
}

pub struct FontManager {
    collection: FontCollection,
}

impl FontManager {
    fn new() -> Self {
        let mut collection = FontCollection::new();

        // Fontconfig supplies deterministic system-installed Roboto and Noto
        // Color Emoji fonts in the Docker runtime, with system fallback for
        // other scripts and characters.
        collection.set_default_font_manager(FontMgr::new(), None);

        Self { collection }
    }

    /// Run a rendering operation with the font manager owned by this thread.
    /// The manager and all Skia paragraph objects remain on the same thread.
    pub fn with_thread_local<R>(f: impl FnOnce(&FontManager) -> R) -> R {
        THREAD_FONT_MANAGER.with(|manager| {
            let manager = manager.borrow();
            f(&manager)
        })
    }

    /// Create a shaped paragraph and lay it out at the requested width.
    pub fn build_paragraph(
        &self,
        text: &str,
        font_size: f32,
        font_family: &str,
        max_width: f32,
        color: skia_safe::Color,
        max_lines: Option<usize>,
    ) -> Paragraph {
        self.build_paragraph_aligned(
            text,
            font_size,
            font_family,
            max_width,
            color,
            max_lines,
            TextAlign::Start,
        )
    }

    /// The same paragraph builder with explicit alignment for quote layouts.
    pub fn build_paragraph_aligned(
        &self,
        text: &str,
        font_size: f32,
        font_family: &str,
        max_width: f32,
        color: skia_safe::Color,
        max_lines: Option<usize>,
        alignment: TextAlign,
    ) -> Paragraph {
        let mut text_style = TextStyle::new();
        text_style.set_color(color);
        text_style.set_font_size(font_size);
        text_style.set_font_families(&[font_family, "Noto Color Emoji", "sans-serif"]);

        let mut paragraph_style = ParagraphStyle::new();
        paragraph_style.set_text_style(&text_style);
        paragraph_style.set_text_align(alignment);
        paragraph_style.set_text_direction(paragraph_direction(text));
        if let Some(lines) = max_lines {
            paragraph_style.set_max_lines(lines);
            paragraph_style.set_ellipsis("…");
        }

        let mut builder = ParagraphBuilder::new(&paragraph_style, &self.collection);
        builder.push_style(&text_style);
        builder.add_text(text);

        let mut paragraph = builder.build();
        paragraph.layout(max_width.max(1.0));
        paragraph
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

