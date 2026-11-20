use biuviewer_core::ViewerError;

/// 静态图片格式。识别结果只描述容器，不表示解码已经成功。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
    Gif,
    Webp,
    Tiff,
    Ico,
    Heic,
    Avif,
}

/// 先看文件头魔数，识别不了时再用扩展名兜底。
pub struct FormatDetector;

impl FormatDetector {
    pub fn detect(bytes: &[u8]) -> Result<ImageFormat, ViewerError> {
        Self::detect_with_extension(bytes, None)
    }

    pub fn detect_with_extension(
        bytes: &[u8],
        extension: Option<&str>,
    ) -> Result<ImageFormat, ViewerError> {
        if let Some(format) = detect_magic(bytes) {
            return Ok(format);
        }
        if let Some(format) = extension.and_then(format_from_extension) {
            return Ok(format);
        }
        let label = extension.unwrap_or("未知");
        Err(ViewerError::Unsupported(label.to_string()))
    }

    pub fn supported_extensions() -> &'static [&'static str] {
        &[
            "png", "jpg", "jpeg", "bmp", "gif", "webp", "tif", "tiff", "ico", "heic", "heif",
            "avif",
        ]
    }
}

fn detect_magic(bytes: &[u8]) -> Option<ImageFormat> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(ImageFormat::Png);
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(ImageFormat::Jpeg);
    }
    if bytes.starts_with(b"BM") {
        return Some(ImageFormat::Bmp);
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some(ImageFormat::Gif);
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some(ImageFormat::Webp);
    }
    if bytes.starts_with(b"II*\x00") || bytes.starts_with(b"MM\x00*") {
        return Some(ImageFormat::Tiff);
    }
    if bytes.starts_with(&[0x00, 0x00, 0x01, 0x00]) {
        return Some(ImageFormat::Ico);
    }
    detect_iso_bmff(bytes)
}

/// 识别 HEIF/HEIC 与 AVIF。两者都是 ISO BMFF，靠 `ftyp` 的 brand 区分。
fn detect_iso_bmff(bytes: &[u8]) -> Option<ImageFormat> {
    if bytes.len() < 16 || &bytes[4..8] != b"ftyp" {
        return None;
    }
    let major = &bytes[8..12];
    let mut is_avif = major == b"avif" || major == b"avis" || major == b"avio";
    let mut is_heic = major == b"heic"
        || major == b"heix"
        || major == b"hevc"
        || major == b"hevx"
        || major == b"heim"
        || major == b"heis"
        || major == b"hevm"
        || major == b"hevs";
    let is_mif1 = major == b"mif1" || major == b"msf1";

    // 兼容 brand 列表：box header 16 字节后每 4 字节一个 brand。
    let mut offset = 16;
    while offset + 4 <= bytes.len() {
        let brand = &bytes[offset..offset + 4];
        if brand == b"avif" || brand == b"avis" || brand == b"avio" {
            is_avif = true;
        }
        if brand == b"heic"
            || brand == b"heix"
            || brand == b"hevc"
            || brand == b"hevx"
            || brand == b"heim"
            || brand == b"heis"
            || brand == b"hevm"
            || brand == b"hevs"
        {
            is_heic = true;
        }
        offset += 4;
    }

    if is_avif {
        Some(ImageFormat::Avif)
    } else if is_heic || is_mif1 {
        Some(ImageFormat::Heic)
    } else {
        None
    }
}

fn format_from_extension(extension: &str) -> Option<ImageFormat> {
    let normalized = extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    match normalized.as_str() {
        "png" => Some(ImageFormat::Png),
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "bmp" => Some(ImageFormat::Bmp),
        "gif" => Some(ImageFormat::Gif),
        "webp" => Some(ImageFormat::Webp),
        "tif" | "tiff" => Some(ImageFormat::Tiff),
        "ico" => Some(ImageFormat::Ico),
        "heic" | "heif" => Some(ImageFormat::Heic),
        "avif" => Some(ImageFormat::Avif),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_each_p0_format_from_magic() {
        let samples: &[(&[u8], ImageFormat)] = &[
            (b"\x89PNG\r\n\x1a\nrest", ImageFormat::Png),
            (b"\xFF\xD8\xFFrest", ImageFormat::Jpeg),
            (b"BMrest", ImageFormat::Bmp),
            (b"GIF89arest", ImageFormat::Gif),
            (b"GIF87arest", ImageFormat::Gif),
            (b"RIFF1234WEBP", ImageFormat::Webp),
            (b"II*\x00rest", ImageFormat::Tiff),
            (b"MM\x00*rest", ImageFormat::Tiff),
            (b"\x00\x00\x01\x00rest", ImageFormat::Ico),
        ];
        for (bytes, expected) in samples {
            assert_eq!(FormatDetector::detect(bytes).unwrap(), *expected);
        }
    }

    #[test]
    fn magic_wins_when_extension_is_wrong() {
        let png_named_jpg = b"\x89PNG\r\n\x1a\n";
        assert_eq!(
            FormatDetector::detect_with_extension(png_named_jpg, Some("jpg")).unwrap(),
            ImageFormat::Png
        );
    }

    #[test]
    fn extension_is_used_when_magic_is_missing() {
        assert_eq!(
            FormatDetector::detect_with_extension(b"not-a-header", Some("WEBP")).unwrap(),
            ImageFormat::Webp
        );
    }

    #[test]
    fn non_image_is_unsupported() {
        let err = FormatDetector::detect(b"hello world").unwrap_err();
        assert!(matches!(err, biuviewer_core::ViewerError::Unsupported(_)));
        assert!(err.user_message().contains("不支持"));
    }

    #[test]
    fn supported_extensions_cover_target_formats() {
        let extensions = FormatDetector::supported_extensions();
        for required in [
            "png", "jpg", "jpeg", "bmp", "gif", "webp", "tif", "tiff", "ico", "heic", "heif",
            "avif",
        ] {
            assert!(extensions.contains(&required), "missing {required}");
        }
    }

    fn ftyp(major: &[u8; 4], compatible: &[&[u8; 4]]) -> Vec<u8> {
        let size = 16 + compatible.len() * 4;
        let mut bytes = Vec::with_capacity(size);
        bytes.extend_from_slice(&(size as u32).to_be_bytes());
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(major);
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        for brand in compatible {
            bytes.extend_from_slice(*brand);
        }
        bytes
    }

    #[test]
    fn detects_heic_and_avif_brands() {
        assert_eq!(
            FormatDetector::detect(&ftyp(b"heic", &[b"mif1"])).unwrap(),
            ImageFormat::Heic
        );
        assert_eq!(
            FormatDetector::detect(&ftyp(b"mif1", &[b"avif"])).unwrap(),
            ImageFormat::Avif
        );
        assert_eq!(
            FormatDetector::detect(&ftyp(b"avif", &[b"mif1"])).unwrap(),
            ImageFormat::Avif
        );
        assert_eq!(
            FormatDetector::detect(&ftyp(b"mif1", &[])).unwrap(),
            ImageFormat::Heic
        );
    }

    #[test]
    fn heic_and_avif_extensions_are_recognized() {
        assert_eq!(
            FormatDetector::detect_with_extension(b"", Some("heif")).unwrap(),
            ImageFormat::Heic
        );
        assert_eq!(
            FormatDetector::detect_with_extension(b"", Some("AVIF")).unwrap(),
            ImageFormat::Avif
        );
    }
}
