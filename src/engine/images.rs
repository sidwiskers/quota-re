use skia_safe::{canvas::SrcRectConstraint, Canvas, Data, Image, Paint, Rect};

const MAX_ENCODED_IMAGE_BYTES: usize = 10 * 1024 * 1024;
const MAX_IMAGE_SIDE: i32 = 8_192;
const MAX_IMAGE_PIXELS: u64 = 12_000_000;

/// Decode a bounded image for rendering. Skia keeps many encoded images lazy,
/// but dimensions are checked before any draw operation can force decoding.
pub fn decode_safe_image(bytes: &[u8]) -> Option<Image> {
    if bytes.is_empty() || bytes.len() > MAX_ENCODED_IMAGE_BYTES {
        return None;
    }

    let image = Image::from_encoded(Data::new_copy(bytes))?;
    let width = image.width();
    let height = image.height();

    if width <= 0
        || height <= 0
        || width > MAX_IMAGE_SIDE
        || height > MAX_IMAGE_SIDE
        || (width as u64).saturating_mul(height as u64) > MAX_IMAGE_PIXELS
    {
        return None;
    }

    Some(image)
}

/// Draw an image with a centered crop so its aspect ratio matches the target.
pub fn draw_cropped_image(canvas: &Canvas, image: &Image, destination: Rect, paint: &Paint) {
    let source_width = image.width() as f32;
    let source_height = image.height() as f32;
    let destination_width = destination.width().max(1.0);
    let destination_height = destination.height().max(1.0);
    let source_ratio = source_width / source_height;
    let destination_ratio = destination_width / destination_height;

    let (crop_width, crop_height, left, top) = if source_ratio > destination_ratio {
        let crop_width = source_height * destination_ratio;
        (crop_width, source_height, (source_width - crop_width) / 2.0, 0.0)
    } else {
        let crop_height = source_width / destination_ratio;
        (source_width, crop_height, 0.0, (source_height - crop_height) / 2.0)
    };

    let source = Rect::from_xywh(left, top, crop_width, crop_height);
    canvas.draw_image_rect(image, Some((&source, SrcRectConstraint::Strict)), destination, paint);
}

#[cfg(test)]
mod tests {
    use super::decode_safe_image;

    #[test]
    fn rejects_empty_and_non_image_data() {
        assert!(decode_safe_image(&[]).is_none());
        assert!(decode_safe_image(b"not an encoded image").is_none());
    }

    #[test]
    fn rejects_encoded_images_over_the_input_limit() {
        let oversized = vec![0_u8; 10 * 1024 * 1024 + 1];
        assert!(decode_safe_image(&oversized).is_none());
    }
}
