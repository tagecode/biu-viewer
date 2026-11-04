use std::io::Cursor;

use biuviewer_core::ViewerError;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageError, ImageReader, Limits};

use crate::exif::apply_exif_orientation;

/// 单边像素上限。超过后拒绝解码，避免按声明尺寸分配内存。
pub const MAX_EDGE: u32 = 32_768;
/// 总像素上限。
pub const MAX_PIXELS: u64 = 200_000_000;

/// 解码后的 RGBA8 像素缓冲。行优先，每个像素 4 字节。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl DecodedImage {
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let index = (u64::from(y) * u64::from(self.width) + u64::from(x)) * 4;
        let start = usize::try_from(index).unwrap_or(0);
        let bytes = &self.rgba[start..start + 4];
        [bytes[0], bytes[1], bytes[2], bytes[3]]
    }
}

/// 可放到后台线程执行的同步解码后端。
pub trait DecoderBackend: Send + Sync {
    fn decode(&self, bytes: &[u8]) -> Result<DecodedImage, ViewerError>;
}

/// 基于 `image` crate 的默认解码器。
#[derive(Debug, Default, Clone, Copy)]
pub struct ImageCrateDecoder;

impl DecoderBackend for ImageCrateDecoder {
    fn decode(&self, bytes: &[u8]) -> Result<DecodedImage, ViewerError> {
        decode_static(bytes)
    }
}

/// 同步解码。先读尺寸并拒绝超限图片，再把像素转成 RGBA8。
pub fn decode_static(bytes: &[u8]) -> Result<DecodedImage, ViewerError> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|err| ViewerError::Io(err.to_string()))?;
    let mut decoder = reader.into_decoder().map_err(map_image_error)?;
    let (width, height) = decoder.dimensions();
    reject_oversized(width, height)?;
    let exif_value = match decoder.orientation() {
        Ok(orientation) => orientation.to_exif(),
        Err(err) => {
            tracing::warn!("读取 EXIF 方向失败，按正向显示: {err}");
            Orientation::NoTransforms.to_exif()
        }
    };

    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_EDGE);
    limits.max_image_height = Some(MAX_EDGE);
    limits.max_alloc = Some(MAX_PIXELS.saturating_mul(4));
    decoder.set_limits(limits).map_err(map_image_error)?;

    let image = DynamicImage::from_decoder(decoder).map_err(map_image_error)?;
    let image = apply_exif_orientation(image, exif_value);
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    reject_oversized(width, height)?;
    Ok(DecodedImage {
        width,
        height,
        rgba: rgba.into_raw(),
    })
}

fn reject_oversized(width: u32, height: u32) -> Result<(), ViewerError> {
    let pixels = u64::from(width).saturating_mul(u64::from(height));
    if width > MAX_EDGE || height > MAX_EDGE || pixels > MAX_PIXELS {
        return Err(ViewerError::Unsupported(format!(
            "图片尺寸超出上限（{width}×{height}）"
        )));
    }
    Ok(())
}

pub(crate) fn map_image_error(err: ImageError) -> ViewerError {
    match err {
        ImageError::Unsupported(_) => ViewerError::Unsupported(err.to_string()),
        ImageError::Limits(_) => ViewerError::Unsupported(format!("图片尺寸超出上限（{err}）")),
        // 输入已经是内存中的字节，提前结束表示文件被截断，而不是文件系统故障。
        ImageError::IoError(_) => ViewerError::Corrupted(err.to_string()),
        ImageError::Decoding(_) | ImageError::Encoding(_) | ImageError::Parameter(_) => {
            ViewerError::Corrupted(err.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use biuviewer_core::ViewerError;
    use image::{DynamicImage, ImageFormat as EncodedFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    fn solid(width: u32, height: u32, pixel: [u8; 4]) -> RgbaImage {
        RgbaImage::from_pixel(width, height, Rgba(pixel))
    }

    fn encode(image: &RgbaImage, format: EncodedFormat) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image.clone())
            .write_to(&mut cursor, format)
            .unwrap();
        cursor.into_inner()
    }

    fn assert_exact(format: EncodedFormat) {
        let pixel = [12, 34, 56, 255];
        let bytes = encode(&solid(2, 3, pixel), format);
        let decoded = ImageCrateDecoder.decode(&bytes).unwrap();
        assert_eq!((decoded.width, decoded.height), (2, 3));
        assert_eq!(decoded.rgba.len(), 2 * 3 * 4);
        assert_eq!(decoded.pixel(1, 2), pixel);
    }

    #[test]
    fn decodes_lossless_p0_formats() {
        assert_exact(EncodedFormat::Png);
        assert_exact(EncodedFormat::Bmp);
        assert_exact(EncodedFormat::Gif);
        assert_exact(EncodedFormat::Tiff);
        assert_exact(EncodedFormat::Ico);
        assert_exact(EncodedFormat::WebP);
    }

    #[test]
    fn decodes_jpeg_dimensions() {
        let bytes = encode(&solid(4, 5, [200, 10, 10, 255]), EncodedFormat::Jpeg);
        let decoded = ImageCrateDecoder.decode(&bytes).unwrap();
        assert_eq!((decoded.width, decoded.height), (4, 5));
        assert_eq!(decoded.rgba.len(), 4 * 5 * 4);
    }

    #[test]
    fn truncated_png_is_corrupted_and_does_not_panic() {
        let err = ImageCrateDecoder.decode(b"\x89PNG\r\n\x1a\n").unwrap_err();
        assert!(
            matches!(err, ViewerError::Corrupted(_)),
            "truncated file should be Corrupted, got {err:?}"
        );
    }

    #[test]
    fn oversized_edge_is_rejected_without_decoding_pixels() {
        let err = ImageCrateDecoder
            .decode(&bmp_with_size(40_000, 1))
            .unwrap_err();
        assert!(err.user_message().contains("尺寸超出上限"));
    }

    #[test]
    fn oversized_pixel_count_is_rejected() {
        let err = ImageCrateDecoder
            .decode(&bmp_with_size(20_000, 20_000))
            .unwrap_err();
        assert!(err.user_message().contains("尺寸超出上限"));
    }

    fn bmp_with_size(width: i32, height: i32) -> Vec<u8> {
        let mut data = vec![0_u8; 54];
        data[0] = b'B';
        data[1] = b'M';
        data[10..14].copy_from_slice(&54_u32.to_le_bytes());
        data[14..18].copy_from_slice(&40_u32.to_le_bytes());
        data[18..22].copy_from_slice(&width.to_le_bytes());
        data[22..26].copy_from_slice(&height.to_le_bytes());
        data[26..28].copy_from_slice(&1_u16.to_le_bytes());
        data[28..30].copy_from_slice(&24_u16.to_le_bytes());
        data
    }
}
