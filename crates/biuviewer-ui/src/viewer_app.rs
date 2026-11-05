//! 主视图：负责图片加载、视口交互、动画与窗口级事件。

use std::cell::Cell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use biuviewer_core::{
    Extent, FileDialogService, ImageList, MemoryCache, Offset, ScaleMode, TransformState,
    ViewerError, Viewport,
};
use biuviewer_decoder::{is_supported_path, FolderScanner};
use gpui_kit::{
    canvas, div, point, px, rgba, size, AnyElement, Bounds, ClickEvent, Context, CursorStyle,
    DragMoveEvent, ExternalPaths, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Point,
    Render, RenderImage, ScrollDelta, ScrollEvent, ScrollHandle, StatefulInteractiveElement,
    Styled, Task, Window,
};

use crate::image_payload::{
    load_image_catching_panics, load_thumbnail_catching_panics, transformed_frame, LoadedImage,
};
use crate::theme::{
    self, STATUS_HEIGHT, THUMBNAIL_BOX, THUMBNAIL_GAP, THUMBNAIL_STRIP_HEIGHT, TOOLBAR_HEIGHT,
};

/// 解码超过该时长才显示加载指示器，避免小图闪烁。
const LOADING_DELAY: Duration = Duration::from_millis(150);
const CACHE_BUDGET: usize = 512 * 1024 * 1024;
const THUMBNAIL_CACHE_BUDGET: usize = 64 * 1024 * 1024;
const THUMBNAIL_CONCURRENCY: usize = 24;
const OPEN_URLS_INTERVAL: Duration = Duration::from_millis(200);
const ZOOM_STEP: f32 = 1.25;
const WHEEL_ZOOM_SENSITIVITY: f32 = 0.0015;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoadState {
    Idle,
    Loading,
    Ready,
    Failed,
}

/// 当前真正绘制到窗口上的纹理。变换后的图片只有一帧。
pub(crate) struct Display {
    pub(crate) image: Arc<RenderImage>,
    pub(crate) frame: usize,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

pub struct ViewerApp {
    dialog: Arc<dyn FileDialogService>,
    image_list: ImageList,
    view: Viewport,
    transform: TransformState,
    loaded: Option<Arc<LoadedImage>>,
    display: Option<Display>,
    pending_texture_drops: Vec<Arc<RenderImage>>,
    cache: MemoryCache<LoadedImage>,
    thumb_cache: MemoryCache<RenderImage>,
    thumb_in_flight: HashSet<PathBuf>,
    thumb_failed: HashSet<PathBuf>,
    thumb_scroll: ScrollHandle,
    load_state: LoadState,
    show_loading: bool,
    error: Option<ViewerError>,
    error_path: Option<PathBuf>,
    banner: Option<String>,
    request_seq: u64,
    load_task: Option<Task<()>>,
    load_timer: Option<Task<()>>,
    animation_task: Option<Task<()>>,
    animation_generation: u64,
    frame_index: usize,
    viewport_size: Extent,
    image_area_bounds: Rc<Cell<Bounds<Pixels>>>,
    focus_handle: FocusHandle,
    focus_requested: bool,
    fullscreen: bool,
    toolbar_hover: bool,
    show_thumbnails: bool,
    dragging: Option<Point<Pixels>>,
    drop_active: bool,
    last_mouse: Option<Point<Pixels>>,
    scale_factor: f32,
    open_paths: Arc<Mutex<Vec<PathBuf>>>,
    open_paths_task: Option<Task<()>>,
}

impl ViewerApp {
    pub fn new(
        dialog: Arc<dyn FileDialogService>,
        initial: Option<PathBuf>,
        open_paths: Arc<Mutex<Vec<PathBuf>>>,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        let mut app = Self {
            dialog,
            image_list: ImageList::empty(),
            view: Viewport::default(),
            transform: TransformState::default(),
            loaded: None,
            display: None,
            pending_texture_drops: Vec::new(),
            cache: MemoryCache::new(CACHE_BUDGET),
            thumb_cache: MemoryCache::new(THUMBNAIL_CACHE_BUDGET),
            thumb_in_flight: HashSet::new(),
            thumb_failed: HashSet::new(),
            thumb_scroll: ScrollHandle::new(),
            load_state: LoadState::Idle,
            show_loading: false,
            error: None,
            error_path: None,
            banner: None,
            request_seq: 0,
            load_task: None,
            load_timer: None,
            animation_task: None,
            animation_generation: 0,
            frame_index: 0,
            viewport_size: Extent::default(),
            image_area_bounds: Rc::new(Cell::new(Bounds::default())),
            focus_handle,
            focus_requested: false,
            fullscreen: false,
            toolbar_hover: false,
            show_thumbnails: true,
            dragging: None,
            drop_active: false,
            last_mouse: None,
            scale_factor: 1.0,
            open_paths,
            open_paths_task: None,
        };
        app.watch_open_urls(cx);
        if let Some(path) = initial {
            app.open_path(path, cx);
        }
        app
    }

