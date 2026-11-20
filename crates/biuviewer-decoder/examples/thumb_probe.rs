#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::time::Instant;

use biuviewer_decoder::{decode_static, decode_thumbnail, FormatDetector};

fn main() {
    let path = PathBuf::from(std::env::args().nth(1).expect("需要图片路径"));
    let bytes = std::fs::read(&path).unwrap();
    let format = FormatDetector::detect(&bytes).unwrap();

    let start = Instant::now();
    let full = decode_static(&bytes).unwrap();
    let full_ms = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    let thumb = decode_thumbnail(&bytes, format, 128).unwrap();
    let thumb_ms = start.elapsed().as_secs_f64() * 1000.0;

    println!(
        "{}: full {}x{} {:.1} ms; thumb {}x{} {:.1} ms",
        path.display(),
        full.width,
        full.height,
        full_ms,
        thumb.width,
        thumb.height,
        thumb_ms
    );
}
