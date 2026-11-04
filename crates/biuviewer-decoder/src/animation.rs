use std::io::Cursor;
use std::time::Duration;

use biuviewer_core::ViewerError;
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, Delay};

use crate::decode::{map_image_error, MAX_EDGE, MAX_PIXELS};

/// 浏览器对过小帧延迟的兜底值。
pub const MIN_FRAME_DELAY: Duration = Duration::from_millis(100);
/// 小于该值的延迟按 `MIN_FRAME_DELAY` 处理。
pub const FRAME_DELAY_THRESHOLD: Duration = Duration::from_millis(20);

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

/// 解码 GIF 的全部帧。`GifDecoder` 会把每帧合成为完整画布大小的 RGBA。
pub fn decode_gif(bytes: &[u8]) -> Result<DecodedAnimation, ViewerError> {
    let decoder = GifDecoder::new(Cursor::new(bytes)).map_err(map_image_error)?;
    let frames = decoder
        .into_frames()
        .collect_frames()
        .map_err(map_image_error)?;
    let first = frames
        .first()
        .ok_or_else(|| ViewerError::Corrupted("GIF 不包含任何帧".to_string()))?;
    let (width, height) = first.buffer().dimensions();
    reject_oversized(width, height)?;

    let mut decoded = Vec::with_capacity(frames.len());
    for (index, frame) in frames.into_iter().enumerate() {
        let (frame_width, frame_height) = frame.buffer().dimensions();
        if (frame_width, frame_height) != (width, height) {
            tracing::warn!("跳过尺寸不一致的 GIF 帧 {index}: {frame_width}×{frame_height}");
            continue;
        }
        let delay = delay_to_duration(frame.delay());
        decoded.push(DecodedFrame {
            rgba: frame.into_buffer().into_raw(),
            delay: normalize_delay(delay),
        });
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

fn reject_oversized(width: u32, height: u32) -> Result<(), ViewerError> {
    let pixels = u64::from(width).saturating_mul(u64::from(height));
    if width > MAX_EDGE || height > MAX_EDGE || pixels > MAX_PIXELS {
        return Err(ViewerError::Unsupported(format!(
            "图片尺寸超出上限（{width}×{height}）"
        )));
    }
    Ok(())
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
}
