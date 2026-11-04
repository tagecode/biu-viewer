use image::metadata::Orientation;
use image::DynamicImage;

/// 按 EXIF Orientation（1–8）变换像素。无法识别的值按 1（不变换）处理。
pub fn apply_exif_orientation(image: DynamicImage, exif_value: u8) -> DynamicImage {
    let orientation = Orientation::from_exif(exif_value).unwrap_or(Orientation::NoTransforms);
    let mut image = image;
    image.apply_orientation(orientation);
    image
}

/// 从原始 EXIF/TIFF 块读取方向。缺失或损坏时返回 1。
pub fn orientation_value_from_exif(chunk: &[u8]) -> u8 {
    Orientation::from_exif_chunk(chunk)
        .map(Orientation::to_exif)
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use biuviewer_core::TransformState;
    use image::{Rgba, RgbaImage};

    const A: [u8; 4] = [1, 0, 0, 255];
    const B: [u8; 4] = [0, 1, 0, 255];
    const C: [u8; 4] = [0, 0, 1, 255];
    const D: [u8; 4] = [1, 1, 1, 255];

    fn pattern() -> DynamicImage {
        let mut image = RgbaImage::new(2, 2);
        image.put_pixel(0, 0, Rgba(A));
        image.put_pixel(1, 0, Rgba(B));
        image.put_pixel(0, 1, Rgba(C));
        image.put_pixel(1, 1, Rgba(D));
        DynamicImage::ImageRgba8(image)
    }

    fn raw(image: &DynamicImage) -> Vec<[u8; 4]> {
        let rgba = image.to_rgba8();
        rgba.pixels().map(|pixel| pixel.0).collect()
    }

    #[test]
    fn eight_orientations_match_the_expected_pixels() {
        let source = pattern();
        let cases = [
            (1_u8, vec![A, B, C, D]),
            (2, vec![B, A, D, C]),
            (3, vec![D, C, B, A]),
            (4, vec![C, D, A, B]),
            (5, vec![A, C, B, D]),
            (6, vec![C, A, D, B]),
            (7, vec![D, B, C, A]),
            (8, vec![B, D, A, C]),
        ];
        for (orientation, expected) in cases {
            let corrected = apply_exif_orientation(source.clone(), orientation);
            assert_eq!(raw(&corrected), expected, "orientation {orientation}");
        }
    }

    #[test]
    fn unknown_orientation_is_left_unchanged() {
        let source = pattern();
        assert_eq!(
            raw(&apply_exif_orientation(source.clone(), 0)),
            raw(&source)
        );
        assert_eq!(
            raw(&apply_exif_orientation(source.clone(), 9)),
            raw(&source)
        );
    }

    #[test]
    fn corrupt_exif_chunk_is_treated_as_orientation_one() {
        assert_eq!(orientation_value_from_exif(b"not exif"), 1);
        assert_eq!(orientation_value_from_exif(&[]), 1);
    }

    #[test]
    fn exif_chunk_round_trips_orientation_six() {
        assert_eq!(orientation_value_from_exif(&exif_chunk(6)), 6);
    }

    #[test]
    fn jpeg_orientation_six_swaps_width_and_height() {
        let decoded = crate::decode_static(&jpeg_with_exif(2, 3, &exif_chunk(6))).unwrap();
        assert_eq!((decoded.width, decoded.height), (3, 2));
    }

    #[test]
    fn corrupt_jpeg_exif_still_displays_the_image() {
        let decoded = crate::decode_static(&jpeg_with_exif(2, 3, b"garbagegarbage")).unwrap();
        assert_eq!((decoded.width, decoded.height), (2, 3));
    }

    #[test]
    fn resetting_user_transform_keeps_exif_corrected_pixels() {
        let source = pattern();
        let corrected = apply_exif_orientation(source.clone(), 6);
        let mut transform = TransformState {
            quarter_turns_cw: 1,
            flip_horizontal: true,
            flip_vertical: false,
        };
        transform.reset();
        assert!(transform.is_identity());
        assert_ne!(raw(&corrected), raw(&source));
        assert_eq!(raw(&corrected), vec![C, A, D, B]);
    }

    fn jpeg_with_exif(width: u32, height: u32, exif_body: &[u8]) -> Vec<u8> {
        let image = RgbaImage::from_pixel(width, height, Rgba([9, 8, 7, 255]));
        let mut cursor = std::io::Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut cursor, image::ImageFormat::Jpeg)
            .unwrap();
        let jpeg = cursor.into_inner();
        let mut payload = b"Exif\0\0".to_vec();
        payload.extend_from_slice(exif_body);
        let length = u16::try_from(payload.len() + 2).unwrap();
        let mut output = Vec::with_capacity(jpeg.len() + payload.len() + 4);
        output.extend_from_slice(&[0xFF, 0xD8, 0xFF, 0xE1]);
        output.extend_from_slice(&length.to_be_bytes());
        output.extend(payload);
        output.extend_from_slice(&jpeg[2..]);
        output
    }

    fn exif_chunk(orientation: u16) -> Vec<u8> {
        let mut chunk = vec![
            b'I', b'I', 0x2A, 0x00, // little-endian TIFF
            0x08, 0x00, 0x00, 0x00, // IFD at offset 8
            0x01, 0x00, // one entry
            0x12, 0x01, // tag 0x0112 Orientation
            0x03, 0x00, // SHORT
            0x01, 0x00, 0x00, 0x00, // count 1
        ];
        chunk.extend(orientation.to_le_bytes());
        chunk.extend([0, 0]);
        chunk
    }
}
