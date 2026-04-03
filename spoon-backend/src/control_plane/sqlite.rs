//! SQLite control-plane database facade.
//!
//! Owns the repo-level tokio boundary over `rusqlite` and exposes async methods
//! for initialization. All driver-level details (raw SQL, `rusqlite::Connection`,
//! and `spawn_blocking`) stay inside this module and its sibling
//! [`migrations`] module; business-rule code in lifecycle, store, or projection
//! modules must never import them directly.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use crate::Result;

use super::migrations::run_migrations;

/// Backend-wide control-plane database handle.
///
/// Construct via [`ControlPlaneDb::open`], which creates (or opens) the
/// SQLite file and runs schema migrations. The resulting value can be
/// shared by cloning; each call opens a fresh SQLite connection under the hood
/// and keeps the blocking bridge inside this module.
#[derive(Clone)]
pub struct ControlPlaneDb {
    db_path: PathBuf,
}

impl ControlPlaneDb {
    /// Open (or create) the control-plane database at the given path and
    /// run all pending schema migrations.
    pub async fn open(db_path: &std::path::Path) -> Result<Self> {
        // Ensure the parent directory exists.
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| crate::BackendError::fs("create", parent, e))?;
        }

        let db_path = db_path.to_path_buf();
        initialize_database(db_path.clone()).await?;

        tracing::info!(db = %db_path.display(), "control-plane DB initialized");

        Ok(Self { db_path })
    }

    /// Open an ephemeral control-plane database. Useful for testing.
    ///
    /// Despite the historical method name, this uses a unique temporary file so
    /// state persists across multiple per-call connections while remaining local
    /// to the current test/process.
    pub async fn open_in_memory() -> Result<Self> {
        let unique_name = format!(
            "spoon-control-plane-{}-{}.sqlite3",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let db_path = std::env::temp_dir().join(unique_name);
        initialize_database(db_path.clone()).await?;
        Ok(Self { db_path })
    }

    /// Execute a closure on the underlying SQLite connection.
    ///
    /// This is the primary escape hatch for store/repository modules that
    /// need to run queries. The closure runs on the dedicated tokio-rusqlite
    /// thread, so it must be `Send + 'static`.
    ///
    /// The closure receives `&mut rusqlite::Connection` and may return
    /// `rusqlite::Result<T>`. The `rusqlite::Error` is automatically mapped
    /// into [`crate::Result`].
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> std::result::Result<T, rusqlite::Error>
            + Send
            + 'static,
    {
        let db_path = self.db_path.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = open_connection(&db_path)?;
            f(&mut conn).map_err(|e| crate::BackendError::external("control-plane DB call failed", e))
        })
        .await
        .map_err(|e| crate::BackendError::external("control-plane DB task join failed", e))?
    }
}

fn configure_connection(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    Ok(())
}

fn open_connection(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)
        .map_err(|e| crate::BackendError::external("failed to open control-plane DB", e))?;
    configure_connection(&conn)
        .map_err(|e| crate::BackendError::external("failed to configure control-plane DB", e))?;
    Ok(conn)
}

async fn initialize_database(db_path: PathBuf) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::BackendError::fs("create", parent, e))?;
        }
        let mut conn = open_connection(&db_path)?;
        run_migrations(&mut conn)
            .map_err(|e| crate::BackendError::external("control-plane migration failed", e))
    })
    .await
    .map_err(|e| crate::BackendError::external("control-plane DB init task join failed", e))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_db_opens_and_migrates() {
        let db = ControlPlaneDb::open_in_memory()
            .await
            .expect("in-memory DB should open");

        // Verify the migration version was advanced.
        let version: i64 = db
            .call(|conn| {
                Ok(conn
                    .query_row(
                        "PRAGMA user_version",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0))
            })
            .await
            .expect("schema query should succeed");

        assert_eq!(version, 2, "migrations 0001 and 0002 should be applied");
    }
}
