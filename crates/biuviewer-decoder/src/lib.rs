//! 图像解码。格式识别与像素解码。

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod animation;
mod decode;
mod exif;
mod folder;
mod format;

/// 解码失败时使用核心错误类型，保证依赖方向指向 `biuviewer-core`。
pub type DecodeError = biuviewer_core::ViewerError;

pub use animation::{decode_gif, DecodedAnimation, DecodedFrame};
pub use decode::{
    decode_static, DecodedImage, DecoderBackend, ImageCrateDecoder, MAX_EDGE, MAX_PIXELS,
};
pub use exif::{apply_exif_orientation, orientation_value_from_exif};
pub use folder::{is_supported_path, natural_cmp, FolderScanner};
pub use format::{FormatDetector, ImageFormat};
