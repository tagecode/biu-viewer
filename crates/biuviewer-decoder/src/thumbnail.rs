//! 缩略图解码：JPEG 利用 DCT 缩放先解到接近目标尺寸，再统一缩放到长边上限。

use std::io::Cursor;

use biuviewer_core::ViewerError;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageReader, RgbaImage};
use jpeg_decoder::{Decoder as JpegDecoder, PixelFormat};

use crate::animation::decode_gif_first_frame;
use crate::decode::{decode_static, reject_oversized, DecodedImage};
use crate::exif::apply_exif_orientation;
use crate::format::ImageFormat;

/// 解码缩略图。`max_edge` 是长边上限；JPEG 会按 1/8、1/4、1/2 的 DCT 比例先缩小，
/// 避免为了 128px 缩略图先分配整张 12MP 的 RGBA 缓冲。
pub fn decode_thumbnail(
    bytes: &[u8],
    format: ImageFormat,
    max_edge: u32,
) -> Result<DecodedImage, ViewerError> {
    if format == ImageFormat::Jpeg {
        if let Some(scaled) = decode_jpeg_scaled(bytes, max_edge)? {
            return Ok(scaled);
        }
    }

    let decoded = if format == ImageFormat::Gif {
        decode_gif_first_frame(bytes)?
    } else {
        decode_static(bytes)?
    };
    resize_to_fit(decoded, max_edge)
}

fn decode_jpeg_scaled(bytes: &[u8], max_edge: u32) -> Result<Option<DecodedImage>, ViewerError> {
    let orientation = jpeg_orientation(bytes);
    let mut decoder = JpegDecoder::new(Cursor::new(bytes));
    decoder.read_info().map_err(map_jpeg_error)?;
    let Some(info) = decoder.info() else {
        return Ok(None);
    };
    // 灰度和 CMYK 走完整解码，避免在这里重复实现颜色转换。
    if info.pixel_format != PixelFormat::RGB24 {
        return Ok(None);
    }
    let width = u32::from(info.width);
    let height = u32::from(info.height);
    reject_oversized(width, height)?;

    let (target_width, target_height) = fit_within(width, height, max_edge);
    let target_width = u16::try_from(target_width).unwrap_or(u16::MAX);
    let target_height = u16::try_from(target_height).unwrap_or(u16::MAX);
    let (scaled_width, scaled_height) = decoder
        .scale(target_width, target_height)
        .map_err(map_jpeg_error)?;

    let pixels = decoder.decode().map_err(map_jpeg_error)?;
    let pixel_count = usize::from(scaled_width) * usize::from(scaled_height);
    if pixels.len() < pixel_count.saturating_mul(3) {
        return Ok(None);
    }

    let mut rgba = Vec::with_capacity(pixel_count.saturating_mul(4));
    for pixel in pixels.as_chunks::<3>().0 {
        rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
    }
    let image = RgbaImage::from_raw(u32::from(scaled_width), u32::from(scaled_height), rgba)
        .ok_or_else(|| ViewerError::Corrupted("JPEG 解码缓冲尺寸不一致".to_string()))?;
    let oriented = apply_exif_orientation(DynamicImage::ImageRgba8(image), orientation);
    let oriented = oriented.to_rgba8();
    let (width, height) = oriented.dimensions();
    resize_to_fit(
        DecodedImage {
            width,
            height,
            rgba: oriented.into_raw(),
        },
        max_edge,
    )
    .map(Some)
}

fn jpeg_orientation(bytes: &[u8]) -> u8 {
    ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()
        .and_then(|reader| reader.into_decoder().ok())
        .and_then(|mut decoder| decoder.orientation().ok())
        .map(Orientation::to_exif)
        .unwrap_or(1)
}

fn map_jpeg_error(err: jpeg_decoder::Error) -> ViewerError {
    match err {
        jpeg_decoder::Error::Unsupported(feature) => {
            ViewerError::Unsupported(format!("不支持的 JPEG 特性：{feature:?}"))
        }
        jpeg_decoder::Error::Io(err) => ViewerError::from(err),
        other => ViewerError::Corrupted(other.to_string()),
    }
}

fn resize_to_fit(decoded: DecodedImage, max_edge: u32) -> Result<DecodedImage, ViewerError> {
    let (target_width, target_height) = fit_within(decoded.width, decoded.height, max_edge);
    if (target_width, target_height) == (decoded.width, decoded.height) {
        return Ok(decoded);
    }
    let source = RgbaImage::from_raw(decoded.width, decoded.height, decoded.rgba)
        .ok_or_else(|| ViewerError::Corrupted("缩略图缓冲尺寸不一致".to_string()))?;
    let resized = image::imageops::resize(
        &source,
        target_width,
        target_height,
        image::imageops::FilterType::Triangle,
    );
    Ok(DecodedImage {
        width: target_width,
        height: target_height,
        rgba: resized.into_raw(),
    })
}

fn fit_within(width: u32, height: u32, edge: u32) -> (u32, u32) {
    if width == 0 || height == 0 {
        return (width, height);
    }
    let longest = width.max(height);
    if longest <= edge {
        return (width, height);
    }
    let scale = f64::from(edge) / f64::from(longest);
    let target_width = ((f64::from(width) * scale).round() as u32).max(1);
    let target_height = ((f64::from(height) * scale).round() as u32).max(1);
    (target_width, target_height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageFormat as EncodedFormat, Rgba, RgbaImage};

    fn encode(width: u32, height: u32, format: EncodedFormat) -> Vec<u8> {
        let image = RgbaImage::from_pixel(width, height, Rgba([20, 40, 60, 255]));
        let mut cursor = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut cursor, format)
            .unwrap();
        cursor.into_inner()
    }

    #[test]
    fn thumbnail_size_keeps_aspect_ratio() {
        assert_eq!(fit_within(256, 128, 128), (128, 64));
        assert_eq!(fit_within(100, 300, 128), (43, 128));
        assert_eq!(fit_within(64, 64, 128), (64, 64));
        assert_eq!(fit_within(0, 100, 128), (0, 100));
    }

    #[test]
    fn jpeg_thumbnail_is_scaled_down_and_oriented() {
        let bytes = encode(400, 200, EncodedFormat::Jpeg);
        let decoded = decode_thumbnail(&bytes, ImageFormat::Jpeg, 128).unwrap();
        assert_eq!((decoded.width, decoded.height), (128, 64));
        assert_eq!(decoded.rgba.len(), 128 * 64 * 4);
    }

    #[test]
    fn small_jpeg_is_not_upscaled() {
        let bytes = encode(40, 20, EncodedFormat::Jpeg);
        let decoded = decode_thumbnail(&bytes, ImageFormat::Jpeg, 128).unwrap();
        assert_eq!((decoded.width, decoded.height), (40, 20));
    }

    #[test]
    fn png_thumbnail_falls_back_to_full_decode_then_resize() {
        let bytes = encode(400, 200, EncodedFormat::Png);
        let decoded = decode_thumbnail(&bytes, ImageFormat::Png, 128).unwrap();
        assert_eq!((decoded.width, decoded.height), (128, 64));
    }
}
