//! 深色主题下的固定色板。MVP 不提供主题切换。

use gpui_kit::Rgba;

const fn color(hex: u32) -> Rgba {
    let bytes = hex.to_be_bytes();
    Rgba {
        r: bytes[1] as f32 / 255.0,
        g: bytes[2] as f32 / 255.0,
        b: bytes[3] as f32 / 255.0,
        a: 1.0,
    }
}

pub const BACKGROUND: Rgba = color(0x1b1b1f);
pub const VIEWPORT: Rgba = color(0x101013);
pub const PANEL: Rgba = color(0x26262b);
pub const PANEL_HOVER: Rgba = color(0x323239);
pub const BORDER: Rgba = color(0x3a3a42);
pub const TEXT: Rgba = color(0xe6e6e6);
pub const MUTED: Rgba = color(0x9a9aa3);
pub const ACCENT: Rgba = color(0x4c9aff);
pub const ERROR: Rgba = color(0xff6b6b);

pub const TOOLBAR_HEIGHT: f32 = 44.0;
pub const STATUS_HEIGHT: f32 = 26.0;
pub const THUMBNAIL_STRIP_HEIGHT: f32 = 92.0;
pub const THUMBNAIL_BOX: f32 = 64.0;
pub const THUMBNAIL_GAP: f32 = 6.0;
pub const THUMBNAIL_EDGE: u32 = 128;
