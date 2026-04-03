use crate::Result;
use crate::control_plane::sqlite::ControlPlaneDb;
use crate::event::LifecycleStage;
use crate::layout::RuntimeLayout;
use rusqlite::params;

pub async fn begin_operation(
    layout: &RuntimeLayout,
    operation_type: &str,
    package: Option<&str>,
    bucket: Option<&str>,
) -> Result<i64> {
    let db = ControlPlaneDb::open(&layout.scoop.control_plane_db_path()).await?;
    let operation_type = operation_type.to_string();
    let package = package.map(ToString::to_string);
    let bucket = bucket.map(ToString::to_string);
    db.call(move |conn| {
        conn.execute(
            "INSERT INTO operation_journal (operation_type, package, bucket, status)
             VALUES (?1, ?2, ?3, 'running')",
            params![operation_type, package, bucket],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .await
}

pub async fn set_operation_stage(
    layout: &RuntimeLayout,
    operation_id: i64,
    stage: LifecycleStage,
) -> Result<()> {
    let db = ControlPlaneDb::open(&layout.scoop.control_plane_db_path()).await?;
    let stage = stage.as_str().to_string();
    db.call(move |conn| {
        conn.execute(
            "UPDATE operation_journal
             SET details = ?2
             WHERE id = ?1",
            params![operation_id, stage],
        )?;
        Ok(())
    })
    .await
}

pub async fn complete_operation(
    layout: &RuntimeLayout,
    operation_id: i64,
    status: &str,
    details: Option<&str>,
) -> Result<()> {
    let db = ControlPlaneDb::open(&layout.scoop.control_plane_db_path()).await?;
    let status = status.to_string();
    let details = details.map(ToString::to_string);
    db.call(move |conn| {
        conn.execute(
            "UPDATE operation_journal
             SET status = ?2,
                 finished_at = datetime('now'),
                 details = CASE
                    WHEN ?3 IS NULL THEN details
                    WHEN details IS NULL OR details = '' THEN ?3
                    ELSE details || char(10) || ?3
                 END
             WHERE id = ?1",
            params![operation_id, status, details],
        )?;
        Ok(())
    })
    .await
}
