//! 文件解码到 GPU 纹理的桥接层，只依赖解码器与 GPUI，不接触窗口状态。

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use biuviewer_core::{TransformState, ViewerError};
use biuviewer_decoder::{decode_gif, decode_static, FormatDetector, ImageFormat};
use gpui_kit::RenderImage;
use image::imageops::FilterType;
use image::{Delay, Frame, RgbaImage};

use crate::theme::THUMBNAIL_EDGE;

/// 静态图片与变换后的纹理都只有一帧；动画驱动只读取原始 `LoadedImage` 的帧延迟。
const PLACEHOLDER_DELAY: Duration = Duration::from_millis(100);

/// 后台线程解码完成的结果。`bytes` 按 `宽 × 高 × 4 × 帧数` 估算，用于 LRU 预算。
pub struct LoadedImage {
    pub width: u32,
    pub height: u32,
    pub frames: usize,
    pub image: Arc<RenderImage>,
    pub bytes: usize,
}

impl LoadedImage {
    pub fn is_animated(&self) -> bool {
        self.frames > 1
    }
}

/// 读取并解码文件。GIF 解码为多帧，其余格式解码为单帧。
pub fn load_image(path: &Path) -> Result<LoadedImage, ViewerError> {
    let raw = std::fs::read(path)?;
    let extension = path.extension().and_then(|ext| ext.to_str());
    let format = FormatDetector::detect_with_extension(&raw, extension)?;

    if format == ImageFormat::Gif {
        let animation = decode_gif(&raw)?;
        let frames = animation.frames.len();
        let bytes = animation.byte_size();
        let mut render_frames = Vec::with_capacity(frames);
        for frame in animation.frames {
            let mut bgra = frame.rgba;
            swap_red_blue(&mut bgra);
            let render_frame = into_frame(animation.width, animation.height, bgra, frame.delay)
                .ok_or_else(|| ViewerError::Corrupted(path.display().to_string()))?;
            render_frames.push(render_frame);
        }
        return Ok(LoadedImage {
            width: animation.width,
            height: animation.height,
            frames,
            image: Arc::new(RenderImage::new(render_frames)),
            bytes,
        });
    }

    let decoded = decode_static(&raw)?;
    let bytes = decoded.rgba.len();
    let mut bgra = decoded.rgba;
    swap_red_blue(&mut bgra);
    let frame = into_frame(decoded.width, decoded.height, bgra, PLACEHOLDER_DELAY)
        .ok_or_else(|| ViewerError::Corrupted(path.display().to_string()))?;
    Ok(LoadedImage {
        width: decoded.width,
        height: decoded.height,
        frames: 1,
        image: Arc::new(RenderImage::new(vec![frame])),
        bytes,
    })
}

/// 解码发生在后台线程，第三方解码器理论上可能 panic。捕获后转成错误卡片，主程序不退出。
pub fn load_image_catching_panics(path: &Path) -> Result<LoadedImage, ViewerError> {
    match catch_unwind(AssertUnwindSafe(|| load_image(path))) {
        Ok(result) => result,
        Err(payload) => {
            let detail = panic_message(&payload);
            tracing::error!("解码 {} 时发生 panic: {detail}", path.display());
            Err(ViewerError::Corrupted(path.display().to_string()))
        }
    }
}

/// 生成缩略图纹理：长边不超过 [`THUMBNAIL_EDGE`]，只保留第一帧。
pub fn load_thumbnail(path: &Path) -> Result<Arc<RenderImage>, ViewerError> {
    let raw = std::fs::read(path)?;
    let extension = path.extension().and_then(|ext| ext.to_str());
    let format = FormatDetector::detect_with_extension(&raw, extension)?;

    let (width, height, rgba) = if format == ImageFormat::Gif {
        let animation = decode_gif(&raw)?;
        let frame = animation
            .frames
            .into_iter()
            .next()
            .ok_or_else(|| ViewerError::Corrupted(path.display().to_string()))?;
        (animation.width, animation.height, frame.rgba)
    } else {
        let decoded = decode_static(&raw)?;
        (decoded.width, decoded.height, decoded.rgba)
    };

    let source = RgbaImage::from_raw(width, height, rgba)
        .ok_or_else(|| ViewerError::Corrupted(path.display().to_string()))?;
    let (target_width, target_height) = fit_within(width, height, THUMBNAIL_EDGE);
    let resized = if (target_width, target_height) == (width, height) {
        source
    } else {
        image::imageops::resize(&source, target_width, target_height, FilterType::Triangle)
    };

    let mut bgra = resized.into_raw();
    swap_red_blue(&mut bgra);
    let frame = into_frame(target_width, target_height, bgra, PLACEHOLDER_DELAY)
        .ok_or_else(|| ViewerError::Corrupted(path.display().to_string()))?;
    Ok(Arc::new(RenderImage::new(vec![frame])))
}

