//! 解码单个文件并打印耗时，用于手动性能验收。
//!
//! 用法: cargo run --release -p biuviewer-decoder --example decode_file -- <图片路径>

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::time::Instant;

use biuviewer_decoder::{decode_gif, decode_static, FormatDetector, ImageFormat};

fn main() {
    let path = PathBuf::from(std::env::args().nth(1).expect("需要图片路径"));
    let bytes = std::fs::read(&path).unwrap();
    let format = FormatDetector::detect(&bytes).unwrap();

    let start = Instant::now();
    if format == ImageFormat::Gif {
        let animation = decode_gif(&bytes).unwrap();
        let elapsed = start.elapsed();
        println!(
            "{}: GIF {}×{} 共 {} 帧，用时 {:.1} ms",
            path.display(),
            animation.width,
            animation.height,
            animation.frames.len(),
            elapsed.as_secs_f64() * 1000.0
        );
    } else {
        let image = decode_static(&bytes).unwrap();
        let elapsed = start.elapsed();
        println!(
            "{}: {format:?} {}×{}，用时 {:.1} ms",
            path.display(),
            image.width,
            image.height,
            elapsed.as_secs_f64() * 1000.0
        );
    }
}