    // ------------------------------------------------------------------
    // 打开与加载
    // ------------------------------------------------------------------

    pub(crate) fn open_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if let Err(err) = std::fs::metadata(&path) {
            self.show_banner(ViewerError::from(err), cx);
            return;
        }

        let is_dir = path.is_dir();
        if !is_dir && !is_supported_path(&path) {
            self.show_banner(ViewerError::Unsupported(path.display().to_string()), cx);
            return;
        }

        let scanned = if is_dir {
            FolderScanner::scan(&path).map(|paths| (paths, 0))
        } else {
            FolderScanner::scan_for_file(&path)
        };

        let (paths, index) = match scanned {
            Ok(result) => result,
            Err(err) => {
                self.show_banner(err, cx);
                return;
            }
        };

        if paths.is_empty() {
            self.image_list = ImageList::empty();
            self.clear_image();
            self.banner = Some("该文件夹中没有可显示的图片".to_string());
            self.error = None;
            self.error_path = None;
            cx.notify();
            return;
        }

        self.image_list = ImageList::new(paths, index);
        self.banner = None;
        self.open_current(cx);
    }

    fn open_current(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.image_list.current_path().map(PathBuf::from) else {
            self.clear_image();
            cx.notify();
            return;
        };

        self.transform.reset();
        self.frame_index = 0;
        self.error = None;
        self.error_path = None;
        self.banner = None;
        self.animation_generation += 1;
        self.animation_task = None;
        self.cache.set_pinned(Some(path.clone()));
        self.scroll_thumbnails_to_current();

        if let Some(loaded) = self.cache.get(&path) {
            self.finish_with_loaded(path, loaded, cx);
            return;
        }

        self.start_loading(path, cx);
    }

    fn start_loading(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let seq = self.next_request();
        self.load_state = LoadState::Loading;
        self.show_loading = false;
        self.error = None;
        self.error_path = None;

        let decode_path = path.clone();
        let decode = cx
            .background_executor()
            .spawn(async move { load_image_catching_panics(&decode_path) });
        self.load_task = Some(cx.spawn(async move |weak, cx| {
            let result = decode.await;
            let _ = weak.update(cx, |this, cx| this.on_loaded(result, seq, cx));
        }));

        self.load_timer = Some(cx.spawn(async move |weak, cx| {
            cx.background_executor().timer(LOADING_DELAY).await;
            let _ = weak.update(cx, |this, cx| {
                if this.request_seq == seq && this.load_state == LoadState::Loading {
                    this.show_loading = true;
                    cx.notify();
                }
            });
        }));
    }

    fn next_request(&mut self) -> u64 {
        self.request_seq += 1;
        self.request_seq
    }

    fn on_loaded(
        &mut self,
        result: Result<LoadedImage, ViewerError>,
        seq: u64,
        cx: &mut Context<Self>,
    ) {
        // 过期结果直接丢弃，防止快速翻页时闪回。
        if seq != self.request_seq {
            return;
        }
        let Some(path) = self.image_list.current_path().map(PathBuf::from) else {
            return;
        };

        match result {
            Ok(loaded) => {
                let loaded = Arc::new(loaded);
                let bytes = loaded.bytes;
                let inserted = self.cache.insert(path.clone(), loaded.clone(), bytes);
                if !inserted.inserted {
                    tracing::warn!("图片超过缓存预算，只显示不缓存：{}", path.display());
                }
                for (_, evicted) in inserted.evicted {
                    self.pending_texture_drops.push(evicted.image.clone());
                }
                self.finish_with_loaded(path, loaded, cx);
            }
            Err(err) => {
                tracing::warn!("加载 {} 失败: {err}", path.display());
                self.load_state = LoadState::Failed;
                self.show_loading = false;
                self.error = Some(err);
                self.error_path = Some(path);
                self.loaded = None;
                self.set_display(None);
                cx.notify();
            }
        }
    }

    fn finish_with_loaded(
        &mut self,
        path: PathBuf,
        loaded: Arc<LoadedImage>,
        cx: &mut Context<Self>,
    ) {
        self.cache.set_pinned(Some(path));
        self.load_state = LoadState::Ready;
        self.show_loading = false;
        self.error = None;
        self.error_path = None;
        self.loaded = Some(loaded);
        self.rebuild_display();
        self.view = Viewport::fit(self.display_extent(), self.viewport_size);
        self.start_animation(cx);
        cx.notify();
    }