pub fn load_thumbnail_catching_panics(path: &Path) -> Result<Arc<RenderImage>, ViewerError> {
    match catch_unwind(AssertUnwindSafe(|| load_thumbnail(path))) {
        Ok(result) => result,
        Err(payload) => {
            let detail = panic_message(&payload);
            tracing::error!("生成缩略图 {} 时发生 panic: {detail}", path.display());
            Err(ViewerError::Corrupted(path.display().to_string()))
        }
    }
}

/// 把用户的旋转/翻转叠加到已解码的 BGRA 缓冲上，不重新读取文件。
pub fn transformed_frame(
    source: &Arc<RenderImage>,
    frame_index: usize,
    transform: TransformState,
) -> Option<Arc<RenderImage>> {
    if transform.is_identity() {
        return Some(source.clone());
    }
    let size = source.size(frame_index);
    let width = size.width.0.max(0) as u32;
    let height = size.height.0.max(0) as u32;
    let bytes = source.as_bytes(frame_index)?;
    let (width, height, bgra) = transform.apply_to_rgba(width, height, bytes)?;
    let frame = into_frame(width, height, bgra, PLACEHOLDER_DELAY)?;
    Some(Arc::new(RenderImage::new(vec![frame])))
}

/// 从 `file://` URL 或普通路径生成路径列表。macOS 的“打开方式”会传入前者。
pub fn paths_from_urls(urls: &[String]) -> Vec<PathBuf> {
    urls.iter().filter_map(|url| path_from_url(url)).collect()
}

fn path_from_url(url: &str) -> Option<PathBuf> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }
    let raw = trimmed.strip_prefix("file://").unwrap_or(trimmed);
    let decoded = percent_decode(raw);
    if decoded.is_empty() {
        return None;
    }
    // file:///C:/... 在 Windows 上多一个前导斜杠。
    let decoded = if decoded.len() >= 3 && decoded.starts_with('/') && decoded.as_bytes()[2] == b':'
    {
        decoded[1..].to_string()
    } else {
        decoded
    };
    Some(PathBuf::from(decoded))
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let high = hex_value(bytes[index + 1]);
            let low = hex_value(bytes[index + 2]);
            if let (Some(high), Some(low)) = (high, low) {
                output.push(high * 16 + low);
                index += 3;
                continue;
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn swap_red_blue(bytes: &mut [u8]) {
    for pixel in bytes.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
    }
}

/// GPUI 的 `RenderImage` 按 BGRA 上传，因此 `RgbaImage` 里装的其实是交换过 R/B 的字节。
fn into_frame(width: u32, height: u32, bgra: Vec<u8>, delay: Duration) -> Option<Frame> {
    let buffer = RgbaImage::from_raw(width, height, bgra)?;
    let millis = u32::try_from(delay.as_millis()).unwrap_or(u32::MAX);
    Some(Frame::from_parts(
        buffer,
        0,
        0,
        Delay::from_numer_denom_ms(millis, 1),
    ))
}

fn fit_within(width: u32, height: u32, edge: u32) -> (u32, u32) {
    if width == 0 || height == 0 {
        return (width, height);
    }
    let longest = width.max(height);
    if longest <= edge {
        return (width, height);
    }
    let scale = edge as f64 / longest as f64;
    let target_width = ((width as f64 * scale).round() as u32).max(1);
    let target_height = ((height as f64 * scale).round() as u32).max(1);
    (target_width, target_height)
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "未知 panic".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbnail_size_keeps_aspect_ratio() {
        assert_eq!(fit_within(256, 128, 128), (128, 64));
        assert_eq!(fit_within(100, 300, 128), (43, 128));
        assert_eq!(fit_within(64, 64, 128), (64, 64));
        assert_eq!(fit_within(0, 100, 128), (0, 100));
    }

    #[test]
    fn file_urls_are_decoded() {
        let paths = paths_from_urls(&[
            "file:///tmp/a%20b.png".to_string(),
            "/tmp/plain.png".to_string(),
            "".to_string(),
        ]);
        assert_eq!(
            paths,
            vec![
                PathBuf::from("/tmp/a b.png"),
                PathBuf::from("/tmp/plain.png")
            ]
        );
    }

    #[test]
    fn red_blue_channels_are_swapped_in_four_byte_pixels() {
        let mut bytes = vec![1, 2, 3, 4, 5, 6, 7, 8];
        swap_red_blue(&mut bytes);
        assert_eq!(bytes, vec![3, 2, 1, 4, 7, 6, 5, 8]);
    }
}
