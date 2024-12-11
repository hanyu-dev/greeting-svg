//! Database Implementation
//!
//! - `Sqlite`
//! - `PostgreSQL` TODO

use std::{
    path::Path,
    sync::{Arc, OnceLock},
};

use anyhow::{anyhow, bail, Context, Result};
use tokio::sync::{mpsc, Semaphore};

#[cfg(feature = "sqlite")]
/// Database pool: `sqlite`
static DB_POOL_SQLITE: OnceLock<deadpool_sqlite::Pool> = OnceLock::new();

/// Persistent storage
///
/// - `sqlite`
pub(super) struct Persistent;

impl Persistent {
    #[tracing::instrument(err)]
    pub(super) async fn init() -> Result<mpsc::Sender<(Arc<str>, u64)>> {
        // init database
        #[cfg(feature = "sqlite")]
        SqliteImpl::init().await?;

        let (tx, mut rx) = mpsc::channel(1024);

        let permit = Arc::new(Semaphore::new(1));

        tokio::spawn(async move {
            while let Some((id, count)) = rx.recv().await {
                let permit = permit.clone();

                tokio::spawn(async move {
                    let _permit = permit.acquire().await.unwrap();

                    if let Err(e) = SqliteImpl::sqlite_write(id, count as i64).await {
                        tracing::error!("Write to sqlite error: {}", e);
                    }
                });
            }
        });

        Ok(tx)
    }
}

#[cfg(feature = "sqlite")]
struct SqliteImpl;

#[cfg(feature = "sqlite")]
impl SqliteImpl {
    /// Initialize sqlite database
    async fn init() -> Result<()> {
        let path = Path::new("./db.sqlite3");

        let new_db = !path.exists();
        let pool =
            deadpool_sqlite::Config::new(path).create_pool(deadpool_sqlite::Runtime::Tokio1)?;

        if new_db {
            tracing::debug!("New database, create tables...");

            if let Err(e) = pool
                    .get()
                    .await?
                    .interact(|conn| {
                        conn.prepare(
                            r#"CREATE TABLE IF NOT EXISTS counters ( id TEXT PRIMARY KEY, count INTERGER NOT NULL DEFAULT 0);"#,
                        )
                        .unwrap()
                        .execute([])
                        .unwrap();
                    })
                    .await
                {
                    bail!("Failed to initialize database: {}", e);
                }
        }

        DB_POOL_SQLITE
            .set(pool)
            .expect("SQLite DB pool cannot be initialized one more time");

        tracing::info!("SQLite DB initialized");

        if !new_db {
            // Load data from DB
            super::Counter::insert_all(SqliteImpl::sqlite_get_all().await?);
        }

        Ok(())
    }

    /// Read all counters from sqlite
    pub(super) async fn sqlite_get_all() -> Result<Vec<(Arc<str>, u64)>> {
        let result = DB_POOL_SQLITE
            .get()
            .context("SQLite DB not initialized")?
            .get()
            .await?
            .interact(move |conn| -> Result<Vec<(Arc<str>, u64)>> {
                let mut stmt = conn.prepare("SELECT * FROM counters")?;

                // Terrible code, `rusqlite` is really a mess...
                let rows = stmt.query_map([], |row| {
                    Ok((row.get::<_, Arc<str>>(0)?, row.get::<_, i64>(1)? as u64))
                })?;

                let results = rows.filter_map(|row| row.ok()).collect();

                Ok(results)
            })
            .await
            .map_err(|e| anyhow!("{:#?}", e));

        match result {
            Ok(result) => result,
            Err(e) => Err(e),
        }
    }

    #[cfg(test)]
    async fn sqlite_get(id: Arc<str>) -> Result<Option<u64>> {
        let result = DB_POOL_SQLITE
            .get()
            .context("SQLite DB not initialized")?
            .get()
            .await?
            .interact(move |conn| -> Result<Option<u64>> {
                let mut stmt = conn.prepare("SELECT * FROM counters where id=?")?;

                // Make compiler happy, do not just wrap the following in Ok
                let result = match stmt.query_map([id], |row| row.get::<_, i64>(1))?.next() {
                    Some(Ok(count)) => Some(count as u64),
                    Some(Err(e)) => Err(e)?,
                    None => None,
                };

                Ok(result)
            })
            .await
            .map_err(|e| anyhow!("{:#?}", e));

        match result {
            Ok(result) => result,
            Err(e) => Err(e),
        }
    }

    #[inline]
    /// Write to `SQLite`
    async fn sqlite_write(id: Arc<str>, count: i64) -> Result<()> {
        DB_POOL_SQLITE
            .get()
            .context("SQLite DB not initialized")?
            .get()
            .await?
            .interact(move |conn| {
                conn.execute(
                    "INSERT OR REPLACE INTO counters (id, count) VALUES (?1, ?2)",
                    (&id, count),
                )
                .unwrap();
            })
            .await
            .map_err(|e| anyhow!("{:#?}", e))
    }
}

#[tokio::test]
async fn test_sqlite() {
    macro_toolset::init_tracing_simple!();

    let _ = Persistent::init().await;

    SqliteImpl::sqlite_write("test_data".into(), (u64::MAX - 1) as i64)
        .await
        .unwrap();

    assert_eq!(
        SqliteImpl::sqlite_get("test_data".into())
            .await
            .unwrap()
            .unwrap(),
        (u64::MAX - 1)
    );

    SqliteImpl::sqlite_get_all().await.unwrap();
}
