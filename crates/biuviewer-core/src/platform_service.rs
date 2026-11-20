use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use crate::ViewerError;

/// 原生对话框尚未返回时的 future。界面层在主线程上轮询它，因此不要求 `Send`：
/// macOS 的 NSOpenPanel 必须在主线程创建。
pub type FileDialogFuture =
    Pin<Box<dyn Future<Output = Result<Option<PathBuf>, ViewerError>> + 'static>>;

/// 界面层调用原生文件对话框的接口。实现放在 `biuviewer-platform`，由主程序注入。
///
/// 接口返回未开始的 future 而不是直接返回路径：原生对话框（macOS 的模态面板、
/// Windows 的 IFileDialog）不能阻塞在 GPUI 的事件回调里，否则会在 `App` 借用未
/// 释放时重入应用状态。
pub trait FileDialogService: Send + Sync {
    fn pick_image(&self) -> FileDialogFuture;
    fn pick_folder(&self) -> FileDialogFuture;
}

/// 测试或无声启动时使用，永远返回“用户取消”。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopFileDialog;

impl FileDialogService for NoopFileDialog {
    fn pick_image(&self) -> FileDialogFuture {
        Box::pin(async { Ok(None) })
    }

    fn pick_folder(&self) -> FileDialogFuture {
        Box::pin(async { Ok(None) })
    }
}
