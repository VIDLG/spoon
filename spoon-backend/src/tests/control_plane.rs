//! Focused bootstrap tests for the SQLite control-plane initialization.

use crate::control_plane::{
    ControlPlaneDb, acquire_lock, begin_operation, complete_operation, list_doctor_issues,
    release_lock, set_operation_stage,
};
use crate::event::LifecycleStage;
use crate::layout::RuntimeLayout;
use crate::scoop::{NoopScoopRuntimeHost, doctor_with_host};
use crate::tests::temp_dir;

/// Verify that a seeded installed-package row can be inserted and read back
/// through the control-plane DB facade.
#[tokio::test]
async fn sqlite_control_plane_roundtrips_installed_state() {
    let db = ControlPlaneDb::open_in_memory()
        .await
        .expect("in-memory DB should open");

    // Seed a row.
    db.call(|conn| {
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
    db.call(|conn| {
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

#[tokio::test]
async fn sqlite_store_facade_hides_driver_details() {
    let root = temp_dir("control-plane-facade");
    std::fs::create_dir_all(&root).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&root);

    let op_id = begin_operation(&layout, "install", Some("jq"), Some("main"))
        .await
        .expect("begin operation");
    set_operation_stage(&layout, op_id, LifecycleStage::Materializing)
        .await
        .expect("set stage");
    complete_operation(&layout, op_id, "completed", Some("done"))
        .await
        .expect("complete operation");

    let acquired = acquire_lock(&layout, "scoop:install:jq", "install")
        .await
        .expect("acquire lock");
    assert!(acquired);
    release_lock(&layout, "scoop:install:jq")
        .await
        .expect("release lock");

    let issues = list_doctor_issues(&layout).await.expect("list issues");
    assert!(issues.is_empty(), "fresh control plane should have no doctor issues");
}

#[tokio::test]
async fn lock_conflict_and_journal_stop_points_are_diagnosable() {
    let root = temp_dir("control-plane-stop-points");
    std::fs::create_dir_all(&root).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&root);

    let first = acquire_lock(&layout, "scoop:install:demo", "install")
        .await
        .expect("first lock acquisition");
    let second = acquire_lock(&layout, "scoop:install:demo", "install")
        .await
        .expect("second lock acquisition");
    assert!(first);
    assert!(!second, "second acquisition should surface lock conflict");

    let op_id = begin_operation(&layout, "install", Some("demo"), Some("main"))
        .await
        .expect("begin operation");
    set_operation_stage(&layout, op_id, LifecycleStage::PostInstallHooks)
        .await
        .expect("set stage");
    complete_operation(&layout, op_id, "failed", Some("hook failed"))
        .await
        .expect("complete failed operation");

    let db = ControlPlaneDb::open(&layout.scoop.control_plane_db_path())
        .await
        .expect("open db");
    let (status, details, lock_count): (String, String, i64) = db
        .call(move |conn| {
            let row = conn.query_row(
                "SELECT status, details FROM operation_journal WHERE id = ?1",
                rusqlite::params![op_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )?;
            let lock_count = conn.query_row(
                "SELECT COUNT(*) FROM operation_locks WHERE lock_key = 'scoop:install:demo'",
                [],
                |row| row.get::<_, i64>(0),
            )?;
            Ok((row.0, row.1, lock_count))
        })
        .await
        .expect("query control plane");

    assert_eq!(status, "failed");
    assert!(
        details.contains("post_install_hooks"),
        "last stage should remain diagnosable: {details}"
    );
    assert!(
        details.contains("hook failed"),
        "failure detail should remain diagnosable: {details}"
    );
    assert_eq!(lock_count, 1, "held lock should stay visible in control plane");

    release_lock(&layout, "scoop:install:demo")
        .await
        .expect("release lock");
}

#[tokio::test]
async fn doctor_reports_failed_lifecycle_residue() {
    let root = temp_dir("doctor-failed-lifecycle");
    std::fs::create_dir_all(root.join("scoop").join("buckets").join("main"))
        .expect("create main bucket dir");
    let layout = RuntimeLayout::from_root(&root);

    let op_id = begin_operation(&layout, "update", Some("demo"), Some("main"))
        .await
        .expect("begin operation");
    set_operation_stage(&layout, op_id, LifecycleStage::Integrating)
        .await
        .expect("set stage");
    complete_operation(&layout, op_id, "failed", Some("integration failed"))
        .await
        .expect("complete failed operation");

    let host = NoopScoopRuntimeHost;
    let details = doctor_with_host(&root, "", &host)
        .await
        .expect("doctor details");

    assert!(
        !details.success,
        "failed lifecycle residue should make doctor unsuccessful"
    );
    let failed_issue = details
        .control_plane_issues
        .iter()
        .find(|issue| issue.category == "failed_lifecycle")
        .expect("failed lifecycle issue");
    assert_eq!(failed_issue.severity, "error");
    assert_eq!(failed_issue.package.as_deref(), Some("demo"));
    assert!(
        failed_issue.description.contains("integrating"),
        "stage should be present in doctor issue: {}",
        failed_issue.description
    );
    assert!(
        failed_issue.description.contains("integration failed"),
        "error detail should be present in doctor issue: {}",
        failed_issue.description
    );
}