    fn clear_image(&mut self) {
        self.loaded = None;
        self.set_display(None);
        self.load_state = LoadState::Idle;
        self.show_loading = false;
        self.error = None;
        self.error_path = None;
        self.cache.set_pinned(None);
        self.animation_generation += 1;
        self.animation_task = None;
    }

    fn show_banner(&mut self, err: ViewerError, cx: &mut Context<Self>) {
        let message = err.user_message();
        tracing::warn!("{message}");
        self.banner = Some(message);
        cx.notify();
    }

    // ------------------------------------------------------------------
    // 纹理与动画
    // ------------------------------------------------------------------

    fn display_extent(&self) -> Extent {
        match &self.display {
            Some(display) => Extent::new(display.width as f32, display.height as f32),
            None => Extent::default(),
        }
    }

    fn rebuild_display(&mut self) {
        let Some(loaded) = self.loaded.clone() else {
            self.set_display(None);
            return;
        };
        let frame_index = self.frame_index.min(loaded.frames.saturating_sub(1));
        let Some(image) = transformed_frame(&loaded.image, frame_index, self.transform) else {
            tracing::error!("应用图像变换失败");
            self.set_display(None);
            return;
        };
        let (width, height) = self.transform.display_size(loaded.width, loaded.height);
        let frame = if self.transform.is_identity() {
            frame_index
        } else {
            0
        };
        self.set_display(Some(Display {
            image,
            frame,
            width,
            height,
        }));
    }

    fn set_display(&mut self, next: Option<Display>) {
        if let Some(old) = self.display.take() {
            let replaced = match &next {
                Some(display) => display.image.id != old.image.id,
                None => true,
            };
            if replaced {
                self.pending_texture_drops.push(old.image);
            }
        }
        self.display = next;
    }

    fn drain_texture_drops(&mut self, window: &mut Window) {
        for image in self.pending_texture_drops.drain(..) {
            if let Err(err) = window.drop_image(image) {
                tracing::warn!("释放纹理失败: {err}");
            }
        }
    }

    fn start_animation(&mut self, cx: &mut Context<Self>) {
        self.animation_generation += 1;
        self.animation_task = None;

        let Some(loaded) = self.loaded.clone() else {
            return;
        };
        if !loaded.is_animated() {
            return;
        }

        let generation = self.animation_generation;
        let image_id = loaded.image.id;
        self.animation_task = Some(cx.spawn(async move |weak, cx| loop {
            let delay = match weak.update(cx, |this, _cx| {
                if this.animation_generation != generation {
                    return None;
                }
                let loaded = this
                    .loaded
                    .as_ref()
                    .filter(|loaded| loaded.image.id == image_id)?;
                let index = this.frame_index.min(loaded.frames.saturating_sub(1));
                let (numer, denom) = loaded.image.delay(index).numer_denom_ms();
                Some(Duration::from_millis(u64::from(numer)) / denom.max(1))
            }) {
                Ok(Some(delay)) => delay,
                _ => break,
            };

            cx.background_executor().timer(delay).await;

            let keep_going = weak.update(cx, |this, cx| {
                if this.animation_generation != generation {
                    return false;
                }
                let Some(loaded) = this.loaded.clone() else {
                    return false;
                };
                if loaded.image.id != image_id {
                    return false;
                }
                let frames = loaded.frames.max(1);
                this.frame_index = (this.frame_index + 1) % frames;
                this.rebuild_display();
                cx.notify();
                true
            });
            if !keep_going.unwrap_or(false) {
                break;
            }
        }));
    }

    // ------------------------------------------------------------------
    // 视口操作
    // ------------------------------------------------------------------

    pub(crate) fn zoom_in(&mut self, cx: &mut Context<Self>) {
        self.zoom_by(ZOOM_STEP, cx);
    }

    pub(crate) fn zoom_out(&mut self, cx: &mut Context<Self>) {
        self.zoom_by(1.0 / ZOOM_STEP, cx);
    }

    fn zoom_by(&mut self, factor: f32, cx: &mut Context<Self>) {
        let image = self.display_extent();
        if image.width <= 0.0 || image.height <= 0.0 {
            return;
        }
        self.view = self.view.zoom_by(image, self.viewport_size, factor);
        cx.notify();
    }

    pub(crate) fn fit_to_window(&mut self, cx: &mut Context<Self>) {
        let image = self.display_extent();
        if image.width <= 0.0 || image.height <= 0.0 {
            return;
        }
        self.view = Viewport::fit(image, self.viewport_size);
        cx.notify();
    }

