use gpui_kit::{div, px, rgb, IntoElement, ParentElement, Render, Styled, Window};

use crate::EMPTY_STATE_LABEL;

/// 尚未打开图片时的占位页。
#[derive(Default)]
pub struct EmptyState;

impl EmptyState {
    pub fn new() -> Self {
        Self
    }
}

impl Render for EmptyState {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut gpui_kit::Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .bg(rgb(0x1c1c1c))
            .text_color(rgb(0xc8c8c8))
            .text_size(px(16.0))
            .child(EMPTY_STATE_LABEL)
    }
}
