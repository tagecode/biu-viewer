//! 平台适配。文件对话框等原生能力。

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::path::PathBuf;

use biuviewer_core::{FileDialogService, ViewerError};

/// 平台调用失败时使用核心错误类型，保证依赖方向指向 `biuviewer-core`。
pub type PlatformError = ViewerError;

/// 与 `biuviewer-decoder::FormatDetector::supported_extensions` 保持一致的对话框过滤器。
const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "tif", "ico",
];

/// 使用系统原生对话框（macOS 的 NSOpenPanel、Windows 的 IFileDialog、Linux 的 XDG Portal）。
#[derive(Debug, Default, Clone, Copy)]
pub struct NativeFileDialog;

impl NativeFileDialog {
    pub const fn new() -> Self {
        Self
    }
}

impl FileDialogService for NativeFileDialog {
    fn pick_image(&self) -> Result<Option<PathBuf>, ViewerError> {
        Ok(rfd::FileDialog::new()
            .set_title("打开图片")
            .add_filter("图片", IMAGE_EXTENSIONS)
            .pick_file())
    }

    fn pick_folder(&self) -> Result<Option<PathBuf>, ViewerError> {
        Ok(rfd::FileDialog::new().set_title("打开文件夹").pick_folder())
    }
}
