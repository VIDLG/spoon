//! Focused bootstrap tests for the SQLite control-plane initialization.

use crate::control_plane::{
    acquire_lock, begin_operation, complete_operation, list_doctor_issues, release_lock,
    set_operation_stage,
};
use crate::db::Db;
use crate::event::LifecycleStage;
use crate::layout::RuntimeLayout;
use crate::scoop::state::{
    InstalledPackageCommandSurface, InstalledPackageIdentity, InstalledPackageState,
    write_installed_state,
};
use crate::scoop::{NoopPorts, doctor_with_host};
use crate::tests::temp_dir;

/// Verify that a seeded installed-package row can be inserted and read back
/// through the control-plane DB facade.
#[tokio::test]
async fn sqlite_control_plane_roundtrips_installed_state() {
    let db = Db::open_in_memory()
        .await
        .expect("in-memory DB should open");

    write_installed_state(
        &db,
        &InstalledPackageState {
            identity: InstalledPackageIdentity {
                package: "ripgrep".to_string(),
                version: "14.1.0".to_string(),
                bucket: "main".to_string(),
                architecture: Some("x64".to_string()),
                cache_size_bytes: None,
            },
            command_surface: InstalledPackageCommandSurface {
                bins: vec!["rg.exe".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .await
    .expect("write should succeed");

    let package: (String, String, String) = db
        .call(|conn| {
            Ok(conn.query_row(
                "SELECT package, version, bucket FROM installed_packages WHERE package = ?1",
                rusqlite::params!["ripgrep"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?)
        })
        .await
        .expect("query should succeed");

    assert_eq!(package.0, "ripgrep");
    assert_eq!(package.1, "14.1.0");
    assert_eq!(package.2, "main");

    let bins: String = db
        .call(|conn| {
            Ok(conn.query_row(
                "SELECT bins FROM installed_package_command_surface WHERE package = ?1",
                rusqlite::params!["ripgrep"],
                |row| row.get(0),
            )?)
        })
        .await
        .expect("surface query should succeed");
    assert_eq!(bins, "[\"rg.exe\"]");
}

/// Verify that the schema can record at least one operation journal row.
#[tokio::test]
async fn sqlite_control_plane_records_operation_journal() {
    let db = Db::open_in_memory()
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
    let db = Db::open(&layout.scoop.db_path())
        .await
        .expect("open db");

    let op_id = begin_operation(&db, "install", Some("jq"), Some("main"))
        .await
        .expect("begin operation");
    set_operation_stage(&db, op_id, LifecycleStage::Materializing)
        .await
        .expect("set stage");
    complete_operation(&db, op_id, "completed", Some("done"))
        .await
        .expect("complete operation");

    let acquired = acquire_lock(&db, "scoop:install:jq", "install")
        .await
        .expect("acquire lock");
    assert!(acquired);
    release_lock(&db, "scoop:install:jq")
        .await
        .expect("release lock");

    let issues = list_doctor_issues(&db).await.expect("list issues");
    assert!(issues.is_empty(), "fresh control plane should have no doctor issues");
}

#[tokio::test]
async fn lock_conflict_and_journal_stop_points_are_diagnosable() {
    let root = temp_dir("control-plane-stop-points");
    std::fs::create_dir_all(&root).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&root);
    let db = Db::open(&layout.scoop.db_path())
        .await
        .expect("open db");

    let first = acquire_lock(&db, "scoop:install:demo", "install")
        .await
        .expect("first lock acquisition");
    let second = acquire_lock(&db, "scoop:install:demo", "install")
        .await
        .expect("second lock acquisition");
    assert!(first);
    assert!(!second, "second acquisition should surface lock conflict");

    let op_id = begin_operation(&db, "install", Some("demo"), Some("main"))
        .await
        .expect("begin operation");
    set_operation_stage(&db, op_id, LifecycleStage::PostInstallHooks)
        .await
        .expect("set stage");
    complete_operation(&db, op_id, "failed", Some("hook failed"))
        .await
        .expect("complete failed operation");

    let db = Db::open(&layout.scoop.db_path())
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

    release_lock(&db, "scoop:install:demo")
        .await
        .expect("release lock");
}

#[tokio::test]
async fn doctor_reports_failed_lifecycle_residue() {
    let root = temp_dir("doctor-failed-lifecycle");
    std::fs::create_dir_all(root.join("scoop").join("buckets").join("main"))
        .expect("create main bucket dir");
    let layout = RuntimeLayout::from_root(&root);
    let db = Db::open(&layout.scoop.db_path())
        .await
        .expect("open db");

    let op_id = begin_operation(&db, "update", Some("demo"), Some("main"))
        .await
        .expect("begin operation");
    set_operation_stage(&db, op_id, LifecycleStage::Integrating)
        .await
        .expect("set stage");
    complete_operation(&db, op_id, "failed", Some("integration failed"))
        .await
        .expect("complete failed operation");

    let host = NoopPorts;
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
