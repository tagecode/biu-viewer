//! 生成手动性能测试素材。
//!
//! 用法: cargo run --release -p biuviewer-decoder --example make_assets -- /tmp/biu-bench

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;

use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame, Rgba, RgbaImage};

/// 确定性伪随机，保证每次生成的素材一致。
fn noise(seed: u32) -> u8 {
    let mut value = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    value ^= value >> 13;
    (value & 0xff) as u8
}

fn gradient(width: u32, height: u32, phase: u32) -> RgbaImage {
    let mut image = RgbaImage::new(width, height);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let shift = phase as u8;
        *pixel = Rgba([
            ((x * 255 / width.max(1)) as u8).wrapping_add(shift),
            ((y * 255 / height.max(1)) as u8).wrapping_add(shift / 2),
            shift.wrapping_mul(3),
            255,
        ]);
    }
    image
}

/// 渐变叠加颗粒，近似真实照片的压缩特征。
fn photo(width: u32, height: u32, phase: u32) -> RgbaImage {
    let mut image = gradient(width, height, phase);
    let mut seed = 0x9e37_79b9_u32;
    for pixel in image.pixels_mut() {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let grain = ((seed >> 27) & 0x1f) as i16 - 16;
        for channel in 0..3 {
            pixel[channel] = (i16::from(pixel[channel]) + grain).clamp(0, 255) as u8;
        }
    }
    image
}

fn noisy(width: u32, height: u32) -> RgbaImage {
    let mut image = RgbaImage::new(width, height);
    let mut seed = 0x1234_5678_u32;
    for pixel in image.pixels_mut() {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        *pixel = Rgba([
            (seed >> 24) as u8,
            (seed >> 16) as u8,
            (seed >> 8) as u8,
            (noise(seed) | 0x1f).max(0x20),
        ]);
    }
    image
}

fn main() {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "/tmp/biu-bench".into()),
    );
    let folder = root.join("1000-photos");
    fs::create_dir_all(&folder).unwrap();

    println!("生成 1000 张 400×300 PNG → {}", folder.display());
    for index in 0..1000u32 {
        let image = gradient(400, 300, index);
        image
            .save(folder.join(format!("photo-{index:04}.png")))
            .unwrap();
    }

    let big = root.join("large-10mb.png");
    println!("生成约 10MB 的噪声 PNG → {}", big.display());
    noisy(2000, 1500).save(&big).unwrap();

    let uhd = root.join("uhd-4k.png");
    println!("生成 3840×2160 PNG → {}", uhd.display());
    photo(3840, 2160, 7).save(&uhd).unwrap();

    let gif_path = root.join("animation.gif");
    println!("生成 40 帧 GIF → {}", gif_path.display());
    let file = fs::File::create(&gif_path).unwrap();
    let mut encoder = GifEncoder::new(BufWriter::new(file));
    encoder.set_repeat(Repeat::Infinite).unwrap();
    for frame_index in 0..40u32 {
        let mut image = gradient(600, 400, frame_index * 6);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            if (x + frame_index * 15) % 120 < 30 && y < 120 {
                *pixel = Rgba([255, 255, 255, 255]);
            }
        }
        let frame = Frame::from_parts(image, 0, 0, Delay::from_numer_denom_ms(80, 1));
        encoder.encode_frame(frame).unwrap();
    }

    let broken = root.join("corrupted.png");
    println!("生成损坏文件 → {}", broken.display());
    let mut bytes = fs::read(folder.join("photo-0000.png")).unwrap();
    bytes.truncate(bytes.len() / 2);
    fs::write(&broken, bytes).unwrap();

    let notes = root.join("notes.txt");
    fs::write(&notes, b"not an image").unwrap();

    println!("完成：{}", root.display());
}
