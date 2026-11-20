#![allow(clippy::unwrap_used, clippy::expect_used)]

use biuviewer_decoder::{decode_static, decode_thumbnail, FormatDetector, ImageFormat};

const HEIC: &[u8] = include_bytes!("fixtures/sample.heic");
const AVIF: &[u8] = include_bytes!("fixtures/sample.avif");

fn assert_sample(image: biuviewer_decoder::DecodedImage, format: ImageFormat) {
    assert_eq!(image.width, 256);
    assert_eq!(image.height, 256);
    assert_eq!(image.rgba.len(), 256 * 256 * 4);
    let center = (128 * 256 + 128) * 4;
    let pixel = &image.rgba[center..center + 4];
    assert!(
        pixel[0] < 40 && pixel[1] < 40 && pixel[2] < 40,
        "{format:?}"
    );
}

#[test]
fn detects_and_decodes_heic_fixture() {
    assert_eq!(FormatDetector::detect(HEIC).unwrap(), ImageFormat::Heic);
    assert_sample(decode_static(HEIC).unwrap(), ImageFormat::Heic);
}

#[test]
fn detects_and_decodes_avif_fixture() {
    assert_eq!(FormatDetector::detect(AVIF).unwrap(), ImageFormat::Avif);
    assert_sample(decode_static(AVIF).unwrap(), ImageFormat::Avif);
}

#[test]
fn heic_and_avif_thumbnails_are_downscaled() {
    let heic = decode_thumbnail(HEIC, ImageFormat::Heic, 128).unwrap();
    assert_eq!((heic.width, heic.height), (128, 128));
    let avif = decode_thumbnail(AVIF, ImageFormat::Avif, 128).unwrap();
    assert_eq!((avif.width, avif.height), (128, 128));
}

#[test]
fn truncated_iso_formats_return_errors_without_panicking() {
    for bytes in [&HEIC[..64], &AVIF[..64]] {
        let result = std::panic::catch_unwind(|| decode_static(bytes));
        assert!(result.is_ok(), "decoder panicked");
        assert!(result.unwrap().is_err());
    }
}
