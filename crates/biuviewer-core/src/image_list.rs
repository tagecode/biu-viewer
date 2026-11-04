use std::path::{Path, PathBuf};

/// 已排序的图片路径列表与当前索引。索引在空列表中也保持为 0，调用方用 `current_path` 判断。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageList {
    paths: Vec<PathBuf>,
    current: usize,
}

impl ImageList {
    pub fn new(paths: Vec<PathBuf>, current: usize) -> Self {
        let current = if paths.is_empty() {
            0
        } else {
            current.min(paths.len() - 1)
        };
        Self { paths, current }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    /// 在已扫描的路径中定位目标文件；找不到时停在首张。
    pub fn from_paths_and_target(paths: Vec<PathBuf>, target: &Path) -> Self {
        let index = paths.iter().position(|path| path == target).unwrap_or(0);
        Self::new(paths, index)
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn current_index(&self) -> usize {
        self.current
    }

    pub fn current_path(&self) -> Option<&Path> {
        self.paths.get(self.current).map(PathBuf::as_path)
    }

    pub fn get(&self, index: usize) -> Option<&Path> {
        self.paths.get(index).map(PathBuf::as_path)
    }

    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    pub fn position_of(&self, path: &Path) -> Option<usize> {
        self.paths.iter().position(|candidate| candidate == path)
    }

    pub fn set_index(&mut self, index: usize) -> bool {
        if index < self.paths.len() && index != self.current {
            self.current = index;
            return true;
        }
        false
    }

    /// 末张再往后保持不动。返回值表示索引是否改变。
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        if self.current + 1 < self.paths.len() {
            self.current += 1;
            return true;
        }
        false
    }

    /// 首张再往前保持不动。返回值表示索引是否改变。
    pub fn prev(&mut self) -> bool {
        if self.current > 0 {
            self.current -= 1;
            return true;
        }
        false
    }

    pub fn first(&mut self) -> bool {
        self.set_index(0)
    }

    pub fn last(&mut self) -> bool {
        if self.paths.is_empty() {
            return false;
        }
        self.set_index(self.paths.len() - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn empty_list_has_no_current_path() {
        let mut list = ImageList::empty();
        assert!(list.is_empty());
        assert_eq!(list.current_index(), 0);
        assert_eq!(list.current_path(), None);
        assert!(!list.next());
        assert!(!list.prev());
        assert!(!list.first());
        assert!(!list.last());
    }

    #[test]
    fn single_element_stays_on_boundaries() {
        let mut list = ImageList::new(paths(&["only.png"]), 0);
        assert!(!list.prev());
        assert!(!list.next());
        assert_eq!(list.current_path().unwrap(), Path::new("only.png"));
    }

    #[test]
    fn next_and_prev_stop_at_boundaries() {
        let mut list = ImageList::new(paths(&["a.png", "b.png", "c.png"]), 0);
        assert!(list.next());
        assert!(list.next());
        assert!(!list.next());
        assert_eq!(list.current_index(), 2);
        assert!(list.prev());
        assert!(list.prev());
        assert!(!list.prev());
        assert_eq!(list.current_index(), 0);
    }

    #[test]
    fn jump_clamps_and_reports_changes() {
        let mut list = ImageList::new(paths(&["a.png", "b.png", "c.png"]), 5);
        assert_eq!(list.current_index(), 2);
        assert!(list.set_index(1));
        assert!(!list.set_index(1));
        assert!(!list.set_index(9));
        assert_eq!(list.current_index(), 1);
    }

    #[test]
    fn first_and_last_jump_to_ends() {
        let mut list = ImageList::new(paths(&["a.png", "b.png", "c.png"]), 1);
        assert!(list.last());
        assert_eq!(list.current_index(), 2);
        assert!(list.first());
        assert_eq!(list.current_index(), 0);
    }

    #[test]
    fn from_paths_and_target_locates_file() {
        let all = paths(&["a1.png", "a2.png", "a10.png"]);
        let list = ImageList::from_paths_and_target(all.clone(), Path::new("a10.png"));
        assert_eq!(list.current_index(), 2);

        let missing = ImageList::from_paths_and_target(all, Path::new("gone.png"));
        assert_eq!(missing.current_index(), 0);
    }
}
