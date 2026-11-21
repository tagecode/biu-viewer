use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use biuviewer_core::{init_logging, AppConfig};
use biuviewer_platform::NativeFileDialog;
use biuviewer_ui::{paths_from_urls, ViewerApp};
use gpui_kit::{
    open_window, point, px, size, App, AppContext, Bounds, TitlebarOptions, WindowBounds,
    WindowOptions,
};

/// 启动 GPUI。`initial` 来自命令行，运行期从“打开方式”收到的路径通过共享队列交给界面。
pub fn run(initial: Option<PathBuf>) {
    init_logging();
    tracing::info!("BiuViewer 启动");

    let pending: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));
    let pending_for_urls = pending.clone();
    let config = AppConfig::load();

    let application = gpui_kit::application();
    application.on_open_urls(move |urls| {
        let paths = paths_from_urls(&urls);
        if paths.is_empty() {
            return;
        }
        let mut queue = pending_for_urls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        queue.extend(paths);
    });

    application.run(move |cx: &mut App| {
        gpui_kit::init(cx);
        cx.on_window_closed(|cx, _closed_id| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let dialog = Arc::new(NativeFileDialog::new());
        let open_paths = pending.clone();
        if let Err(err) = open_window(main_window_options(&config), cx, move |window, cx| {
            restore_legacy_window_size(window, &config);
            cx.new(|cx| ViewerApp::new(dialog, initial, open_paths, config, cx))
        }) {
            tracing::error!("打开窗口失败: {err}");
            cx.quit();
        }
    });
}

/// v1 配置把窗口尺寸存成含标题栏的 frame，而 `WindowBounds::Windowed` 按内容区域创建。
/// 升级后首次启动时按实际内缩把内容区域缩回原 frame 尺寸，之后保存的就是内容尺寸。
fn restore_legacy_window_size(window: &mut gpui_kit::Window, config: &AppConfig) {
    if !config.window_size_is_frame {
        return;
    }
    let frame = window.window_bounds().get_bounds().size;
    let content = window.viewport_size();
    let inset_width = f32::from(frame.width) - f32::from(content.width);
    let inset_height = f32::from(frame.height) - f32::from(content.height);
    let width = (config.window.width - inset_width).max(640.0);
    let height = (config.window.height - inset_height).max(480.0);
    window.resize(size(px(width), px(height)));
    tracing::info!(
        "迁移 v1 窗口尺寸: frame {:.0}x{:.0} / 内缩 {:.0}x{:.0} -> 内容 {width:.0}x{height:.0}",
        f32::from(frame.width),
        f32::from(frame.height),
        inset_width,
        inset_height
    );
}

fn main_window_options(config: &AppConfig) -> WindowOptions {
    let window = config.window;
    WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some("BiuViewer".into()),
            appears_transparent: false,
            traffic_light_position: None,
        }),
        is_resizable: true,
        is_minimizable: true,
        window_min_size: Some(size(px(640.0), px(480.0))),
        window_bounds: Some(WindowBounds::Windowed(Bounds {
            origin: point(px(window.x), px(window.y)),
            size: size(px(window.width), px(window.height)),
        })),
        ..Default::default()
    }
}
