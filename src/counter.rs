//! Counter implementation

mod db;

use std::{
    borrow::Cow,
    net::IpAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, LazyLock, OnceLock,
    },
};

use anyhow::{bail, Result};
use axum::http::StatusCode;
use dashmap::DashMap;
use tokio::sync::mpsc;

use crate::{config::CONF_MAX_COUNTERS, utils::auth};

// === Static variables ===

/// Counter map
static COUNTERS: LazyLock<DashMap<Arc<str>, AtomicU64, foldhash::fast::RandomState>> =
    LazyLock::new(|| {
        DashMap::with_capacity_and_hasher(8192, foldhash::fast::RandomState::default())
    });

/// Persistent storage tx
static DB_PERSISTENT_TX: OnceLock<mpsc::Sender<(Arc<str>, Option<u64>)>> = OnceLock::new();

// === impls ===

/// Counter implementation
pub(crate) struct Counter;

impl Counter {
    /// Initialize the counter map
    pub(crate) async fn init(config: &crate::config::Config) {
        // Persistent storage
        // No need to log here when error occurs
        let _ = db::Persistent::init()
            .await
            .map(|tx| DB_PERSISTENT_TX.set(tx));

        Self::insert_all(config.user_id.iter().map(|id| (id.clone(), 0)).collect());
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
        remote_ip: Option<IpAddr>,
    ) -> Option<u64> {
        let id: Arc<str> = id.into();

        let current_count = COUNTERS.get(&id).map(|u| {
            if debug_mode {
                u.load(Ordering::Relaxed)
            } else {
                u.fetch_add(1, Ordering::AcqRel) + 1
            }
        });

        if debug_mode {
            tracing::debug!("Debug mode enabled, no count increase");

            return current_count;
        }

        if current_count.is_some() {
            // Save to database.
            Self::persist_data_tx(id, current_count).await;
        } else if auth(access_key, remote_ip) {
            Self::insert_new_counter(id).await;
            return Some(1);
        } else {
            // do nothing, access key does not match
            tracing::warn!("Access key incorrect or config not set: {access_key:?}");
        }

        current_count
    }

    #[inline]
    #[tracing::instrument(level = "debug")]
    /// Delete a counter
    pub(crate) async fn delete(
        id: &str,
        access_key: Option<&Cow<'_, str>>,
        remote_ip: Option<IpAddr>,
    ) -> Result<()> {
        if !auth(access_key, remote_ip) {
            tracing::warn!("Access key incorrect or config not set");
            bail!(StatusCode::UNAUTHORIZED)
        }

        // Delete counter
        if let Some((_, count)) = COUNTERS.remove(id) {
            tracing::debug!(
                "Deleted counter with count {}",
                count.load(Ordering::Relaxed)
            );

            Self::persist_data_tx(id.into(), None).await;
        } else {
            tracing::debug!("Counter not found for [{id}]");
            bail!(StatusCode::NOT_FOUND)
        }

        Ok(())
    }

    #[inline]
    /// Insert a new counter
    async fn insert_new_counter(id: Arc<str>) {
        tokio::spawn(async move {
            tracing::info!("New counter for id [{}]", id);

            // Insert new counter
            COUNTERS.insert(id.clone(), AtomicU64::new(1));

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

            Self::persist_data_tx(id, Some(1)).await;
        });
    }

    #[inline]
    /// Send persistent data to the database
    ///
    /// If database is not ready, this will be actually a no-op
    async fn persist_data_tx(id: Arc<str>, new_count: Option<u64>) {
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
                let _ = tx.send((id, Some(count))).await;
            }
        }

        Ok(())
    }
}
