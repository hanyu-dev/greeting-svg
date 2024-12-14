//! Counter implementation

mod db;

use std::{
    borrow::Cow,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc, LazyLock, OnceLock,
    },
};

use anyhow::Result;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use tokio::sync::mpsc;

// === Static variables ===

/// Counter map
static COUNTERS: LazyLock<DashMap<Arc<str>, AtomicU64, foldhash::fast::RandomState>> =
    LazyLock::new(|| {
        DashMap::with_capacity_and_hasher(8192, foldhash::fast::RandomState::default())
    });

/// Persistent storage tx
static DB_PERSISTENT_TX: OnceLock<mpsc::Sender<(Arc<str>, u64)>> = OnceLock::new();

// === Configs ===

/// New counter `access_key`
static CONF_ACCESS_KEY: OnceLock<ArcSwap<String>> = OnceLock::new();
/// Max number of counters
static CONF_MAX_COUNTERS: AtomicUsize = AtomicUsize::new(131072);

// === impls ===

/// Counter implementation
pub(crate) struct Counter;

impl Counter {
    /// Initialize the counter map
    pub(crate) async fn init(config: &crate::config::Config) {
        Self::update_config(config);

        // Persistent storage
        // No need to log here when error occurs
        let _ = db::Persistent::init()
            .await
            .map(|tx| DB_PERSISTENT_TX.set(tx));

        Self::insert_all(config.user_id.iter().map(|id| (id.clone(), 0)).collect());
    }

    /// Update counter related config from given
    /// [Config](crate::config::Config).
    pub(crate) fn update_config(config: &crate::config::Config) {
        // * Update max counters limits
        CONF_MAX_COUNTERS.store(config.max_counter, Ordering::Relaxed);

        // * Update access_key
        //
        // * If we have access_key set, we replace the old access_key with the new one.
        // * If new access_key is None, we do nothing and we have to restart the server.
        if config
            .access_key
            .as_ref()
            .is_some_and(|access_key| access_key.len() > 0)
        {
            let new_access_key = config.access_key.clone().unwrap();

            match CONF_ACCESS_KEY.get() {
                Some(access_key) => access_key.store(new_access_key),
                None => CONF_ACCESS_KEY.set(ArcSwap::new(new_access_key)).unwrap(),
            }
        }
    }

    /// Insert counters into [COUNTERS].
    pub(super) fn insert_all(counters: Vec<(Arc<str>, u64)>) {
        use rayon::prelude::*;
        counters.into_par_iter().for_each(|(id, count)| {
            tracing::debug!("Inserting counter {} with count {}", id, count);

            COUNTERS.insert(id, AtomicU64::new(count));
        });
    }

    #[inline]
    /// Increase the counter by 1, or create one if it doesn't exist.
    ///
    /// Will trigger the persistent storage write.
    pub(crate) async fn fetch_add(
        id: &str,
        access_key: Option<&Cow<'_, str>>,
        debug_mode: bool,
    ) -> Option<u64> {
        let id: Arc<str> = id.into();

        let current_count = COUNTERS.get(&id).map(|u| {
            if debug_mode {
                u.load(Ordering::Relaxed)
            } else {
                u.fetch_add(1, Ordering::AcqRel) + 1
            }
        });

        // ! verify access key when no corresponding counter exists
        if current_count.is_none()
            && access_key.zip((CONF_ACCESS_KEY).get()).is_some_and(
                |(access_key, desired_access_key)| access_key == desired_access_key.load().as_str(),
            )
        {
            Self::insert_counter(id.clone()).await;
            Self::persist_data_tx(id.clone(), 1).await;
            return Some(1);
        }

        if current_count.is_some() {
            // Save to database.
            Self::persist_data_tx(id, current_count.unwrap()).await;
        }

        current_count
    }

    #[inline]
    /// Insert a new counter
    async fn insert_counter(id: Arc<str>) {
        tokio::spawn(async move {
            tracing::info!("New counter for id [{}]", id);

            // Insert new counter
            COUNTERS.insert(id, AtomicU64::new(1));

            // Check capacity
            if COUNTERS.len() > CONF_MAX_COUNTERS.load(Ordering::Acquire) {
                tracing::warn!("Too many counters, trigger cleanup task...");

                tokio::spawn(async {
                    for item in COUNTERS.iter() {
                        if item.load(Ordering::Acquire) == 1 {
                            tracing::debug!("Cleanup counter: {}", item.key());
                            let id = item.key().clone();

                            // Will not block, since different tokio thread.
                            tokio::spawn(async move {
                                COUNTERS.remove(&id);
                            });
                        }
                    }
                });
            }
        });
    }

    #[inline]
    /// Send persistent data to the database
    ///
    /// If database is not ready, this will be actually a no-op
    async fn persist_data_tx(id: Arc<str>, new_count: u64) {
        tokio::spawn(async move {
            if let Some(tx) = DB_PERSISTENT_TX.get() {
                let _ = tx.send((id, new_count)).await;
            }
        });
    }

    /// Persist all data to the database
    pub(crate) async fn persist_all() -> Result<()> {
        for kv in COUNTERS.iter() {
            let id = kv.key().clone();
            let count = kv.load(Ordering::Acquire);

            if let Some(tx) = DB_PERSISTENT_TX.get() {
                let _ = tx.send((id, count)).await;
            }
        }

        Ok(())
    }
}
