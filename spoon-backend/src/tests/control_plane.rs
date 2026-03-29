//! Focused bootstrap tests for the SQLite control-plane initialization.

use crate::control_plane::ControlPlaneDb;

/// Verify that a seeded installed-package row can be inserted and read back
/// through the control-plane DB facade.
#[tokio::test]
async fn sqlite_control_plane_roundtrips_installed_state() {
    let db = ControlPlaneDb::open_in_memory()
        .await
        .expect("in-memory DB should open");

    // Seed a row.
    db.call_write(|conn| {
        conn.execute(
            "INSERT INTO installed_packages
                (package, version, bucket, architecture, bins)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["ripgrep", "14.1.0", "main", "x64", "[\"rg.exe\"]"],
        )?;
        Ok(())
    })
    .await
    .expect("insert should succeed");

    // Read it back.
    let (package, version, bucket): (String, String, String) = db
        .call(|conn| {
            Ok(conn.query_row(
                "SELECT package, version, bucket FROM installed_packages WHERE package = ?1",
                rusqlite::params!["ripgrep"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?)
        })
        .await
        .expect("query should succeed");

    assert_eq!(package, "ripgrep");
    assert_eq!(version, "14.1.0");
    assert_eq!(bucket, "main");
}

/// Verify that the schema can record at least one operation journal row.
#[tokio::test]
async fn sqlite_control_plane_records_operation_journal() {
    let db = ControlPlaneDb::open_in_memory()
        .await
        .expect("in-memory DB should open");

    // Insert a journal entry.
    db.call_write(|conn| {
        conn.execute(
            "INSERT INTO operation_journal (operation_type, package, status)
             VALUES (?1, ?2, ?3)",
            rusqlite::params!["install", "fd", "completed"],
        )?;
        Ok(())
    })
    .await
    .expect("journal insert should succeed");

    // Read it back.
    let (op_type, pkg, status): (String, String, String) = db
        .call(|conn| {
            Ok(conn.query_row(
                "SELECT operation_type, package, status FROM operation_journal WHERE package = ?1",
                rusqlite::params!["fd"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?)
        })
        .await
        .expect("journal query should succeed");

    assert_eq!(op_type, "install");
    assert_eq!(pkg, "fd");
    assert_eq!(status, "completed");
}
