//! 顶部工具栏。按钮以文字符号呈现，避免引入图标资源。

use biuviewer_core::ScaleMode;
use gpui_kit::{
    div, px, AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window,
};

use crate::theme;
use crate::tooltip::TooltipText;
use crate::viewer_app::ViewerApp;

impl ViewerApp {
    pub(crate) fn render_toolbar(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let has_image = self.has_image();
        let index = self.current_index();
        let total = self.total_images();
        let zoom = self.zoom_percent();
        let mode = self.current_scale_mode();

        let counter = if total == 0 {
            "—".to_string()
        } else {
            format!("{} / {}", index + 1, total)
        };

        let mode_label = match mode {
            ScaleMode::Fit => "适应",
            ScaleMode::Actual => "1:1",
            ScaleMode::Custom => "自定义",
        };
        let zoom_label = format!("{zoom:.0}% · {mode_label}");

        let fullscreen = self.is_fullscreen();

        let mut bar = div()
            .id("toolbar")
            .flex()
            .flex_row()
            .items_center()
            .gap(px(4.0))
            .w_full()
            .h(px(theme::TOOLBAR_HEIGHT))
            .flex_shrink_0()
            .px(px(8.0))
            .bg(theme::PANEL)
            .border_b(px(1.0))
            .border_color(theme::BORDER);

        bar = bar
            .child(Self::tool_button(
                "tb-open",
                "打开",
                "打开图片 (⌘O)",
                true,
                cx,
                |this, _window, cx| this.pick_image(cx),
            ))
            .child(Self::tool_button(
                "tb-folder",
                "文件夹",
                "打开文件夹 (⇧⌘O)",
                true,
                cx,
                |this, _window, cx| this.pick_folder(cx),
            ))
            .child(Self::divider())
            .child(Self::tool_button(
                "tb-prev",
                "◀",
                "上一张 (←)",
                total > 1,
                cx,
                |this, _window, cx| this.prev_image(cx),
            ))
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(theme::MUTED)
                    .child(counter),
            )
            .child(Self::tool_button(
                "tb-next",
                "▶",
                "下一张 (→)",
                total > 1,
                cx,
                |this, _window, cx| this.next_image(cx),
            ))
            .child(Self::divider())
            .child(Self::tool_button(
                "tb-zoom-out",
                "−",
                "缩小 (⌘-)",
                has_image,
                cx,
                |this, _window, cx| this.zoom_out(cx),
            ))
            .child(
                div()
                    .id("tb-zoom-label")
                    .min_w(px(78.0))
                    .text_size(px(12.0))
                    .text_color(theme::MUTED)
                    .child(zoom_label),
            )
            .child(Self::tool_button(
                "tb-zoom-in",
                "+",
                "放大 (⌘+)",
                has_image,
                cx,
                |this, _window, cx| this.zoom_in(cx),
            ))
            .child(Self::tool_button(
                "tb-fit",
                "适应",
                "适应窗口 (0)",
                has_image,
                cx,
                |this, _window, cx| this.fit_to_window(cx),
            ))
            .child(Self::tool_button(
                "tb-actual",
                "1:1",
                "原始大小",
                has_image,
                cx,
                |this, _window, cx| this.actual_size(cx),
            ))
            .child(Self::divider())
            .child(Self::tool_button(
                "tb-rotate-ccw",
                "↺",
                "逆时针旋转 (⇧R)",
                has_image,
                cx,
                |this, _window, cx| this.rotate_ccw(cx),
            ))
            .child(Self::tool_button(
                "tb-rotate-cw",
                "↻",
                "顺时针旋转 (R)",
                has_image,
                cx,
                |this, _window, cx| this.rotate_cw(cx),
            ))
            .child(Self::tool_button(
                "tb-flip-h",
                "⇋",
                "水平翻转 (H)",
                has_image,
                cx,
                |this, _window, cx| this.flip_horizontal(cx),
            ))
            .child(Self::tool_button(
                "tb-flip-v",
                "⇅",
                "垂直翻转 (V)",
                has_image,
                cx,
                |this, _window, cx| this.flip_vertical(cx),
            ))
            .child(Self::tool_button(
                "tb-reset",
                "重置",
                "重置旋转、翻转与缩放",
                has_image,
                cx,
                |this, _window, cx| this.reset_transform(cx),
            ))
            .child(div().flex_1())
            .child(Self::tool_button(
                "tb-thumbs",
                "缩略图",
                "显示或隐藏缩略图条",
                true,
                cx,
                |this, _window, cx| this.toggle_thumbnails(cx),
            ))
            .child(Self::tool_button(
                "tb-fullscreen",
                if fullscreen { "退出全屏" } else { "全屏" },
                "全屏 (F)",
                true,
                cx,
                |this, window, cx| this.toggle_fullscreen(window, cx),
            ));

        let _ = window;
        bar.into_any_element()
    }

    fn tool_button(
        id: &'static str,
        label: &'static str,
        tip: &'static str,
        enabled: bool,
        cx: &mut Context<Self>,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> AnyElement {
        let text_color = if enabled { theme::TEXT } else { theme::MUTED };
        let mut button = div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .h(px(28.0))
            .px(px(10.0))
            .rounded(px(6.0))
            .text_size(px(12.0))
            .text_color(text_color);

        if enabled {
            button = button
                .cursor_pointer()
                .hover(|style| style.bg(theme::PANEL_HOVER))
                .tooltip(move |_window, cx| TooltipText::view(tip, cx))
                .on_click(cx.listener(move |this, _event: &ClickEvent, window, cx| {
                    on_click(this, window, cx);
                }));
        } else {
            button = button.opacity(0.5);
        }

        button.child(label).into_any_element()
    }

    fn divider() -> AnyElement {
        div()
            .w(px(1.0))
            .h(px(18.0))
            .mx(px(4.0))
            .flex_shrink_0()
            .bg(theme::BORDER)
            .into_any_element()
    }
}
