use skia_safe::textlayout::TextDirection;

/// Select a paragraph's base direction from its first strong character.
/// Skia's paragraph engine still handles mixed-direction runs and shaping.
pub fn paragraph_direction(text: &str) -> TextDirection {
    for character in text.chars() {
        let codepoint = character as u32;
        if matches!(
            codepoint,
            0x0590..=0x08FF
                | 0xFB1D..=0xFDFF
                | 0xFE70..=0xFEFF
                | 0x10800..=0x10FFF
                | 0x1E800..=0x1EEFF
        ) {
            return TextDirection::RTL;
        }
        if character.is_alphabetic() {
            return TextDirection::LTR;
        }
    }

    TextDirection::LTR
}

#[cfg(test)]
mod tests {
    use super::paragraph_direction;
    use skia_safe::textlayout::TextDirection;

    #[test]
    fn detects_common_right_to_left_scripts() {
        assert_eq!(paragraph_direction("مرحبا بالعالم"), TextDirection::RTL);
        assert_eq!(paragraph_direction("שלום"), TextDirection::RTL);
    }

    #[test]
    fn keeps_latin_text_left_to_right_even_with_leading_punctuation() {
        assert_eq!(paragraph_direction("— Hello"), TextDirection::LTR);
        assert_eq!(paragraph_direction("🙂 Hello مرحبا"), TextDirection::LTR);
    }
}
