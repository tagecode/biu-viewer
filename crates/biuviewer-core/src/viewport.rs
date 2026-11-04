/// 宽度与高度，单位为逻辑像素。图像尺寸与视口尺寸共用此类型。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Extent {
    pub width: f32,
    pub height: f32,
}

impl Extent {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// 相对视口中心的平移量，向右/向下为正。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Offset {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// 缩放模式。Fit 模式随窗口尺寸自动重算，Actual 固定为物理像素 1:1，Custom 为用户缩放。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScaleMode {
    #[default]
    Fit,
    Actual,
    Custom,
}

pub const MIN_SCALE: f32 = 0.01;
pub const MAX_SCALE: f32 = 32.0;

/// 图片在视口中的缩放与平移。图片始终以视口中心为基准，再叠加 `offset`。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub mode: ScaleMode,
    pub scale: f32,
    pub offset: Offset,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            mode: ScaleMode::Fit,
            scale: 1.0,
            offset: Offset::default(),
        }
    }
}

impl Viewport {
    /// Fit 模式：完整显示且小图不放大。
    pub fn fit(image: Extent, viewport: Extent) -> Self {
        Self {
            mode: ScaleMode::Fit,
            scale: fit_scale(image, viewport),
            offset: Offset::default(),
        }
    }

    /// 1:1 原始尺寸。`dpi` 为窗口缩放因子，使一个图像像素对应一个物理像素。
    pub fn actual(image: Extent, viewport: Extent, dpi: f32) -> Self {
        let scale = clamp_scale(1.0 / dpi.max(0.01));
        Self {
            mode: ScaleMode::Actual,
            scale,
            offset: clamp_offset(scale, image, viewport, Offset::default()),
        }
    }

    /// 以视口坐标中的 `cursor` 为锚点缩放。`factor` 大于 1 为放大。
    pub fn zoomed_at(self, image: Extent, viewport: Extent, factor: f32, cursor: Offset) -> Self {
        let new_scale = clamp_scale(self.scale * factor);
        if (new_scale - self.scale).abs() < f32::EPSILON {
            return self;
        }
        let center = Offset::new(viewport.width / 2.0, viewport.height / 2.0);
        let cursor_from_center = Offset::new(cursor.x - center.x, cursor.y - center.y);
        let ratio = new_scale / self.scale;
        let offset = Offset::new(
            cursor_from_center.x + (self.offset.x - cursor_from_center.x) * ratio,
            cursor_from_center.y + (self.offset.y - cursor_from_center.y) * ratio,
        );
        Self {
            mode: ScaleMode::Custom,
            scale: new_scale,
            offset: clamp_offset(new_scale, image, viewport, offset),
        }
    }

    /// 以视口中心为锚点缩放。
    pub fn zoom_by(self, image: Extent, viewport: Extent, factor: f32) -> Self {
        let center = Offset::new(viewport.width / 2.0, viewport.height / 2.0);
        self.zoomed_at(image, viewport, factor, center)
    }

    pub fn panned(self, image: Extent, viewport: Extent, delta: Offset) -> Self {
        let offset = Offset::new(self.offset.x + delta.x, self.offset.y + delta.y);
        Self {
            offset: clamp_offset(self.scale, image, viewport, offset),
            ..self
        }
    }

    /// 窗口尺寸或图像尺寸变化后重算。Fit 模式重新适配，其它模式只做边界约束。
    pub fn recomputed(self, image: Extent, viewport: Extent) -> Self {
        match self.mode {
            ScaleMode::Fit => Self::fit(image, viewport),
            _ => Self {
                offset: clamp_offset(self.scale, image, viewport, self.offset),
                ..self
            },
        }
    }

    /// 图片在视口中的显示尺寸。
    pub fn display_size(self, image: Extent) -> Extent {
        Extent::new(image.width * self.scale, image.height * self.scale)
    }

    /// 图片左上角相对视口左上角的位置。
    pub fn image_origin(self, image: Extent, viewport: Extent) -> Offset {
        let display = self.display_size(image);
        Offset::new(
            (viewport.width - display.width) / 2.0 + self.offset.x,
            (viewport.height - display.height) / 2.0 + self.offset.y,
        )
    }

    /// 可拖拽时鼠标应显示抓手。任一方向图片大于视口即可拖拽。
    pub fn is_pannable(self, image: Extent, viewport: Extent) -> bool {
        let display = self.display_size(image);
        display.width > viewport.width + 0.5 || display.height > viewport.height + 0.5
    }
}

/// Fit 比例：铺满视口但不放大超过 1:1。
pub fn fit_scale(image: Extent, viewport: Extent) -> f32 {
    if image.width <= 0.0 || image.height <= 0.0 || viewport.width <= 0.0 || viewport.height <= 0.0
    {
        return 1.0;
    }
    let scale = (viewport.width / image.width).min(viewport.height / image.height);
    clamp_scale(scale.min(1.0))
}