    pub(crate) fn actual_size(&mut self, cx: &mut Context<Self>) {
        let image = self.display_extent();
        if image.width <= 0.0 || image.height <= 0.0 {
            return;
        }
        self.view = Viewport::actual(image, self.viewport_size, self.scale_factor);
        cx.notify();
    }

    fn is_pannable(&self) -> bool {
        self.view
            .is_pannable(self.display_extent(), self.viewport_size)
    }

    // ------------------------------------------------------------------
    // 翻页与变换
    // ------------------------------------------------------------------

    pub(crate) fn next_image(&mut self, cx: &mut Context<Self>) {
        if self.image_list.next() {
            self.open_current(cx);
        }
    }

    pub(crate) fn prev_image(&mut self, cx: &mut Context<Self>) {
        if self.image_list.prev() {
            self.open_current(cx);
        }
    }

    pub(crate) fn first_image(&mut self, cx: &mut Context<Self>) {
        if self.image_list.first() {
            self.open_current(cx);
        }
    }

    pub(crate) fn last_image(&mut self, cx: &mut Context<Self>) {
        if self.image_list.last() {
            self.open_current(cx);
        }
    }

    pub(crate) fn jump_to(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.image_list.set_index(index) {
            self.open_current(cx);
        }
    }

    pub(crate) fn rotate_cw(&mut self, cx: &mut Context<Self>) {
        self.transform.rotate_cw();
        self.after_transform_change(cx);
    }

    pub(crate) fn rotate_ccw(&mut self, cx: &mut Context<Self>) {
        self.transform.rotate_ccw();
        self.after_transform_change(cx);
    }

    pub(crate) fn flip_horizontal(&mut self, cx: &mut Context<Self>) {
        self.transform.flip_horizontal();
        self.after_transform_change(cx);
    }

    pub(crate) fn flip_vertical(&mut self, cx: &mut Context<Self>) {
        self.transform.flip_vertical();
        self.after_transform_change(cx);
    }

    pub(crate) fn reset_transform(&mut self, cx: &mut Context<Self>) {
        self.transform.reset();
        self.rebuild_display();
        // 重置同时恢复适应窗口（T12：重置回到 Fit 模式）。
        self.view = Viewport::fit(self.display_extent(), self.viewport_size);
        cx.notify();
    }

    fn after_transform_change(&mut self, cx: &mut Context<Self>) {
        self.rebuild_display();
        self.view = self
            .view
            .recomputed(self.display_extent(), self.viewport_size);
        cx.notify();
    }

    // ------------------------------------------------------------------
    // 对话框、拖拽与全屏
    // ------------------------------------------------------------------

    pub(crate) fn pick_image(&mut self, cx: &mut Context<Self>) {
        match self.dialog.pick_image() {
            Ok(Some(path)) => self.open_path(path, cx),
            Ok(None) => {}
            Err(err) => self.show_banner(err, cx),
        }
    }

    pub(crate) fn pick_folder(&mut self, cx: &mut Context<Self>) {
        match self.dialog.pick_folder() {
            Ok(Some(path)) => self.open_path(path, cx),
            Ok(None) => {}
            Err(err) => self.show_banner(err, cx),
        }
    }

    pub(crate) fn toggle_fullscreen(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.toggle_fullscreen();
        self.fullscreen = window.is_fullscreen();
        self.toolbar_hover = false;
        cx.notify();
    }

    fn on_files_dropped(
        &mut self,
        paths: &ExternalPaths,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.drop_active = false;
        let Some(path) = paths.paths().first().cloned() else {
            cx.notify();
            return;
        };
        self.open_path(path, cx);
    }

    fn on_external_drag_move(
        &mut self,
        _event: &DragMoveEvent<ExternalPaths>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.drop_active {
            self.drop_active = true;
            cx.notify();
        }
    }

    fn on_file_drop_exit(
        &mut self,
        _event: &gpui_kit::FileDropEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.drop_active {
            self.drop_active = false;
            cx.notify();
        }
    }

    fn watch_open_urls(&mut self, cx: &mut Context<Self>) {
        let queue = self.open_paths.clone();
        self.open_paths_task = Some(cx.spawn(async move |weak, cx| loop {
            cx.background_executor().timer(OPEN_URLS_INTERVAL).await;
            let paths = {
                let mut guard = queue
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                std::mem::take(&mut *guard)
            };
            let Some(path) = paths.into_iter().next() else {
                continue;
            };
            if weak
                .update(cx, |this, cx| this.open_path(path, cx))
                .is_err()
            {
                break;
            }
        }));
    }

    // ------------------------------------------------------------------
    // 事件
    // ------------------------------------------------------------------

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.to_ascii_lowercase();
        let modifiers = event.keystroke.modifiers;
        let command = modifiers.platform || modifiers.control;

