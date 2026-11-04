//! 与界面无关的核心类型。解码器、平台层和界面都依赖本 crate，本 crate 不反向依赖它们。

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod cache;
pub mod error;
pub mod image_list;
pub mod log;
pub mod platform_service;
pub mod transform;
pub mod viewport;

pub use cache::{InsertResult, MemoryCache};
pub use error::ViewerError;
pub use image_list::ImageList;
pub use log::init_logging;
pub use platform_service::{FileDialogService, NoopFileDialog};
pub use transform::TransformState;
pub use viewport::{fit_scale, Extent, Offset, ScaleMode, Viewport, MAX_SCALE, MIN_SCALE};
