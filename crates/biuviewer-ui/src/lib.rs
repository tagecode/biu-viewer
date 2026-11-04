//! 界面层。只依赖核心与解码器，不依赖主程序。

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod empty_state;
mod image_payload;
mod status_bar;
mod theme;
mod thumbnail_strip;
mod toolbar;
mod tooltip;
mod viewer_app;

/// 空状态文案。窗口尚未打开图片时显示。
pub const EMPTY_STATE_LABEL: &str = "打开图片或拖入文件夹";

pub use empty_state::EmptyState;
pub use image_payload::paths_from_urls;
pub use viewer_app::ViewerApp;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_state_label_matches_mvp() {
        assert_eq!(EMPTY_STATE_LABEL, "打开图片或拖入文件夹");
    }

    #[test]
    fn ui_can_name_decoder_errors() {
        fn _assert_decode_error(
            err: biuviewer_decoder::DecodeError,
        ) -> biuviewer_core::ViewerError {
            err
        }
    }
}
