use crate::Result;
use crate::control_plane::sqlite::ControlPlaneDb;
use crate::layout::RuntimeLayout;
use rusqlite::{Error as SqlError, params};

pub async fn acquire_lock(
    layout: &RuntimeLayout,
    lock_key: &str,
    operation_type: &str,
) -> Result<bool> {
    let db = ControlPlaneDb::open_for_layout(layout).await?;
    let lock_key = lock_key.to_string();
    let operation_type = operation_type.to_string();
    db.call_write(move |conn| {
        match conn.execute(
            "INSERT INTO operation_locks (lock_key, operation_type)
             VALUES (?1, ?2)",
            params![lock_key, operation_type],
        ) {
            Ok(_) => Ok(true),
            Err(SqlError::SqliteFailure(err, _))
                if err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE =>
            {
                Ok(false)
            }
            Err(err) => Err(err),
        }
    })
    .await
}

pub async fn release_lock(layout: &RuntimeLayout, lock_key: &str) -> Result<()> {
    let db = ControlPlaneDb::open_for_layout(layout).await?;
    let lock_key = lock_key.to_string();
    db.call_write(move |conn| {
        conn.execute(
            "DELETE FROM operation_locks WHERE lock_key = ?1",
            params![lock_key],
        )?;
        Ok(())
    })
    .await
}
