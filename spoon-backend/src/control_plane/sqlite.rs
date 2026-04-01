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
use crate::layout::RuntimeLayout;

use super::migrations::run_migrations;

#[derive(Debug, Clone)]
enum ControlPlaneLocation {
    File(PathBuf),
    EphemeralFile(PathBuf),
}

impl ControlPlaneLocation {
    fn path(&self) -> &Path {
        match self {
            Self::File(path) | Self::EphemeralFile(path) => path,
        }
    }
}

pub fn db_path_for_layout(layout: &RuntimeLayout) -> PathBuf {
    layout.scoop.state_root.join("control-plane.sqlite3")
}

/// Backend-wide control-plane database handle.
///
/// Construct via [`ControlPlaneDb::open`], which creates (or opens) the
/// SQLite file and runs schema migrations. The resulting value can be
/// shared by cloning; each call opens a fresh SQLite connection under the hood
/// and keeps the blocking bridge inside this module.
#[derive(Clone)]
pub struct ControlPlaneDb {
    location: ControlPlaneLocation,
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
        let location = ControlPlaneLocation::File(db_path.clone());

        initialize_database(location.clone()).await?;

        tracing::info!(db = %db_path.display(), "control-plane DB initialized");

        Ok(Self { location })
    }

    pub async fn open_for_layout(layout: &RuntimeLayout) -> Result<Self> {
        Self::open(&db_path_for_layout(layout)).await
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
        let path = std::env::temp_dir().join(unique_name);
        let location = ControlPlaneLocation::EphemeralFile(path);
        initialize_database(location.clone()).await?;
        Ok(Self { location })
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
        let location = self.location.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = open_connection(&location)?;
            f(&mut conn).map_err(|e| crate::BackendError::external("control-plane DB call failed", e))
        })
        .await
        .map_err(|e| crate::BackendError::external("control-plane DB task join failed", e))?
    }

    /// Execute a read-write closure on the underlying SQLite connection.
    ///
    /// Alias for [`Self::call`] -- the distinction is purely documentary
    /// at this layer. Transactions should be managed by the caller inside
    /// the closure.
    pub async fn call_write<F, T>(&self, f: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> std::result::Result<T, rusqlite::Error>
            + Send
            + 'static,
    {
        self.call(f).await
    }
}

fn configure_connection(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    Ok(())
}

fn open_connection(location: &ControlPlaneLocation) -> Result<Connection> {
    let conn = Connection::open(location.path())
        .map_err(|e| crate::BackendError::external("failed to open control-plane DB", e))?;
    configure_connection(&conn)
        .map_err(|e| crate::BackendError::external("failed to configure control-plane DB", e))?;
    Ok(conn)
}

async fn initialize_database(location: ControlPlaneLocation) -> Result<()> {
    let db_path = location.path().to_path_buf();
    tokio::task::spawn_blocking(move || {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::BackendError::fs("create", parent, e))?;
        }
        let conn = open_connection(&location)?;
        run_migrations(&conn)
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

        // Verify the schema_metadata table was seeded.
        let version: i64 = db
            .call(|conn| {
                Ok(conn
                    .query_row(
                        "SELECT MAX(version) FROM schema_metadata",
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
