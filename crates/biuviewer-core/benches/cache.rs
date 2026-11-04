//! 内存 LRU 缓存基准：命中、插入与淘汰路径。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use biuviewer_core::MemoryCache;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const MB: usize = 1024 * 1024;

fn path(index: usize) -> PathBuf {
    PathBuf::from(format!("/photos/photo-{index:05}.png"))
}

fn bench_cache_hit(c: &mut Criterion) {
    let mut cache = MemoryCache::new(512 * MB);
    for index in 0..200 {
        cache.insert(path(index), std::sync::Arc::new(vec![0u8; 2 * MB]), 2 * MB);
    }

    let mut cursor = 0usize;
    c.bench_function("cache-hit-2mb", |b| {
        b.iter(|| {
            cursor = (cursor + 7) % 200;
            black_box(cache.get(black_box(&path(cursor))));
        });
    });
}

fn bench_cache_insert_evict(c: &mut Criterion) {
    let block = std::sync::Arc::new(vec![0u8; MB]);
    c.bench_function("cache-insert-evict", |b| {
        let mut cache = MemoryCache::new(64 * MB);
        let mut index = 0usize;
        b.iter(|| {
            cache.insert(path(index), block.clone(), black_box(MB));
            index += 1;
        });
    });
}

criterion_group!(benches, bench_cache_hit, bench_cache_insert_evict);
criterion_main!(benches);
