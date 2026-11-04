//! 底部状态栏：文件名、尺寸与帧信息。

use gpui_kit::{div, px, AnyElement, IntoElement, ParentElement, Styled};

use crate::theme;
use crate::viewer_app::ViewerApp;

impl ViewerApp {
    pub(crate) fn render_status_bar(&self) -> AnyElement {
        let file_name = self
            .current_file_name()
            .unwrap_or_else(|| "未打开文件".to_string());
        let dimensions = self
            .image_dimensions()
            .map(|(width, height)| format!("{width} × {height}"))
            .unwrap_or_else(|| "—".to_string());
        let frames = self
            .frame_info()
            .filter(|(_, total)| *total > 1)
            .map(|(index, total)| format!("第 {} / {} 帧", index + 1, total));
        let zoom = if self.has_image() {
            format!("{:.0}%", self.zoom_percent())
        } else {
            "—".to_string()
        };

        let mut right = div()
            .flex()
            .items_center()
            .gap(px(12.0))
            .child(dimensions)
            .child(zoom);
        if let Some(frames) = frames {
            right = right.child(frames);
        }

        div()
            .flex()
            .items_center()
            .justify_between()
            .flex_shrink_0()
            .w_full()
            .h(px(theme::STATUS_HEIGHT))
            .px(px(10.0))
            .bg(theme::PANEL)
            .border_t(px(1.0))
            .border_color(theme::BORDER)
            .text_size(px(11.0))
            .text_color(theme::MUTED)
            .child(div().overflow_hidden().child(file_name))
            .child(right)
            .into_any_element()
    }
}
