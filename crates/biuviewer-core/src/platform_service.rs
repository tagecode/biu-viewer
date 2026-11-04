use std::path::PathBuf;

use crate::ViewerError;

/// 界面层调用原生文件对话框的接口。实现放在 `biuviewer-platform`，由主程序注入。
pub trait FileDialogService: Send + Sync {
    fn pick_image(&self) -> Result<Option<PathBuf>, ViewerError>;
    fn pick_folder(&self) -> Result<Option<PathBuf>, ViewerError>;
}

/// 测试或无声启动时使用，永远返回“用户取消”。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopFileDialog;

impl FileDialogService for NoopFileDialog {
    fn pick_image(&self) -> Result<Option<PathBuf>, ViewerError> {
        Ok(None)
    }

    fn pick_folder(&self) -> Result<Option<PathBuf>, ViewerError> {
        Ok(None)
    }
}
