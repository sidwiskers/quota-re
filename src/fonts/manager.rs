use skia_safe::{
    textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle, TypefaceFontProvider},
    Data, FontMgr, Typeface,
};
use std::sync::Arc;

pub struct FontManager {
    pub collection: FontCollection,
}

impl FontManager {
    /// Initializes the Skia FontCollection. 
    /// Unlike PIL which requires manual per-character script detection and manual font loading,
    /// Skia's textlayout engine handles HarfBuzz shaping, bidi, and font fallback automatically.
    pub fn new() -> Self {
        let mut font_provider = TypefaceFontProvider::new();
        let mut collection = FontCollection::new();

        // Load the system default font manager to handle standard emojis (Noto Color Emoji)
        // and CJK / Arabic fallbacks seamlessly.
        let default_mgr = FontMgr::new();
        collection.set_default_font_manager(default_mgr, None);

        // TODO: In production, we will load the exact "Roboto" TTF files from disk or memory 
        // using font_provider.register_typeface(Typeface::from_data(...)) to match QuotLy exactly.
        
        collection.set_asset_font_manager(Some(font_provider.clone().into()));

        Self { collection }
    }

    /// Helper to create a fully shaped paragraph ready for measurement and rendering
    pub fn build_paragraph(
        &self,
        text: &str,
        font_size: f32,
        font_family: &str,
        max_width: f32,
        color: skia_safe::Color,
        max_lines: Option<usize>,
    ) -> Paragraph {
        let mut text_style = TextStyle::new();
        text_style.set_color(color);
        text_style.set_font_size(font_size);
        text_style.set_font_families(&[font_family, "Noto Color Emoji", "sans-serif"]);

        let mut paragraph_style = ParagraphStyle::new();
        paragraph_style.set_text_style(&text_style);
        if let Some(lines) = max_lines {
            paragraph_style.set_max_lines(lines);
            paragraph_style.set_ellipsis("…");
        }

        let mut builder = ParagraphBuilder::new(&paragraph_style, &self.collection);
        builder.push_style(&text_style);
        builder.add_text(text);
        
        let mut paragraph = builder.build();
        // Layout the paragraph with the given constraints
        paragraph.layout(max_width);
        
        paragraph
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}
