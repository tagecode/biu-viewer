//! 轻量配置：只保存窗口几何、缩放模式与缩略图开关。
//!
//! 不使用 serde，配置是 `key=value` 文本，读取时忽略未知键与坏行。
//! 这样升级版本时新增字段不会让旧配置失效。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::viewport::ScaleMode;

const DEFAULT_WIDTH: f32 = 960.0;
const DEFAULT_HEIGHT: f32 = 640.0;
const DEFAULT_X: f32 = 80.0;
const DEFAULT_Y: f32 = 60.0;
const MIN_WIDTH: f32 = 640.0;
const MIN_HEIGHT: f32 = 480.0;
const MAX_WIDTH: f32 = 7680.0;
const MAX_HEIGHT: f32 = 4320.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowConfig {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            x: DEFAULT_X,
            y: DEFAULT_Y,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
        }
    }
}

impl WindowConfig {
    /// 防止损坏的配置把窗口放到屏幕外或缩成看不见。
    fn sanitized(mut self) -> Self {
        if !self.x.is_finite() || self.x.abs() > 100_000.0 {
            self.x = DEFAULT_X;
        }
        if !self.y.is_finite() || self.y.abs() > 100_000.0 {
            self.y = DEFAULT_Y;
        }
        self.width = if self.width.is_finite() {
            self.width.clamp(MIN_WIDTH, MAX_WIDTH)
        } else {
            DEFAULT_WIDTH
        };
        self.height = if self.height.is_finite() {
            self.height.clamp(MIN_HEIGHT, MAX_HEIGHT)
        } else {
            DEFAULT_HEIGHT
        };
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub scale_mode: ScaleMode,
    pub show_thumbnails: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            scale_mode: ScaleMode::Fit,
            show_thumbnails: true,
        }
    }
}

impl AppConfig {
    /// 读取默认配置路径。文件不存在、不可读或内容损坏时退回默认值。
    pub fn load() -> Self {
        let Some(path) = config_path() else {
            return Self::default();
        };
        match Self::load_from(&path) {
            Ok(config) => config,
            Err(err) if err.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(err) => {
                tracing::warn!("读取配置 {} 失败: {err}", path.display());
                Self::default()
            }
        }
    }

    pub fn load_from(path: &Path) -> io::Result<Self> {
        let text = fs::read_to_string(path)?;
        Ok(Self::parse(&text))
    }

    pub fn save(&self) -> io::Result<()> {
        let Some(path) = config_path() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "当前平台没有可用的配置目录",
            ));
        };
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.serialize())
    }

    fn parse(text: &str) -> Self {
        let mut config = Self::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "window_x" => {
                    if let Ok(x) = value.parse::<f32>() {
                        config.window.x = x;
                    }
                }
                "window_y" => {
                    if let Ok(y) = value.parse::<f32>() {
                        config.window.y = y;
                    }
                }
                "window_width" => {
                    if let Ok(width) = value.parse::<f32>() {
                        config.window.width = width;
                    }
                }
                "window_height" => {
                    if let Ok(height) = value.parse::<f32>() {
                        config.window.height = height;
                    }
                }
                "scale_mode" => {
                    config.scale_mode = match value {
                        "actual" => ScaleMode::Actual,
                        "custom" => ScaleMode::Custom,
                        _ => ScaleMode::Fit,
                    };
                }
                "show_thumbnails" => {
                    config.show_thumbnails = value != "false" && value != "0";
                }
                _ => {}
            }
        }
        config.window = config.window.sanitized();
        config
    }

    fn serialize(&self) -> String {
        let mode = match self.scale_mode {
            ScaleMode::Fit => "fit",
            ScaleMode::Actual => "actual",
            ScaleMode::Custom => "custom",
        };
        format!(
            "# BiuViewer 配置\n\
             window_x={:.0}\n\
             window_y={:.0}\n\
             window_width={:.0}\n\
             window_height={:.0}\n\
             scale_mode={mode}\n\
             show_thumbnails={}\n",
            self.window.x,
            self.window.y,
            self.window.width,
            self.window.height,
            self.show_thumbnails
        )
    }
}

/// Windows 用 `%APPDATA%\BiuViewer`，macOS 用 Application Support，
/// 其余平台遵循 XDG。取不到目录时返回 `None`，调用方使用默认值即可。
pub fn config_path() -> Option<PathBuf> {
    let dir = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(|base| PathBuf::from(base).join("BiuViewer"))
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME")
            .map(|home| PathBuf::from(home).join("Library/Application Support/BiuViewer"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(|base| PathBuf::from(base).join("biuviewer"))
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/biuviewer"))
            })
    };
    dir.map(|dir| dir.join("config.txt"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_keys_and_ignores_bad_lines() {
        let config = AppConfig::parse(
            "\
             # comment\n\
             window_x=10\n\
             window_y=20\n\
             window_width=1200\n\
             window_height=800\n\
             scale_mode=actual\n\
             show_thumbnails=false\n\
             unknown=value\n\
             broken-line\n",
        );
        assert_eq!(config.window.x, 10.0);
        assert_eq!(config.window.y, 20.0);
        assert_eq!(config.window.width, 1200.0);
        assert_eq!(config.window.height, 800.0);
        assert_eq!(config.scale_mode, ScaleMode::Actual);
        assert!(!config.show_thumbnails);
    }

    #[test]
    fn invalid_values_fall_back_to_defaults() {
        let config = AppConfig::parse(
            "window_x=NaN\nwindow_y=1e30\nwindow_width=10\nwindow_height=abc\nscale_mode=nope\n",
        );
        let defaults = AppConfig::default();
        assert_eq!(config.window.x, defaults.window.x);
        assert_eq!(config.window.y, defaults.window.y);
        assert_eq!(config.window.width, MIN_WIDTH);
        assert_eq!(config.window.height, defaults.window.height);
        assert_eq!(config.scale_mode, ScaleMode::Fit);
    }

    #[test]
    fn round_trips_through_serialized_text() {
        let config = AppConfig {
            window: WindowConfig {
                x: -120.0,
                y: 40.0,
                width: 1280.0,
                height: 720.0,
            },
            scale_mode: ScaleMode::Custom,
            show_thumbnails: false,
        };
        assert_eq!(AppConfig::parse(&config.serialize()), config);
    }
}