        if command {
            match key.as_str() {
                "o" if modifiers.shift => self.pick_folder(cx),
                "o" => self.pick_image(cx),
                "=" | "+" => self.zoom_in(cx),
                "-" => self.zoom_out(cx),
                "0" => self.fit_to_window(cx),
                _ => {}
            }
            return;
        }

        match key.as_str() {
            "left" | "arrowleft" => self.prev_image(cx),
            "right" | "arrowright" => self.next_image(cx),
            "home" => self.first_image(cx),
            "end" => self.last_image(cx),
            "+" | "=" => self.zoom_in(cx),
            "-" => self.zoom_out(cx),
            "0" => self.fit_to_window(cx),
            "f" | "f11" => self.toggle_fullscreen(window, cx),
            "escape" => {
                if window.is_fullscreen() {
                    self.toggle_fullscreen(window, cx);
                } else if self.banner.is_some() {
                    self.banner = None;
                    cx.notify();
                }
            }
            "r" if modifiers.shift => self.rotate_ccw(cx),
            "r" => self.rotate_cw(cx),
            "h" => self.flip_horizontal(cx),
            "v" => self.flip_vertical(cx),
            _ => {}
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_handle.focus(window, cx);
        if event.button != MouseButton::Left || !self.is_pannable() {
            return;
        }
        self.dragging = Some(event.position);
        cx.notify();
    }

