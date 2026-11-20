//! 平台适配。文件对话框等原生能力。

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use biuviewer_core::{FileDialogFuture, FileDialogService};

/// 平台调用失败时使用核心错误类型，保证依赖方向指向 `biuviewer-core`。
pub type PlatformError = biuviewer_core::ViewerError;

/// 与 `biuviewer-decoder::FormatDetector::supported_extensions` 保持一致的对话框过滤器。
const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "tif", "ico", "heic", "heif", "avif",
];

/// 使用系统原生对话框（macOS 的 NSOpenPanel、Windows 的 IFileDialog、Linux 的 XDG Portal）。
///
/// 走 `rfd` 的异步 API：macOS 上以 sheet 形式挂在主窗口，Windows 上在独立线程弹出，
/// 都不会像同步 API 那样在有窗口的嵌套事件循环里阻塞 GPUI 的事件回调。
#[derive(Debug, Default, Clone, Copy)]
pub struct NativeFileDialog;

impl NativeFileDialog {
    pub const fn new() -> Self {
        Self
    }
}

impl FileDialogService for NativeFileDialog {
    fn pick_image(&self) -> FileDialogFuture {
        Box::pin(async {
            let selected = rfd::AsyncFileDialog::new()
                .set_title("打开图片")
                .add_filter("图片", IMAGE_EXTENSIONS)
                .pick_file()
                .await;
            Ok(selected.map(|handle| handle.path().to_path_buf()))
        })
    }

    fn pick_folder(&self) -> FileDialogFuture {
        Box::pin(async {
            let selected = rfd::AsyncFileDialog::new()
                .set_title("打开文件夹")
                .pick_folder()
                .await;
            Ok(selected.map(|handle| handle.path().to_path_buf()))
        })
    }
}
