use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

use biuviewer_core::ViewerError;

use crate::format::FormatDetector;

/// 非递归扫描目录中的图片文件，按文件名自然排序。
pub struct FolderScanner;

impl FolderScanner {
    pub fn scan(dir: &Path) -> Result<Vec<PathBuf>, ViewerError> {
        let entries = fs::read_dir(dir).map_err(ViewerError::from)?;
        let mut paths = Vec::new();
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    tracing::warn!("跳过无法读取的目录项: {err}");
                    continue;
                }
            };
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            match entry.file_type() {
                Ok(file_type) if file_type.is_file() => {}
                _ => continue,
            }
            let path = entry.path();
            if is_supported_path(&path) {
                paths.push(path);
            }
        }
        paths.sort_by(|a, b| natural_cmp(&file_name(a), &file_name(b)));
        Ok(paths)
    }

    /// 扫描单个文件所在的目录，并返回该文件在列表中的位置。
    pub fn scan_for_file(path: &Path) -> Result<(Vec<PathBuf>, usize), ViewerError> {
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let paths = Self::scan(parent)?;
        let target = normalize(path);
        let index = paths
            .iter()
            .position(|candidate| normalize(candidate) == target)
            .unwrap_or(0);
        Ok((paths, index))
    }
}

/// 按扩展名判断是否为受支持的图片。扫描阶段不读文件头，避免逐文件 IO。
pub fn is_supported_path(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };
    let extension = extension.to_ascii_lowercase();
    FormatDetector::supported_extensions().contains(&extension.as_str())
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn normalize(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// 自然排序：数字段按数值比较，其余字符不区分 ASCII 大小写，末尾用原始字符兜底保证稳定。
pub fn natural_cmp(left: &str, right: &str) -> Ordering {
    let mut left_chars = left.chars().peekable();
    let mut right_chars = right.chars().peekable();

    loop {
        match (left_chars.peek().copied(), right_chars.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(a), Some(b)) if a.is_ascii_digit() && b.is_ascii_digit() => {
                let left_run = take_digits(&mut left_chars);
                let right_run = take_digits(&mut right_chars);
                let ordering = compare_digit_runs(&left_run, &right_run);
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            (Some(a), Some(b)) => {
                left_chars.next();
                right_chars.next();
                let ordering = compare_chars(a, b);
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
        }
    }
}

fn take_digits(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut digits = String::new();
    while let Some(character) = chars.peek().copied() {
        if !character.is_ascii_digit() {
            break;
        }
        digits.push(character);
        chars.next();
    }
    digits
}

fn compare_digit_runs(left: &str, right: &str) -> Ordering {
    let left_trimmed = left.trim_start_matches('0');
    let right_trimmed = right.trim_start_matches('0');
    match left_trimmed.len().cmp(&right_trimmed.len()) {
        Ordering::Equal => match left_trimmed.cmp(right_trimmed) {
            // 数值相同时按原始写法兜底，例如 img2 与 img02 的先后保持稳定。
            Ordering::Equal => left.cmp(right),
            ordering => ordering,
        },
        ordering => ordering,
    }
}

fn compare_chars(left: char, right: char) -> Ordering {
    let left_lower = left.to_lowercase().collect::<String>();
    let right_lower = right.to_lowercase().collect::<String>();
    match left_lower.cmp(&right_lower) {
        Ordering::Equal => left.cmp(&right),
        ordering => ordering,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("biuviewer-folder-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn touch(dir: &Path, name: &str) {
        let mut file = File::create(dir.join(name)).unwrap();
        file.write_all(b"x").unwrap();
    }

    #[test]
    fn natural_sort_orders_numbers_by_value() {
        let mut names = vec!["img10.png", "img2.png", "img1.png", "img20.png"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(
            names,
            vec!["img1.png", "img2.png", "img10.png", "img20.png"]
        );
    }

    #[test]
    fn natural_sort_is_case_insensitive_and_stable() {
        assert_eq!(natural_cmp("Photo2.png", "photo10.png"), Ordering::Less);
        assert_eq!(natural_cmp("A.png", "a.png"), Ordering::Less);
        assert_eq!(natural_cmp("photo.PNG", "Photo.png"), Ordering::Greater);
    }

    #[test]
    fn natural_sort_handles_chinese_names() {
        let mut names = vec!["图10.png", "图2.png", "图1.png", "风景.png"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(names, vec!["图1.png", "图2.png", "图10.png", "风景.png"]);
    }

    #[test]
    fn leading_zeros_do_not_break_ordering() {
        assert_eq!(natural_cmp("img02.png", "img2.png"), Ordering::Less);
        assert_eq!(natural_cmp("img002.png", "img010.png"), Ordering::Less);
    }

    #[test]
    fn scan_filters_hidden_and_unsupported_files() {
        let dir = temp_dir("filter");
        touch(&dir, "b10.png");
        touch(&dir, "b2.JPG");
        touch(&dir, "a1.webp");
        touch(&dir, ".hidden.png");
        touch(&dir, "notes.txt");
        fs::create_dir_all(dir.join("nested.png")).unwrap();

        let paths = FolderScanner::scan(&dir).unwrap();
        let names: Vec<String> = paths
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["a1.webp", "b2.JPG", "b10.png"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_directory_returns_empty_list() {
        let dir = temp_dir("empty");
        assert!(FolderScanner::scan(&dir).unwrap().is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_directory_maps_to_not_found() {
        let dir = std::env::temp_dir().join("biuviewer-folder-missing-does-not-exist");
        let err = FolderScanner::scan(&dir).unwrap_err();
        assert!(matches!(err, ViewerError::NotFound(_)));
    }

    #[test]
    fn scan_for_file_locates_the_requested_file() {
        let dir = temp_dir("locate");
        touch(&dir, "a1.png");
        touch(&dir, "a2.png");
        touch(&dir, "a10.png");

        let (paths, index) = FolderScanner::scan_for_file(&dir.join("a10.png")).unwrap();
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[index].file_name().unwrap(), "a10.png");
        assert_eq!(index, 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_for_file_falls_back_to_first_when_missing() {
        let dir = temp_dir("missing-file");
        touch(&dir, "a1.png");
        let (paths, index) = FolderScanner::scan_for_file(&dir.join("gone.png")).unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(index, 0);
        let _ = fs::remove_dir_all(&dir);
    }
}