    fn on_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.dragging.take().is_some() {
            cx.notify();
        }
    }

    fn on_image_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.last_mouse = Some(event.position);
        let Some(last) = self.dragging else {
            return;
        };
        if event.pressed_button != Some(MouseButton::Left) {
            self.dragging = None;
            cx.notify();
            return;
        }
        let delta = Offset::new(
            f32::from(event.position.x - last.x),
            f32::from(event.position.y - last.y),
        );
        if delta.x == 0.0 && delta.y == 0.0 {
            return;
        }
        self.view = self
            .view
            .panned(self.display_extent(), self.viewport_size, delta);
        self.dragging = Some(event.position);
        cx.notify();
    }

    fn on_scroll(&mut self, event: &ScrollEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let image = self.display_extent();
        if image.width <= 0.0 || image.height <= 0.0 {
            return;
        }
        let delta_y = match event.delta {
            ScrollDelta::Pixels(delta) => f32::from(delta.y),
            ScrollDelta::Lines(lines) => lines.y * 20.0,
        };
        if delta_y.abs() < f32::EPSILON {
            return;
        }
        // 滚轮向下（正值）缩小，向上放大。
        let factor = (-delta_y * WHEEL_ZOOM_SENSITIVITY).exp();
        let bounds = self.image_area_bounds.get();
        let cursor = if f32::from(bounds.size.width) > 0.0 {
            Offset::new(
                f32::from(event.position.x - bounds.origin.x),
                f32::from(event.position.y - bounds.origin.y),
            )
        } else {
            Offset::new(
                self.viewport_size.width / 2.0,
                self.viewport_size.height / 2.0,
            )
        };
        self.view = self
            .view
            .zoomed_at(image, self.viewport_size, factor, cursor);
        cx.notify();
    }

    fn on_root_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.last_mouse = Some(event.position);
        if !window.is_fullscreen() {
            return;
        }
        let height = f32::from(window.viewport_size().height);
        let near_bottom = f32::from(event.position.y) > height - 96.0;
        if near_bottom != self.toolbar_hover {
            self.toolbar_hover = near_bottom;
            cx.notify();
        }
    }

    fn on_root_mouse_exit(
        &mut self,
        _event: &gpui_kit::MouseExitEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.last_mouse = None;
        if self.toolbar_hover {
            self.toolbar_hover = false;
            cx.notify();
        }
    }

    // ------------------------------------------------------------------
    // 缩略图
    // ------------------------------------------------------------------

    /// 缩略图条的可视宽度。布局前 `ScrollHandle` 还没有尺寸，退回窗口宽度。
    fn thumb_viewport_width(&self) -> f32 {
        let width = f32::from(self.thumb_scroll.bounds().size.width);
        if width > 0.0 {
            width
        } else {
            self.viewport_size.width.max(1.0)
        }
    }

    fn scroll_thumbnails_to_current(&mut self) {
        let index = self.image_list.current_index();
        let item = THUMBNAIL_BOX + THUMBNAIL_GAP;
        let viewport = self.thumb_viewport_width();
        let total = self.image_list.len() as f32 * item;
        let x = (index as f32 * item - (viewport - THUMBNAIL_BOX) / 2.0)
            .clamp(0.0, (total - viewport).max(0.0));
        // ScrollHandle 用负值表示已滚动距离，正值会在布局时被 clamp 回 0。
        self.thumb_scroll.set_offset(point(px(-x), px(0.0)));
    }

    fn request_visible_thumbnails(&mut self, cx: &mut Context<Self>) {
        // 主图尚未就绪时让出解码线程，保证打开速度。
        if !self.thumbnails_shown()
            || self.image_list.is_empty()
            || self.load_state == LoadState::Loading
        {
            return;
        }
        let (first, last) = self.visible_thumbnail_range();

        for index in first..last {
            if self.thumb_in_flight.len() >= THUMBNAIL_CONCURRENCY {
                break;
            }
            let Some(path) = self.image_list.get(index).map(PathBuf::from) else {
                continue;
            };
            if self.thumb_cache.contains(&path)
                || self.thumb_failed.contains(&path)
                || self.thumb_in_flight.contains(&path)
            {
                continue;
            }
            self.thumb_in_flight.insert(path.clone());
            let decode_path = path.clone();
            let decode = cx
                .background_executor()
                .spawn(async move { load_thumbnail_catching_panics(&decode_path) });
            cx.spawn(async move |weak, cx| {
                let result = decode.await;
                let _ = weak.update(cx, |this, cx| {
                    this.thumb_in_flight.remove(&path);
                    match result {
                        Ok(image) => {
                            let bytes = render_image_bytes(&image);
                            let inserted = this.thumb_cache.insert(path.clone(), image, bytes);
                            for (_, evicted) in inserted.evicted {
                                this.pending_texture_drops.push(evicted);
                            }
                        }
                        Err(err) => {
                            tracing::warn!("缩略图生成失败 {}: {err}", path.display());
                            this.thumb_failed.insert(path);
                        }
                    }
                    cx.notify();
                });
            })
            .detach();
        }
    }

    // ------------------------------------------------------------------
    // 渲染
    // ------------------------------------------------------------------

    fn toolbar_shown(&self) -> bool {
        !self.fullscreen || self.toolbar_hover
    }

    fn thumbnails_shown(&self) -> bool {
        self.show_thumbnails && !self.fullscreen
    }

    fn banner_shown(&self) -> bool {
        self.banner.is_some()
    }

    fn render_root(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if !self.focus_requested {
            self.focus_requested = true;
            // 渲染期间不能直接改焦点，推迟到本帧结束后执行。
            cx.defer_in(window, |this, window, cx| {
                this.focus_handle.focus(window, cx);
            });
        }
        self.scale_factor = window.scale_factor();
        self.fullscreen = window.is_fullscreen();
        self.drain_texture_drops(window);

        let window_size = window.viewport_size();
        let toolbar_height = if self.toolbar_shown() {
            TOOLBAR_HEIGHT
        } else {
            0.0
        };
        let banner_height = if self.banner_shown() { 30.0 } else { 0.0 };
        let status_height = if self.fullscreen { 0.0 } else { STATUS_HEIGHT };
        let thumbnail_height = if self.thumbnails_shown() {
            THUMBNAIL_STRIP_HEIGHT
        } else {
            0.0
        };
        let image_height = (f32::from(window_size.height)
            - toolbar_height
            - banner_height
            - status_height
            - thumbnail_height)
            .max(0.0);
        self.viewport_size = Extent::new(f32::from(window_size.width), image_height);

        let image_extent = self.display_extent();
        if image_extent.width > 0.0 && image_extent.height > 0.0 {
            self.view = self.view.recomputed(image_extent, self.viewport_size);
        }

        self.request_visible_thumbnails(cx);

        let mut root = div()
            .id("viewer-root")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme::BACKGROUND)
            .text_color(theme::TEXT)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_drop::<ExternalPaths>(cx.listener(Self::on_files_dropped))
            .on_drag_move::<ExternalPaths>(cx.listener(Self::on_external_drag_move))
            .on_file_drop_exit(cx.listener(Self::on_file_drop_exit))
            .on_mouse_move(cx.listener(Self::on_root_mouse_move))
            .on_mouse_exit(cx.listener(Self::on_root_mouse_exit));

        if self.toolbar_shown() {
            root = root.child(self.render_toolbar(window, cx));
        }
        if let Some(message) = self.banner.clone() {
            root = root.child(self.render_banner(message, cx));
        }
        root = root.child(self.render_image_area(window, cx));
        if self.thumbnails_shown() {
            root = root.child(self.render_thumbnail_strip(cx));
        }
        if !self.fullscreen {
            root = root.child(self.render_status_bar());
        }
        root.into_any_element()
    }

    fn render_banner(&self, message: String, cx: &mut Context<Self>) -> AnyElement {
        div()
            .w_full()
            .h(px(30.0))
            .flex()
            .items_center()
            .justify_between()
            .px(px(12.0))
            .bg(rgba(0x5a2a2aff))
            .text_size(px(13.0))
            .child(message)
            .child(
                div()
                    .id("banner-close")
                    .px(px(8.0))
                    .cursor_pointer()
                    .hover(|style| style.bg(theme::PANEL_HOVER))
                    .on_click(cx.listener(|this, _event: &ClickEvent, _window, cx| {
                        this.banner = None;
                        cx.notify();
                    }))
                    .child("关闭"),
            )
            .into_any_element()
    }

    fn render_image_area(&self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let bounds_cell = self.image_area_bounds.clone();
        let display = self.display.as_ref().map(|display| Display {
            image: display.image.clone(),
            frame: display.frame,
            width: display.width,
            height: display.height,
        });
        let view = self.view;

        let cursor = if self.dragging.is_some() {
            CursorStyle::ClosedHand
        } else if self.is_pannable() {
            CursorStyle::OpenHand
        } else {
            CursorStyle::Arrow
        };

        let mut area = div()
            .relative()
            .flex_1()
            .w_full()
            .overflow_hidden()
            .bg(theme::VIEWPORT)
            .cursor(cursor)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_image_mouse_move))
            .on_scroll(cx.listener(Self::on_scroll));

        if let Some(display) = display {
            let image = display.image;
            let frame = display.frame;
            // canvas 没有固有尺寸，必须显式撑满，否则布局高度为 0、绘制回调拿不到可见区域。
            area = area.child(
                canvas(
                    move |bounds, _window, _cx| {
                        bounds_cell.set(bounds);
                        bounds
                    },
                    move |_element_bounds, bounds, window, _cx| {
                        let viewport_extent = Extent::new(
                            f32::from(bounds.size.width),
                            f32::from(bounds.size.height),
                        );
                        let image_extent = Extent::new(display.width as f32, display.height as f32);
                        let origin = view.image_origin(image_extent, viewport_extent);
                        let image_bounds = Bounds::new(
                            point(
                                bounds.origin.x + px(origin.x),
                                bounds.origin.y + px(origin.y),
                            ),
                            size(
                                px(image_extent.width * view.scale),
                                px(image_extent.height * view.scale),
                            ),
                        );
                        let _ = window.paint_image(
                            bounds,
                            image_bounds,
                            gpui_kit::Corners::default(),
                            image,
                            frame,
                            false,
                        );
                    },
                )
                .size_full(),
            );
        }

        if self.load_state == LoadState::Loading && self.show_loading {
            area = area.child(self.centered_notice("正在加载…", None));
        }
        if let Some(error) = &self.error {
            area = area.child(self.render_error_card(error));
        } else if self.image_list.is_empty() && self.banner.is_none() {
            area = area.child(
                self.centered_notice(crate::EMPTY_STATE_LABEL, Some("可以将图片或文件夹拖入窗口")),
            );
        }
        if self.drop_active {
            area = area.child(
                div()
                    .absolute()
                    .top(px(0.0))
                    .left(px(0.0))
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgba(0x4c9aff22))
                    .border_2()
                    .border_color(theme::ACCENT)
                    .text_size(px(16.0))
                    .child("松开以打开"),
            );
        }

        area.into_any_element()
    }

    fn centered_notice(&self, title: &'static str, detail: Option<&'static str>) -> AnyElement {
        let mut card = div()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .text_color(theme::MUTED)
            .child(
                div()
                    .text_size(px(18.0))
                    .text_color(theme::TEXT)
                    .child(title),
            );
        if let Some(detail) = detail {
            card = card.child(div().mt(px(8.0)).text_size(px(13.0)).child(detail));
        }
        card.into_any_element()
    }

    fn render_error_card(&self, error: &ViewerError) -> AnyElement {
        let message = error.user_message();
        let file_name = self
            .error_path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();

        let mut card = div()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(8.0))
                    .px(px(24.0))
                    .py(px(20.0))
                    .rounded(px(10.0))
                    .bg(theme::PANEL)
                    .border_1()
                    .border_color(theme::BORDER)
                    .text_size(px(13.0))
                    .child(div().text_size(px(28.0)).child("⚠"))
                    .child(
                        div()
                            .text_size(px(15.0))
                            .text_color(theme::ERROR)
                            .child(message),
                    )
                    .child(div().text_color(theme::MUTED).child(file_name)),
            );

        if matches!(error, ViewerError::Unsupported(_)) {
            card = card.child(
                div()
                    .text_size(px(12.0))
                    .text_color(theme::MUTED)
                    .child("支持：PNG / JPEG / BMP / GIF / WebP / TIFF / ICO"),
            );
        }
        card.into_any_element()
    }
}

