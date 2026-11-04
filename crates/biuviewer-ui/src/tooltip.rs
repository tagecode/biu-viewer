//! 统一的悬浮提示样式。

use gpui_kit::{
    div, px, AnyView, App, AppContext, Context, IntoElement, ParentElement, Render, Styled, Window,
};

use crate::theme;

pub struct TooltipText {
    text: String,
}

impl TooltipText {
    pub fn view(text: impl Into<String>, cx: &mut App) -> AnyView {
        cx.new(|_| TooltipText { text: text.into() }).into()
    }
}

impl Render for TooltipText {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(4.0))
            .bg(theme::PANEL)
            .border_1()
            .border_color(theme::BORDER)
            .text_size(px(12.0))
            .text_color(theme::TEXT)
            .child(self.text.clone())
    }
}
