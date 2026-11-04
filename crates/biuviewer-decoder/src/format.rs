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
            "png", "jpg", "jpeg", "bmp", "gif", "webp", "tif", "tiff", "ico",
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
    None
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
    fn supported_extensions_cover_p0_formats() {
        let extensions = FormatDetector::supported_extensions();
        for required in [
            "png", "jpg", "jpeg", "bmp", "gif", "webp", "tif", "tiff", "ico",
        ] {
            assert!(extensions.contains(&required), "missing {required}");
        }
    }
}
