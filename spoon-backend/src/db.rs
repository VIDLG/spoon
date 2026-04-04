//! Backend database facade.
//!
//! Owns the repo-level tokio boundary over `rusqlite` and exposes async methods
//! for initialization. All driver-level details (raw SQL, `rusqlite::Connection`,
//! and `spawn_blocking`) stay inside this module; business-rule code in lifecycle, store, or projection
//! modules must never import them directly.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

use crate::Result;

include!(concat!(env!("OUT_DIR"), "/control_plane_migrations.rs"));

/// Backend-wide database handle.
///
/// Construct via [`Db::open`], which creates (or opens) the
/// SQLite file and runs all pending schema migrations. The resulting value can
/// be shared by cloning; each call opens a fresh SQLite connection under the
/// hood and keeps the blocking bridge inside this module.
#[derive(Clone)]
pub struct Db(PathBuf);

impl Db {
    /// Open (or create) the backend database at the given path and
    /// run all pending schema migrations.
    pub async fn open(db_path: &Path) -> Result<Self> {
        let db_path = db_path.to_path_buf();
        initialize_database(db_path.clone()).await?;

        tracing::info!(db = %db_path.display(), "control-plane DB initialized");

        Ok(Self(db_path))
    }

    /// Open an ephemeral backend database. Useful for testing.
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
        Ok(Self(db_path))
    }

    /// Execute a closure on the underlying SQLite connection.
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> std::result::Result<T, rusqlite::Error> + Send + 'static,
    {
        let db_path = self.0.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = open_connection(&db_path)?;
            f(&mut conn).map_err(|e| crate::BackendError::control_plane("db call", e))
        })
        .await
        .map_err(|e| crate::BackendError::control_plane("db task join", e))?
    }
}

fn open_connection(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)
        .map_err(|e| crate::BackendError::control_plane("db open", e))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(|e| crate::BackendError::control_plane("db configure", e))?;
    Ok(conn)
}

async fn initialize_database(db_path: PathBuf) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::BackendError::fs("create", parent, e))?;
        }
        let mut conn = open_connection(&db_path)?;
        generated_migrations()
            .to_latest(&mut conn)
            .map_err(|e| crate::BackendError::control_plane("migration", e))
    })
    .await
    .map_err(|e| crate::BackendError::control_plane("db init task join", e))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_db_opens_and_migrates() {
        let db = Db::open_in_memory()
            .await
            .expect("in-memory DB should open");

        let version: i64 = db
            .call(|conn| {
                Ok(conn
                    .query_row("PRAGMA user_version", [], |row| row.get(0))
                    .unwrap_or(0))
            })
            .await
            .expect("schema query should succeed");

        assert_eq!(version, 3, "migrations 0001, 0002, and 0003 should be applied");
    }
}
