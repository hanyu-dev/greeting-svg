//! Linux.do cards, 300s cache

use std::{
    hash::Hash,
    sync::{Arc, LazyLock},
    time::Instant,
};

use dashmap::DashMap;
use macro_toolset::wrapper;

use super::model::UserInfo;

/// static CACHE map
static CACHE: LazyLock<Cache<Arc<str>, Arc<UserInfo>>> =
    LazyLock::new(|| Cache::with_capacity(128));

#[cfg(debug_assertions)]
/// Default cache TTL, 300s
static CACHE_TTL: u64 = 5;

#[cfg(not(debug_assertions))]
/// Default cache TTL, 300s
static CACHE_TTL: u64 = 300;

/// Get cache
pub(super) fn get_cache(key: &str) -> Option<(Arc<UserInfo>, bool)> {
    CACHE
        .get(key)
        .map(|v| (v.0.clone(), v.1.elapsed().as_secs() > CACHE_TTL))
}

/// Write cache
pub(super) async fn write_cache(key: impl Into<Arc<str>>, value: impl Into<Arc<UserInfo>>) {
    let value = value.into();
    if let Some(created) = value.created {
        let key = key.into();
        CACHE.insert(key, (value, created));

        if CACHE.len() > 5120 {
            tokio::spawn(async {
                CACHE.retain_ttl(CACHE_TTL);
            });
        }
    }
}

wrapper! {
    Cache<K, V>(DashMap<K, (V, Instant), foldhash::fast::RandomState>)
}

impl<K: Hash + Eq + Clone, V> Cache<K, V> {
    /// Create an empty [`Cache`] with capacity.
    fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: DashMap::with_capacity_and_hasher(
                capacity,
                foldhash::fast::RandomState::default(),
            ),
        }
    }

    /// Retain basing on TTL, default to be 300s.
    fn retain_ttl(&self, target_ttl: u64) {
        self.inner
            .retain(|_, v| v.1.elapsed().as_secs() <= target_ttl);
    }
}
