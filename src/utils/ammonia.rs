use std::{
    collections::VecDeque,
    sync::{Arc, LazyLock},
};

use dashmap::DashMap;
use parking_lot::Mutex;

/// Ammonia instance
static AMMONIA: LazyLock<ammonia::Builder<'static>> = LazyLock::new(ammonia::Builder::default);

/// Filtered notes
static AMMONIA_FILTERED_CACHE: LazyLock<
    DashMap<Arc<str>, Option<Arc<str>>, foldhash::fast::RandomState>,
> = LazyLock::new(DashMap::default);

/// LRU cache?
static AMMONIA_FILTERED_KEYS: Mutex<VecDeque<Arc<str>>> = Mutex::new(VecDeque::new());

/// The filtered notes counts limit
const AMMONIA_FILTERED_KEYS_LIMIT: usize = 8192;

/// Get filtered note
///
/// Notes: returning None means cache hit but not change, just left the same.
pub(crate) async fn get_filterd_note(note: impl AsRef<str>) -> Option<Arc<str>> {
    let note = note.as_ref();

    match AMMONIA_FILTERED_CACHE.get(note) {
        Some(cached_filtered) => {
            tracing::debug!("Filtered notes cache hit for note `{note}`");

            cached_filtered.value().clone()
        }
        None => {
            let note: Arc<str> = Arc::from(note);
            let filtered_note = AMMONIA.clean(&note).to_string();

            let filtered_note = if filtered_note == note.as_ref() {
                None
            } else {
                Some(Arc::from(filtered_note))
            };

            {
                let filtered_note = filtered_note.clone();
                tokio::spawn(async move {
                    AMMONIA_FILTERED_CACHE.insert(note.clone(), filtered_note);

                    let mut keys = AMMONIA_FILTERED_KEYS.lock();

                    keys.push_front(note);

                    if keys.len() > AMMONIA_FILTERED_KEYS_LIMIT {
                        tokio::spawn(async {
                            tracing::warn!("Too many cached notes, performing cleaning");

                            for _ in 0..(AMMONIA_FILTERED_KEYS_LIMIT / 2) {
                                let note = { AMMONIA_FILTERED_KEYS.lock().pop_back() };

                                if let Some(note) = note {
                                    AMMONIA_FILTERED_CACHE.remove(&note);
                                } else {
                                    break;
                                }
                            }
                        });
                    }
                });
            }

            filtered_note
        }
    }
}
