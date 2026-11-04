//! 应用入口。命令行解析与窗口启动。

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

mod app;
mod cli;

pub use app::run;
pub use cli::{help_text, parse_args, version_line, CliAction};
