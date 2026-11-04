use std::io::ErrorKind;

/// 面向用户的查看器错误。每种变体的 `Display` 都是中文说明。
#[derive(Debug, thiserror::Error)]
pub enum ViewerError {
    #[error("找不到文件或目录：{0}")]
    NotFound(String),
    #[error("没有权限访问：{0}")]
    PermissionDenied(String),
    #[error("不支持的格式：{0}")]
    Unsupported(String),
    #[error("文件已损坏，无法读取：{0}")]
    Corrupted(String),
    #[error("读取失败：{0}")]
    Io(String),
}

impl ViewerError {
    /// 返回可直接展示在界面上的中文说明。
    pub fn user_message(&self) -> String {
        self.to_string()
    }
}

impl From<std::io::Error> for ViewerError {
    fn from(err: std::io::Error) -> Self {
        let detail = err.to_string();
        match err.kind() {
            ErrorKind::NotFound => Self::NotFound(detail),
            ErrorKind::PermissionDenied => Self::PermissionDenied(detail),
            _ => Self::Io(detail),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Error;

    #[test]
    fn each_variant_has_a_chinese_user_message() {
        let cases = [
            ViewerError::NotFound("a.jpg".into()),
            ViewerError::PermissionDenied("secret".into()),
            ViewerError::Unsupported("notes.txt".into()),
            ViewerError::Corrupted("broken.png".into()),
            ViewerError::Io("disk".into()),
        ];
        let expected = ["找不到", "没有权限", "不支持", "损坏", "读取失败"];
        for (err, needle) in cases.into_iter().zip(expected) {
            let message = err.user_message();
            assert!(
                message.contains(needle),
                "message {message:?} should contain {needle}"
            );
        }
    }

    #[test]
    fn io_not_found_maps_to_not_found() {
        let err = ViewerError::from(Error::new(ErrorKind::NotFound, "missing"));
        assert!(matches!(err, ViewerError::NotFound(_)));
        assert!(err.user_message().contains("找不到"));
    }

    #[test]
    fn io_permission_denied_maps_to_permission_denied() {
        let err = ViewerError::from(Error::new(ErrorKind::PermissionDenied, "denied"));
        assert!(matches!(err, ViewerError::PermissionDenied(_)));
        assert!(err.user_message().contains("没有权限"));
    }

    #[test]
    fn other_io_errors_map_to_io() {
        let err = ViewerError::from(Error::new(ErrorKind::UnexpectedEof, "eof"));
        assert!(matches!(err, ViewerError::Io(_)));
    }
}
