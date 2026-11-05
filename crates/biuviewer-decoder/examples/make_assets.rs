//! 生成手动性能测试素材。
//!
//! 用法: cargo run --release -p biuviewer-decoder --example make_assets -- /tmp/biu-bench
//!
//! 产出：`1000-photos/`（翻页与内存）、大图与 GIF、错误样本、`formats/`（7 种 P0 格式）、
//! `exif/`（8 种 EXIF 方向）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;

use image::codecs::gif::{GifEncoder, Repeat};
use image::codecs::ico::IcoEncoder;
use image::codecs::webp::WebPEncoder;
use image::metadata::Orientation;
use image::{
    Delay, DynamicImage, ExtendedColorType, Frame, ImageEncoder, ImageFormat, Rgba, RgbaImage,
};

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

/// 带四角色块与顶部竖条的参考图：8 张 EXIF 样张正确显示时应完全一致。
fn markers(width: u32, height: u32) -> RgbaImage {
    let mut image = RgbaImage::from_pixel(width, height, Rgba([235, 235, 235, 255]));
    let block_w = width / 4;
    let block_h = height / 4;
    let corners = [
        ((0, 0), [220, 40, 40, 255]),
        ((width - block_w, 0), [40, 180, 60, 255]),
        ((0, height - block_h), [40, 80, 220, 255]),
        ((width - block_w, height - block_h), [240, 210, 40, 255]),
    ];
    for ((x0, y0), color) in corners {
        for y in y0..y0 + block_h {
            for x in x0..x0 + block_w {
                image.put_pixel(x, y, Rgba(color));
            }
        }
    }
    for y in height / 12..height / 3 {
        for x in width / 2 - block_w / 6..width / 2 + block_w / 6 {
            image.put_pixel(x, y, Rgba([20, 20, 20, 255]));
        }
    }
    image
}

/// EXIF 规范定义的“存储像素 → 正立显示”变换，独立于 image crate 实现。
fn upright(source: &RgbaImage, orientation: u8) -> RgbaImage {
    let (width, height) = source.dimensions();
    let (out_w, out_h) = if orientation >= 5 {
        (height, width)
    } else {
        (width, height)
    };
    let mut output = RgbaImage::new(out_w, out_h);
    for y in 0..out_h {
        for x in 0..out_w {
            let (sx, sy) = match orientation {
                1 => (x, y),
                2 => (width - 1 - x, y),
                3 => (width - 1 - x, height - 1 - y),
                4 => (x, height - 1 - y),
                5 => (y, x),
                6 => (y, height - 1 - x),
                7 => (width - 1 - y, height - 1 - x),
                _ => (width - 1 - y, x),
            };
            output.put_pixel(x, y, *source.get_pixel(sx, sy));
        }
    }
    output
}

/// 与 image crate 的 `apply_orientation` 交叉验证，防止样张按错误的理解生成。
fn verify_orientation_table(reference: &RgbaImage) {
    for orientation in 1..=8u8 {
        let expected = upright(reference, orientation);
        let mut dynamic = DynamicImage::ImageRgba8(reference.clone());
        dynamic.apply_orientation(Orientation::from_exif(orientation).unwrap());
        let actual = dynamic.to_rgba8();
        assert_eq!(
            expected.dimensions(),
            actual.dimensions(),
            "orientation {orientation} 尺寸不一致"
        );
        assert!(
            expected.pixels().eq(actual.pixels()),
            "orientation {orientation} 像素不一致"
        );
    }
}

fn exif_orientation_chunk(orientation: u16) -> Vec<u8> {
    let mut chunk = vec![
        b'I', b'I', 0x2A, 0x00, // 小端 TIFF
        0x08, 0x00, 0x00, 0x00, // IFD 偏移 8
        0x01, 0x00, // 一个条目
        0x12, 0x01, // tag 0x0112 Orientation
        0x03, 0x00, // 类型 SHORT
        0x01, 0x00, 0x00, 0x00, // 数量 1
    ];
    chunk.extend(orientation.to_le_bytes());
    chunk.extend([0, 0]);
    chunk
}

/// 写 JPEG 并在 SOI 之后插入 EXIF APP1 段。
fn write_jpeg_with_exif(path: &Path, image: &RgbaImage, orientation: u8) {
    let mut cursor = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(image.clone())
        .to_rgb8()
        .write_to(&mut cursor, ImageFormat::Jpeg)
        .unwrap();
    let jpeg = cursor.into_inner();

    let mut payload = b"Exif\0\0".to_vec();
    payload.extend_from_slice(&exif_orientation_chunk(u16::from(orientation)));
    let length = u16::try_from(payload.len() + 2).unwrap();

    let mut output = Vec::with_capacity(jpeg.len() + payload.len() + 4);
    output.extend_from_slice(&[0xFF, 0xD8, 0xFF, 0xE1]);
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(&payload);
    output.extend_from_slice(&jpeg[2..]);
    fs::write(path, output).unwrap();
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

    write_format_samples(&root);
    write_exif_samples(&root);

    println!("完成：{}", root.display());
}

/// 7 种 P0 格式各一张，用于 UI 逐格式验收。
fn write_format_samples(root: &Path) {
    let folder = root.join("formats");
    fs::create_dir_all(&folder).unwrap();
    println!("生成 7 种格式样张 → {}", folder.display());

    let sample = photo(320, 240, 42);
    let dynamic = DynamicImage::ImageRgba8(sample.clone());
    for (name, format) in [
        ("sample.png", ImageFormat::Png),
        ("sample.jpg", ImageFormat::Jpeg),
        ("sample.bmp", ImageFormat::Bmp),
        ("sample.tiff", ImageFormat::Tiff),
    ] {
        dynamic
            .write_to(&mut fs::File::create(folder.join(name)).unwrap(), format)
            .unwrap();
    }

    let gif = fs::File::create(folder.join("sample.gif")).unwrap();
    GifEncoder::new(BufWriter::new(gif))
        .encode_frame(Frame::from_parts(
            sample.clone(),
            0,
            0,
            Delay::from_numer_denom_ms(100, 1),
        ))
        .unwrap();

    let webp = fs::File::create(folder.join("sample.webp")).unwrap();
    WebPEncoder::new_lossless(BufWriter::new(webp))
        .write_image(
            sample.as_raw(),
            sample.width(),
            sample.height(),
            ExtendedColorType::Rgba8,
        )
        .unwrap();

    let icon = image::imageops::resize(&sample, 128, 128, image::imageops::FilterType::Triangle);
    let ico = fs::File::create(folder.join("sample.ico")).unwrap();
    IcoEncoder::new(BufWriter::new(ico))
        .write_image(
            icon.as_raw(),
            icon.width(),
            icon.height(),
            ExtendedColorType::Rgba8,
        )
        .unwrap();
}

/// 8 种 EXIF 方向的 JPEG。存储像素各不相同，按方向校正后应显示为同一张正立图。
fn write_exif_samples(root: &Path) {
    let folder = root.join("exif");
    fs::create_dir_all(&folder).unwrap();
    println!("生成 8 种 EXIF 方向样张 → {}", folder.display());

    let reference = markers(240, 160);
    verify_orientation_table(&reference);

    for orientation in 1..=8u8 {
        // D4 群中每个变换都有唯一的逆，逐个试出能还原成参考图的存储像素。
        let stored = (1..=8u8)
            .map(|candidate| upright(&reference, candidate))
            .find(|stored| upright(stored, orientation) == reference)
            .unwrap();
        write_jpeg_with_exif(
            &folder.join(format!("orientation-{orientation}.jpg")),
            &stored,
            orientation,
        );
    }
}
