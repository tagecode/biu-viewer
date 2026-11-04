//! 底部缩略图条。只渲染可视范围内的项，避免大目录一次性建树。

use gpui_kit::{
    div, img, px, AnyElement, ClickEvent, Context, ElementId, InteractiveElement, IntoElement,
    ObjectFit, ParentElement, StatefulInteractiveElement, Styled, StyledImage,
};

use crate::theme::{self, THUMBNAIL_BOX, THUMBNAIL_GAP, THUMBNAIL_STRIP_HEIGHT};
use crate::viewer_app::ViewerApp;

impl ViewerApp {
    pub(crate) fn render_thumbnail_strip(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let (first, last) = self.visible_thumbnail_range();
        let total = self.total_images();
        let current = self.current_index();
        let item = THUMBNAIL_BOX + THUMBNAIL_GAP;

        let mut row = div()
            .id("thumb-row")
            .flex()
            .flex_row()
            .items_center()
            .h_full()
            .px(px(THUMBNAIL_GAP));

        if first > 0 {
            row = row.child(div().w(px(first as f32 * item)).h_full().flex_shrink_0());
        }

        for index in first..last {
            let thumbnail = self.thumbnail_image(index);
            let failed = self.thumbnail_failed(index);
            let label = self
                .thumbnail_label(index)
                .unwrap_or_else(|| "…".to_string());
            let selected = index == current;

            let mut cell = div()
                .id(ElementId::named_usize("thumb", index))
                .flex()
                .items_center()
                .justify_center()
                .w(px(THUMBNAIL_BOX))
                .h(px(THUMBNAIL_BOX))
                .mr(px(THUMBNAIL_GAP))
                .flex_shrink_0()
                .rounded(px(4.0))
                .overflow_hidden()
                .border_1()
                .border_color(if selected {
                    theme::ACCENT
                } else {
                    theme::BORDER
                })
                .bg(theme::VIEWPORT)
                .cursor_pointer()
                .on_click(cx.listener(move |this, _event: &ClickEvent, _window, cx| {
                    this.jump_to(index, cx);
                }));

            cell = match thumbnail {
                Some(image) => {
                    cell.child(img(image).object_fit(ObjectFit::Contain).w_full().h_full())
                }
                None => cell.child(
                    div()
                        .text_size(px(11.0))
                        .text_color(if failed { theme::ERROR } else { theme::MUTED })
                        .child(if failed { "!".to_string() } else { label }),
                ),
            };
            row = row.child(cell);
        }

        if last < total {
            row = row.child(
                div()
                    .w(px((total - last) as f32 * item))
                    .h_full()
                    .flex_shrink_0(),
            );
        }

        div()
            .id("thumb-strip")
            .flex()
            .h(px(THUMBNAIL_STRIP_HEIGHT))
            .flex_shrink_0()
            .overflow_x_scroll()
            .bg(theme::PANEL)
            .border_t(px(1.0))
            .border_color(theme::BORDER)
            .track_scroll(self.thumb_scroll_handle())
            .child(row)
            .into_any_element()
    }
}
