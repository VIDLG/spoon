//! SQLite control-plane database facade.
//!
//! Owns the `tokio_rusqlite::Connection` and exposes async methods for
//! initialization. All driver-level details (raw SQL, `Connection::call`)
//! stay inside this module and its sibling [`migrations`] module; business-rule
//! code in lifecycle, store, or projection modules must never import
//! `tokio_rusqlite` or `rusqlite` directly.

use crate::Result;

use super::migrations::run_migrations;

/// Backend-wide control-plane database handle.
///
/// Construct via [`ControlPlaneDb::open`], which creates (or opens) the
/// SQLite file and runs schema migrations. The resulting value can be
/// shared by cloning (the inner `Connection` is reference-counted).
#[derive(Clone)]
pub struct ControlPlaneDb {
    conn: tokio_rusqlite::Connection,
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

        let conn = tokio_rusqlite::Connection::open(db_path)
            .await
            .map_err(|e| crate::BackendError::external("failed to open control-plane DB", e))?;

        configure_connection(&conn, "failed to configure control-plane DB").await?;

        run_migrations(&conn).await?;

        tracing::info!(db = %db_path.display(), "control-plane DB initialized");

        Ok(Self { conn })
    }

    /// Open an in-memory control-plane database. Useful for testing.
    pub async fn open_in_memory() -> Result<Self> {
        let conn = tokio_rusqlite::Connection::open_in_memory()
            .await
            .map_err(|e| {
                crate::BackendError::external("failed to open in-memory control-plane DB", e)
            })?;

        configure_connection(&conn, "failed to configure in-memory control-plane DB").await?;

        run_migrations(&conn).await?;

        Ok(Self { conn })
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
        F: FnOnce(&mut rusqlite::Connection) -> std::result::Result<T, rusqlite::Error>
            + Send
            + 'static,
    {
        self.conn
            .call(|conn| f(conn).map_err(tokio_rusqlite::Error::from))
            .await
            .map_err(|e| crate::BackendError::external("control-plane DB call failed", e))
    }

    /// Execute a read-write closure on the underlying SQLite connection.
    ///
    /// Alias for [`Self::call`] -- the distinction is purely documentary
    /// at this layer. Transactions should be managed by the caller inside
    /// the closure.
    pub async fn call_write<F, T>(&self, f: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut rusqlite::Connection) -> std::result::Result<T, rusqlite::Error>
            + Send
            + 'static,
    {
        self.call(f).await
    }
}

/// Set connection-level pragmas.
async fn configure_connection(
    conn: &tokio_rusqlite::Connection,
    error_context: &str,
) -> Result<()> {
    conn.call(|conn| {
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(())
    })
    .await
    .map_err(|e| crate::BackendError::external(error_context, e))
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

        assert_eq!(version, 1, "migration 0001 should be applied");
    }
}
