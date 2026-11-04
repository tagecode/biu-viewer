use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use biuviewer_core::init_logging;
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
        if let Err(err) = open_window(main_window_options(), cx, move |_window, cx| {
            cx.new(|cx| ViewerApp::new(dialog, initial, open_paths, cx))
        }) {
            tracing::error!("打开窗口失败: {err}");
            cx.quit();
        }
    });
}

fn main_window_options() -> WindowOptions {
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
            origin: point(px(80.0), px(60.0)),
            size: size(px(960.0), px(640.0)),
        })),
        ..Default::default()
    }
}
