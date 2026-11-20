use std::io::Cursor;
use std::time::Duration;

use biuviewer_core::ViewerError;
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, Delay, ImageDecoder};

use crate::decode::{map_image_error, reject_oversized, DecodedImage};

/// 浏览器对过小帧延迟的兜底值。
pub const MIN_FRAME_DELAY: Duration = Duration::from_millis(100);
/// 小于该值的延迟按 `MIN_FRAME_DELAY` 处理。
pub const FRAME_DELAY_THRESHOLD: Duration = Duration::from_millis(20);
/// 单次动画解码允许的最大帧数。
pub const MAX_ANIMATION_FRAMES: usize = 1_000;
/// 动画所有帧解码后的 RGBA 总字节上限（约 256MB）。GPU 纹理还会再占一份，
/// 上限过高时低内存机器仍可能被单张 GIF 拖垮。
pub const MAX_ANIMATION_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedFrame {
    pub rgba: Vec<u8>,
    pub delay: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedAnimation {
    pub width: u32,
    pub height: u32,
    pub frames: Vec<DecodedFrame>,
}

impl DecodedAnimation {
    pub fn is_animated(&self) -> bool {
        self.frames.len() > 1
    }

    pub fn byte_size(&self) -> usize {
        self.frames
            .iter()
            .map(|frame| frame.rgba.len())
            .sum::<usize>()
    }
}

/// 解码 GIF 的全部帧。先校验画布尺寸，再逐帧解码并限制帧数与总字节，
/// 避免在限额生效前把整段动画读进内存。`GifDecoder` 会把每帧合成为完整画布大小的 RGBA。
pub fn decode_gif(bytes: &[u8]) -> Result<DecodedAnimation, ViewerError> {
    decode_gif_with_limits(bytes, MAX_ANIMATION_FRAMES, MAX_ANIMATION_BYTES)
}

fn decode_gif_with_limits(
    bytes: &[u8],
    max_frames: usize,
    max_bytes: usize,
) -> Result<DecodedAnimation, ViewerError> {
    let decoder = GifDecoder::new(Cursor::new(bytes)).map_err(map_image_error)?;
    let (width, height) = decoder.dimensions();
    reject_oversized(width, height)?;

    let frame_bytes = usize::try_from(
        u64::from(width)
            .saturating_mul(u64::from(height))
            .saturating_mul(4),
    )
    .unwrap_or(usize::MAX);
    let frame_limit = max_frames.min(max_bytes / frame_bytes.max(1));
    if frame_limit == 0 {
        return Err(ViewerError::Unsupported(format!(
            "GIF 解码后总大小超出上限（{} MiB）",
            max_bytes / (1024 * 1024)
        )));
    }

    let mut decoded = Vec::new();
    let mut total_bytes = 0usize;
    for (index, frame) in decoder.into_frames().enumerate() {
        if index >= frame_limit {
            return Err(ViewerError::Unsupported(format!(
                "GIF 帧数或总大小超出上限（最多 {frame_limit} 帧）"
            )));
        }
        let frame = frame.map_err(map_image_error)?;
        let (frame_width, frame_height) = frame.buffer().dimensions();
        if (frame_width, frame_height) != (width, height) {
            tracing::warn!("跳过尺寸不一致的 GIF 帧 {index}: {frame_width}×{frame_height}");
            continue;
        }
        let delay = normalize_delay(delay_to_duration(frame.delay()));
        let rgba = frame.into_buffer().into_raw();
        total_bytes = total_bytes.saturating_add(rgba.len());
        if total_bytes > max_bytes {
            return Err(ViewerError::Unsupported(format!(
                "GIF 解码后总大小超出上限（{} MiB）",
                max_bytes / (1024 * 1024)
            )));
        }
        decoded.push(DecodedFrame { rgba, delay });
    }

    if decoded.is_empty() {
        return Err(ViewerError::Corrupted("GIF 没有可用帧".to_string()));
    }

    Ok(DecodedAnimation {
        width,
        height,
        frames: decoded,
    })
}

/// 只解码 GIF 的第一帧，供缩略图使用，避免为了一个 128px 缩略图解完整段动画。
pub fn decode_gif_first_frame(bytes: &[u8]) -> Result<DecodedImage, ViewerError> {
    let decoder = GifDecoder::new(Cursor::new(bytes)).map_err(map_image_error)?;
    let (width, height) = decoder.dimensions();
    reject_oversized(width, height)?;
    let frame = decoder
        .into_frames()
        .next()
        .ok_or_else(|| ViewerError::Corrupted("GIF 不包含任何帧".to_string()))?
        .map_err(map_image_error)?;
    let (width, height) = frame.buffer().dimensions();
    Ok(DecodedImage {
        width,
        height,
        rgba: frame.into_buffer().into_raw(),
    })
}

fn delay_to_duration(delay: Delay) -> Duration {
    let (numerator, denominator) = delay.numer_denom_ms();
    if denominator == 0 {
        return Duration::ZERO;
    }
    Duration::from_millis(u64::from(numerator)) / denominator
}

fn normalize_delay(delay: Duration) -> Duration {
    if delay < FRAME_DELAY_THRESHOLD {
        MIN_FRAME_DELAY
    } else {
        delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::codecs::gif::{GifEncoder, Repeat};
    use image::{Delay, Frame, Rgba, RgbaImage};

    fn encode_gif(frames: &[(RgbaImage, u32)]) -> Vec<u8> {
        let mut output = Vec::new();
        {
            let mut encoder = GifEncoder::new(&mut output);
            encoder.set_repeat(Repeat::Infinite).unwrap();
            for (image, delay_ms) in frames {
                let frame = Frame::from_parts(
                    image.clone(),
                    0,
                    0,
                    Delay::from_numer_denom_ms(*delay_ms, 1),
                );
                encoder.encode_frame(frame).unwrap();
            }
        }
        output
    }

    fn solid(width: u32, height: u32, pixel: [u8; 4]) -> RgbaImage {
        RgbaImage::from_pixel(width, height, Rgba(pixel))
    }

    #[test]
    fn decodes_all_frames_and_delays() {
        let gif = encode_gif(&[
            (solid(2, 2, [255, 0, 0, 255]), 50),
            (solid(2, 2, [0, 255, 0, 255]), 120),
        ]);
        let animation = decode_gif(&gif).unwrap();
        assert_eq!((animation.width, animation.height), (2, 2));
        assert_eq!(animation.frames.len(), 2);
        assert_eq!(animation.frames[0].delay, Duration::from_millis(50));
        assert_eq!(animation.frames[1].delay, Duration::from_millis(120));
        assert!(animation.is_animated());
    }

    #[test]
    fn tiny_delays_are_normalized_to_browser_default() {
        let gif = encode_gif(&[
            (solid(1, 1, [1, 2, 3, 255]), 10),
            (solid(1, 1, [4, 5, 6, 255]), 0),
        ]);
        let animation = decode_gif(&gif).unwrap();
        assert_eq!(animation.frames[0].delay, MIN_FRAME_DELAY);
        assert_eq!(animation.frames[1].delay, MIN_FRAME_DELAY);
    }

    #[test]
    fn single_frame_gif_is_not_animated() {
        let gif = encode_gif(&[(solid(2, 3, [9, 9, 9, 255]), 100)]);
        let animation = decode_gif(&gif).unwrap();
        assert_eq!(animation.frames.len(), 1);
        assert!(!animation.is_animated());
        assert_eq!((animation.width, animation.height), (2, 3));
    }

    #[test]
    fn corrupt_gif_returns_corrupted_without_panicking() {
        let err = decode_gif(b"GIF89a garbage").unwrap_err();
        assert!(matches!(err, ViewerError::Corrupted(_)));
    }

    #[test]
    fn frame_count_limit_is_enforced() {
        let gif = encode_gif(&[
            (solid(2, 2, [1, 0, 0, 255]), 100),
            (solid(2, 2, [2, 0, 0, 255]), 100),
            (solid(2, 2, [3, 0, 0, 255]), 100),
        ]);
        let err = decode_gif_with_limits(&gif, 2, MAX_ANIMATION_BYTES).unwrap_err();
        assert!(
            matches!(err, ViewerError::Unsupported(_)),
            "expected Unsupported, got {err:?}"
        );
    }

    #[test]
    fn byte_limit_is_enforced_before_decoding_frames() {
        let gif = encode_gif(&[(solid(2, 2, [1, 2, 3, 255]), 100)]);
        // 2×2 RGBA 每帧 16 字节，上限 8 字节时一帧都放不下。
        let err = decode_gif_with_limits(&gif, MAX_ANIMATION_FRAMES, 8).unwrap_err();
        assert!(
            matches!(err, ViewerError::Unsupported(_)),
            "expected Unsupported, got {err:?}"
        );
    }

    #[test]
    fn oversized_canvas_is_rejected_before_frames_are_decoded() {
        let gif = encode_gif(&[(solid(40_000, 1, [0, 0, 0, 255]), 100)]);
        let err = decode_gif(&gif).unwrap_err();
        assert!(
            err.user_message().contains("尺寸超出上限"),
            "expected size error, got {err:?}"
        );
    }

    #[test]
    fn first_frame_decoder_returns_only_the_first_frame() {
        let gif = encode_gif(&[
            (solid(2, 2, [255, 0, 0, 255]), 50),
            (solid(2, 2, [0, 255, 0, 255]), 50),
        ]);
        let image = decode_gif_first_frame(&gif).unwrap();
        assert_eq!((image.width, image.height), (2, 2));
        let pixel = image.pixel(0, 0);
        assert!(pixel[0] > 200 && pixel[1] < 60, "unexpected {pixel:?}");
    }
}