impl Render for ViewerApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_root(window, cx)
    }
}

fn render_image_bytes(image: &Arc<RenderImage>) -> usize {
    let size = image.size(0);
    let width = size.width.0.max(0) as usize;
    let height = size.height.0.max(0) as usize;
    width.saturating_mul(height).saturating_mul(4)
}

/// 供同 crate 的工具栏、状态栏与缩略图条读取展示所需的状态。
impl ViewerApp {
    pub(crate) fn current_path(&self) -> Option<&Path> {
        self.image_list.current_path()
    }

    pub(crate) fn current_index(&self) -> usize {
        self.image_list.current_index()
    }

    pub(crate) fn total_images(&self) -> usize {
        self.image_list.len()
    }

    pub(crate) fn zoom_percent(&self) -> f32 {
        self.view.scale * 100.0
    }

    pub(crate) fn current_scale_mode(&self) -> ScaleMode {
        self.view.mode
    }

    pub(crate) fn has_image(&self) -> bool {
        self.display.is_some()
    }

    pub(crate) fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    pub(crate) fn current_file_name(&self) -> Option<String> {
        self.current_path()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
    }

    pub(crate) fn image_dimensions(&self) -> Option<(u32, u32)> {
        self.display
            .as_ref()
            .map(|display| (display.width, display.height))
    }

