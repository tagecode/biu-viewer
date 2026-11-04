//! 解码与目录扫描基准。素材在内存中合成，避免把大文件提交到仓库。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::io::Cursor;
use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use image::{DynamicImage, ImageFormat, RgbaImage};

use biuviewer_decoder::{decode_static, FolderScanner};

/// 生成带渐变和噪点的图片，避免纯色图被编码器压缩成极小体积。
fn synthetic_image(width: u32, height: u32) -> DynamicImage {
    let mut image = RgbaImage::new(width, height);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let noise = ((x.wrapping_mul(31) ^ y.wrapping_mul(17)) & 0x3f) as u8;
        *pixel = image::Rgba([
            (x % 256) as u8,
            (y % 256) as u8,
            ((x + y) % 256) as u8 ^ noise,
            255,
        ]);
    }
    DynamicImage::ImageRgba8(image)
}

fn encode(image: &DynamicImage, format: ImageFormat) -> Vec<u8> {
    let mut buffer = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut buffer), format)
        .expect("合成素材应编码成功");
    buffer
}

fn bench_decode(c: &mut Criterion) {
    let cases = [
        (
            "png-4k",
            encode(&synthetic_image(3840, 2160), ImageFormat::Png),
        ),
        (
            "jpeg-4k",
            encode(&synthetic_image(3840, 2160), ImageFormat::Jpeg),
        ),
        (
            "png-12mp",
            encode(&synthetic_image(4000, 3000), ImageFormat::Png),
        ),
    ];

    let mut group = c.benchmark_group("decode");
    for (name, bytes) in &cases {
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        group.bench_function(*name, |b| {
            b.iter(|| decode_static(black_box(bytes)).expect("合成素材应解码成功"));
        });
    }
    group.finish();
}

fn bench_scan(c: &mut Criterion) {
    let dir = std::env::temp_dir().join("biuviewer-bench-scan");
    std::fs::create_dir_all(&dir).expect("创建临时目录");
    for index in 0..1000 {
        let path = dir.join(format!("photo-{index:04}.png"));
        if !path.exists() {
            let _ = std::fs::write(&path, b"placeholder");
        }
    }

    c.bench_function("scan-1000", |b| {
        b.iter(|| {
            let paths = FolderScanner::scan(black_box(&dir)).expect("扫描应成功");
            assert_eq!(paths.len(), 1000);
        });
    });

    let _ = std::fs::remove_dir_all(&dir);
}

fn bench_natural_sort(c: &mut Criterion) {
    let names: Vec<PathBuf> = (0..1000)
        .map(|index| PathBuf::from(format!("photo-{index}.png")))
        .collect();

    c.bench_function("natural-sort-1000", |b| {
        b.iter(|| {
            let mut sorted = names.clone();
            sorted.sort_by(|left, right| {
                biuviewer_decoder::natural_cmp(
                    &left.file_name().unwrap().to_string_lossy(),
                    &right.file_name().unwrap().to_string_lossy(),
                )
            });
            black_box(sorted);
        });
    });
}

criterion_group!(benches, bench_decode, bench_scan, bench_natural_sort);
criterion_main!(benches);
