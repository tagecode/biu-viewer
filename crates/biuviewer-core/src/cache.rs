use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct Entry<V> {
    value: Arc<V>,
    bytes: usize,
    tick: u64,
}

/// 按字节预算淘汰最久未使用的项。当前图片可通过 [`MemoryCache::set_pinned`] 固定，不参与淘汰。
pub struct MemoryCache<V> {
    entries: HashMap<PathBuf, Entry<V>>,
    clock: u64,
    used: usize,
    budget: usize,
    pinned: Option<PathBuf>,
}

/// 插入结果。`evicted` 中是被移出缓存的旧值，调用方用它释放 GPU 纹理。
pub struct InsertResult<V> {
    pub inserted: bool,
    pub evicted: Vec<(PathBuf, Arc<V>)>,
}

impl<V> MemoryCache<V> {
    pub fn new(budget: usize) -> Self {
        Self {
            entries: HashMap::new(),
            clock: 0,
            used: 0,
            budget,
            pinned: None,
        }
    }

    pub fn budget(&self) -> usize {
        self.budget
    }

    pub fn used(&self) -> usize {
        self.used
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, key: &Path) -> bool {
        self.entries.contains_key(key)
    }

    /// 命中并刷新使用时间。
    pub fn get(&mut self, key: &Path) -> Option<Arc<V>> {
        self.clock += 1;
        let tick = self.clock;
        let entry = self.entries.get_mut(key)?;
        entry.tick = tick;
        Some(entry.value.clone())
    }

    /// 只查看，不刷新使用时间。
    pub fn peek(&self, key: &Path) -> Option<Arc<V>> {
        self.entries.get(key).map(|entry| entry.value.clone())
    }

    pub fn set_pinned(&mut self, key: Option<PathBuf>) {
        self.pinned = key;
    }

    pub fn pinned(&self) -> Option<&Path> {
        self.pinned.as_deref()
    }

    pub fn insert(&mut self, key: PathBuf, value: Arc<V>, bytes: usize) -> InsertResult<V> {
        let mut evicted = Vec::new();
        if let Some(old) = self.entries.remove(&key) {
            self.used = self.used.saturating_sub(old.bytes);
            evicted.push((key.clone(), old.value));
        }

        if bytes > self.budget {
            return InsertResult {
                inserted: false,
                evicted,
            };
        }

        while self.used + bytes > self.budget {
            let Some(victim) = self
                .entries
                .iter()
                .filter(|(candidate, _)| self.pinned.as_deref() != Some(candidate.as_path()))
                .min_by_key(|(_, entry)| entry.tick)
                .map(|(candidate, _)| candidate.clone())
            else {
                return InsertResult {
                    inserted: false,
                    evicted,
                };
            };
            if let Some(removed) = self.entries.remove(&victim) {
                self.used = self.used.saturating_sub(removed.bytes);
                evicted.push((victim, removed.value));
            }
        }

        self.clock += 1;
        self.used += bytes;
        self.entries.insert(
            key,
            Entry {
                value,
                bytes,
                tick: self.clock,
            },
        );
        InsertResult {
            inserted: true,
            evicted,
        }
    }

    pub fn remove(&mut self, key: &Path) -> Option<Arc<V>> {
        let removed = self.entries.remove(key)?;
        self.used = self.used.saturating_sub(removed.bytes);
        Some(removed.value)
    }

    /// 清空缓存并返回所有值，便于释放纹理。
    pub fn clear(&mut self) -> Vec<(PathBuf, Arc<V>)> {
        self.used = 0;
        self.entries
            .drain()
            .map(|(key, entry)| (key, entry.value))
            .collect()
    }

    pub fn keys(&self) -> impl Iterator<Item = &PathBuf> {
        self.entries.keys()
    }

    pub fn entry_bytes(&self, key: &Path) -> Option<usize> {
        self.entries.get(key).map(|entry| entry.bytes)
    }
}

impl<V> Default for MemoryCache<V> {
    fn default() -> Self {
        Self::new(512 * 1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(name: &str) -> PathBuf {
        PathBuf::from(name)
    }

    #[test]
    fn evicts_least_recently_used_first() {
        let mut cache = MemoryCache::new(100);
        cache.insert(path("a.png"), Arc::new(1), 40);
        cache.insert(path("b.png"), Arc::new(2), 40);
        let _ = cache.get(&path("a.png"));
        let result = cache.insert(path("c.png"), Arc::new(3), 40);
        assert!(result.inserted);
        assert_eq!(result.evicted.len(), 1);
        assert_eq!(result.evicted[0].0, path("b.png"));
        assert_eq!(cache.used(), 80);
        assert!(cache.contains(&path("a.png")));
        assert!(cache.contains(&path("c.png")));
    }

    #[test]
    fn pinned_entry_is_never_evicted() {
        let mut cache = MemoryCache::new(100);
        cache.insert(path("current.png"), Arc::new(1), 60);
        cache.set_pinned(Some(path("current.png")));
        cache.insert(path("other.png"), Arc::new(2), 40);
        let result = cache.insert(path("third.png"), Arc::new(3), 40);
        assert!(result.inserted);
        assert_eq!(result.evicted.len(), 1);
        assert_eq!(result.evicted[0].0, path("other.png"));
        assert!(cache.contains(&path("current.png")));
        assert!(cache.contains(&path("third.png")));
        assert!(cache.used() <= cache.budget());
    }

    #[test]
    fn oversized_item_is_not_cached() {
        let mut cache = MemoryCache::new(100);
        let result = cache.insert(path("huge.png"), Arc::new(1), 200);
        assert!(!result.inserted);
        assert!(cache.is_empty());
        assert_eq!(cache.used(), 0);
    }

    #[test]
    fn inserting_existing_key_replaces_it() {
        let mut cache = MemoryCache::new(100);
        cache.insert(path("a.png"), Arc::new(1), 40);
        let result = cache.insert(path("a.png"), Arc::new(2), 30);
        assert!(result.inserted);
        assert_eq!(result.evicted.len(), 1);
        assert_eq!(cache.used(), 30);
        assert_eq!(*cache.get(&path("a.png")).unwrap(), 2);
    }

    #[test]
    fn remove_releases_bytes() {
        let mut cache = MemoryCache::new(100);
        cache.insert(path("a.png"), Arc::new(1), 40);
        assert!(cache.remove(&path("a.png")).is_some());
        assert_eq!(cache.used(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn clear_returns_all_entries() {
        let mut cache = MemoryCache::new(100);
        cache.insert(path("a.png"), Arc::new(1), 10);
        cache.insert(path("b.png"), Arc::new(2), 10);
        let cleared = cache.clear();
        assert_eq!(cleared.len(), 2);
        assert_eq!(cache.used(), 0);
    }

    #[test]
    fn repeated_get_refreshes_order() {
        let mut cache = MemoryCache::new(120);
        cache.insert(path("a.png"), Arc::new(1), 40);
        cache.insert(path("b.png"), Arc::new(2), 40);
        cache.insert(path("c.png"), Arc::new(3), 40);
        let _ = cache.get(&path("a.png"));
        let result = cache.insert(path("d.png"), Arc::new(4), 40);
        assert_eq!(result.evicted.len(), 1);
        assert_eq!(result.evicted[0].0, path("b.png"));
    }
}