    pub(crate) fn frame_info(&self) -> Option<(usize, usize)> {
        self.loaded
            .as_ref()
            .filter(|loaded| loaded.frames > 1)
            .map(|loaded| (self.frame_index.min(loaded.frames - 1), loaded.frames))
    }

    pub(crate) fn thumbnail_label(&self, index: usize) -> Option<String> {
        self.image_list
            .get(index)
            .and_then(|path| path.extension())
            .map(|extension| extension.to_string_lossy().to_uppercase())
    }

    pub(crate) fn thumbnail_image(&self, index: usize) -> Option<Arc<RenderImage>> {
        self.image_list
            .get(index)
            .and_then(|path| self.thumb_cache.peek(path))
    }

    pub(crate) fn thumbnail_failed(&self, index: usize) -> bool {
        self.image_list
            .get(index)
            .is_some_and(|path| self.thumb_failed.contains(path))
    }

    pub(crate) fn toggle_thumbnails(&mut self, cx: &mut Context<Self>) {
        self.show_thumbnails = !self.show_thumbnails;
        cx.notify();
    }

    pub(crate) fn thumb_scroll_handle(&self) -> &ScrollHandle {
        &self.thumb_scroll
    }

    /// 当前滚动位置下需要建立元素的缩略图索引区间。
    pub(crate) fn visible_thumbnail_range(&self) -> (usize, usize) {
        let item = THUMBNAIL_BOX + THUMBNAIL_GAP;
        // 存的是负偏移，取反得到已滚动距离。
        let offset = -f32::from(self.thumb_scroll.offset().x);
        visible_index_range(
            offset,
            self.thumb_viewport_width(),
            self.image_list.len(),
            item,
        )
    }
}

/// 由已滚动距离推出需要渲染的缩略图下标区间 `[first, last)`。
fn visible_index_range(
    offset: f32,
    viewport_width: f32,
    total: usize,
    item: f32,
) -> (usize, usize) {
    if total == 0 || item <= 0.0 {
        return (0, 0);
    }
    let first = ((offset.max(0.0) / item).floor() as usize)
        .saturating_sub(1)
        .min(total);
    let visible = (viewport_width.max(1.0) / item).ceil() as usize + 2;
    let last = first.saturating_add(visible).min(total);
    (first, last)
}

#[cfg(test)]
mod tests {
    use super::visible_index_range;

    const ITEM: f32 = 68.0;

    #[test]
    fn range_starts_at_zero_without_scrolling() {
        assert_eq!(visible_index_range(0.0, 680.0, 1000, ITEM), (0, 12));
    }

    #[test]
    fn range_follows_the_scroll_offset() {
        // 滚动 680px（10 项）后，第一项应退到 9 附近，并在前后各留一格。
        let (first, last) = visible_index_range(680.0, 680.0, 1000, ITEM);
        assert_eq!(first, 9);
        assert_eq!(last, 21);
    }

    #[test]
    fn range_is_clamped_at_the_end() {
        let (first, last) = visible_index_range(100_000.0, 680.0, 20, ITEM);
        assert_eq!(last, 20);
        assert_eq!(first, 20);
    }

    #[test]
    fn negative_offset_is_treated_as_no_scroll() {
        assert_eq!(visible_index_range(-500.0, 680.0, 1000, ITEM), (0, 12));
    }
}