pub fn clamp_scale(scale: f32) -> f32 {
    scale.clamp(MIN_SCALE, MAX_SCALE)
}

/// 单个方向的平移约束：图片小于视口时居中，否则不允许拖出空白。
fn clamp_axis(offset: f32, display: f32, viewport: f32) -> f32 {
    if display <= viewport {
        return 0.0;
    }
    let limit = (display - viewport) / 2.0;
    offset.clamp(-limit, limit)
}

pub fn clamp_offset(scale: f32, image: Extent, viewport: Extent, offset: Offset) -> Offset {
    Offset::new(
        clamp_axis(offset.x, image.width * scale, viewport.width),
        clamp_axis(offset.y, image.height * scale, viewport.height),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extent(width: f32, height: f32) -> Extent {
        Extent::new(width, height)
    }

    fn assert_close(left: f32, right: f32) {
        assert!((left - right).abs() < 0.01, "expected {right}, got {left}");
    }

    #[test]
    fn fit_does_not_upscale_small_images() {
        let viewport = Viewport::fit(extent(100.0, 50.0), extent(800.0, 600.0));
        assert_eq!(viewport.scale, 1.0);
        assert_eq!(viewport.offset, Offset::default());
    }

    #[test]
    fn fit_uses_the_smaller_ratio() {
        let viewport = Viewport::fit(extent(400.0, 200.0), extent(200.0, 200.0));
        assert_close(viewport.scale, 0.5);
        let display = viewport.display_size(extent(400.0, 200.0));
        assert_close(display.width, 200.0);
        assert_close(display.height, 100.0);
    }

    #[test]
    fn actual_size_accounts_for_dpi() {
        let viewport = Viewport::actual(extent(100.0, 100.0), extent(800.0, 600.0), 2.0);
        assert_eq!(viewport.scale, 0.5);
        assert_eq!(viewport.mode, ScaleMode::Actual);
    }

    #[test]
    fn zoom_keeps_the_pixel_under_the_cursor() {
        let image = extent(1000.0, 800.0);
        let viewport_size = extent(500.0, 400.0);
        let start = Viewport::fit(image, viewport_size);
        let cursor = Offset::new(400.0, 100.0);

        let image_point_under_cursor = |viewport: Viewport| {
            let origin = viewport.image_origin(image, viewport_size);
            Offset::new(
                (cursor.x - origin.x) / viewport.scale,
                (cursor.y - origin.y) / viewport.scale,
            )
        };

        let before = image_point_under_cursor(start);
        let zoomed = start.zoomed_at(image, viewport_size, 3.0, cursor);
        let after = image_point_under_cursor(zoomed);
        assert_close(before.x, after.x);
        assert_close(before.y, after.y);
    }

    #[test]
    fn zoom_is_clamped_to_the_supported_range() {
        let image = extent(100.0, 100.0);
        let viewport_size = extent(100.0, 100.0);
        let mut viewport = Viewport::fit(image, viewport_size);
        for _ in 0..100 {
            viewport = viewport.zoom_by(image, viewport_size, 2.0);
        }
        assert_eq!(viewport.scale, MAX_SCALE);
        for _ in 0..200 {
            viewport = viewport.zoom_by(image, viewport_size, 0.5);
        }
        assert_eq!(viewport.scale, MIN_SCALE);
    }

    #[test]
    fn panning_is_clamped_to_image_edges() {
        let image = extent(1000.0, 1000.0);
        let viewport_size = extent(500.0, 500.0);
        let viewport = Viewport::actual(image, viewport_size, 1.0).panned(
            image,
            viewport_size,
            Offset::new(10_000.0, -10_000.0),
        );
        assert_eq!(viewport.offset, Offset::new(250.0, -250.0));
    }

    #[test]
    fn small_images_cannot_be_panned() {
        let image = extent(100.0, 100.0);
        let viewport_size = extent(500.0, 500.0);
        let viewport = Viewport::fit(image, viewport_size).panned(
            image,
            viewport_size,
            Offset::new(50.0, 50.0),
        );
        assert_eq!(viewport.offset, Offset::default());
        assert!(!viewport.is_pannable(image, viewport_size));
    }

    #[test]
    fn fit_recomputes_when_the_viewport_changes() {
        let image = extent(400.0, 200.0);
        let viewport = Viewport::fit(image, extent(200.0, 200.0));
        assert_close(viewport.scale, 0.5);
        let resized = viewport.recomputed(image, extent(800.0, 800.0));
        assert_eq!(resized.scale, 1.0);
    }

    #[test]
    fn rotated_dimensions_are_used_for_fit() {
        let image = extent(200.0, 400.0);
        let viewport = Viewport::fit(extent(image.height, image.width), extent(400.0, 400.0));
        assert_close(viewport.scale, 1.0);
        assert_close(viewport.display_size(extent(400.0, 200.0)).width, 400.0);
    }
}
