//! Linux.do cards, 300s cache

use std::{
    collections::VecDeque,
    hash::Hash,
    sync::{Arc, LazyLock, OnceLock},
    time::{Duration, Instant},
};

use dashmap::DashMap;
use macro_toolset::wrapper;
use parking_lot::Mutex;
use tokio::task::JoinHandle;

use super::{model::UserInfo, upstream};

type CacheUpdateQueue = Mutex<VecDeque<(Arc<str>, Instant)>>;

#[cfg(not(debug_assertions))]
/// Default cache TTL, 300s
const CACHE_TTL: u64 = 300;

#[cfg(debug_assertions)]
/// Default cache TTL, 300s
const CACHE_TTL: u64 = 15;

/// Desired max key count, 5120
const DESIRED_MAX_KEY_COUNT: usize = 5120;

static FETCH_PROCESSING: LazyLock<DashMap<Arc<str>, (), foldhash::fast::RandomState>> =
    LazyLock::new(|| {
        DashMap::with_capacity_and_hasher(128, foldhash::fast::RandomState::default())
    });

/// static CACHE map
static CACHE: LazyLock<Cache<Arc<str>, Arc<UserInfo>>> =
    LazyLock::new(|| Cache::with_capacity(DESIRED_MAX_KEY_COUNT));

static CACHE_UPDATE_QUEUE: LazyLock<CacheUpdateQueue> =
    LazyLock::new(|| Mutex::new(VecDeque::with_capacity(DESIRED_MAX_KEY_COUNT)));

static CACHE_QUEUE_TASK_HANDLE: OnceLock<JoinHandle<()>> = OnceLock::new();

pub(super) async fn try_init_cache_update_queue() {
    if CACHE_QUEUE_TASK_HANDLE.get().is_none() {
        let _ = CACHE_QUEUE_TASK_HANDLE.set(tokio::spawn(async {
            const CACHE_TTL_DUR: Duration = Duration::from_secs(CACHE_TTL);

            let mut to_update: Option<Arc<str>> = None;
            let mut to_sleep: Option<Duration> = None;
            loop {
                if let Some(to_sleep) = to_sleep.take() {
                    tracing::trace!(to_sleep = ?to_sleep, "cache_update_queue: to sleep!");
                    tokio::time::sleep(to_sleep).await;
                }

                if let Some(to_update) = to_update.take() {
                    tracing::trace!(to_update = ?to_update, "cache_update_queue: to update!");
                    tokio::spawn(async move {
                        match upstream::fetch(&to_update).await {
                            Ok(value) => write_cache(value).await,
                            Err(e) => {
                                tracing::error!(
                                    to_update = to_update.as_ref(),
                                    "background update error: {e:#?}"
                                );
                            },
                        };
                    });
                }

                {
                    let value = CACHE_UPDATE_QUEUE.lock().pop_back();
                    if let Some((key, instant)) = value {
                        tracing::trace!(key = ?key, "cache_update_queue: handling!");

                        if let Some(to_sleep_dur) = CACHE_TTL_DUR.checked_sub(instant.elapsed()) {
                            to_sleep.replace(to_sleep_dur);
                        } else {
                            tracing::debug!(key = ?key, "cache_update_queue: cache expired, refreshing")
                        }

                        to_update.replace(key);
                    } else {
                        tracing::trace!("No cache queue tasks...");
                        to_sleep.replace(CACHE_TTL_DUR);
                    }
                }
            }
        }));
    }
}

#[must_use = "must handle if need fetch!!!"]
/// Get cache
///
/// Returns: data, update async task (you may await it or just throw it away)
pub(super) fn get_cache_or_fetch(
    key: &str,
    authorized: bool,
) -> (
    Option<Arc<UserInfo>>,
    Option<impl Future + Send + Sync + 'static>,
) {
    let (data, key) = match CACHE.get(key) {
        Some(v) => (
            Some(v.0.clone()),
            if v.1.elapsed().as_secs() > CACHE_TTL {
                Some(key.into())
            } else {
                None
            },
        ),
        None => (None, {
            let key: Arc<str> = key.into();

            match FETCH_PROCESSING.entry(key.clone()) {
                dashmap::Entry::Vacant(v) => {
                    v.insert(());
                    Some(key)
                }
                dashmap::Entry::Occupied(_) => {
                    tracing::debug!(key = ?key, "Processing, just wait...");
                    None
                }
            }
        }),
    };

    let async_task = key.map(|key| async move {
        if authorized {
            tracing::debug!("Cache missed, try fetch in background");

            tokio::spawn(async move {
                match upstream::fetch(&key).await {
                    Ok(value) => write_cache(value).await,
                    Err(e) => {
                        tracing::error!("Fetch upstream data error: {e:#?}");
                    }
                }
            });
        } else {
            tracing::debug!("Cache missed, but not authorized user!");
        }
    });

    (data, async_task)
}

/// Write cache
pub(super) async fn write_cache(value: impl Into<Arc<UserInfo>>) {
    let value = value.into();
    if let Some(created) = value.created {
        let key = value.user.username.clone();
        FETCH_PROCESSING.remove(&key);
        CACHE.insert(key.clone(), (value, created));

        if CACHE.len() > DESIRED_MAX_KEY_COUNT {
            tokio::spawn(async {
                CACHE.retain_ttl(CACHE_TTL);
            });
        }

        let mut cache_update_queue = CACHE_UPDATE_QUEUE.lock();
        cache_update_queue.push_front((key, created));
        if cache_update_queue.len() > DESIRED_MAX_KEY_COUNT {
            drop(cache_update_queue);
            tokio::spawn(async {
                CACHE_UPDATE_QUEUE
                    .lock()
                    .truncate(DESIRED_MAX_KEY_COUNT / 2);
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
