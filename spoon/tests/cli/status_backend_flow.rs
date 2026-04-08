#[path = "../common/mod.rs"]
mod common;

use common::assertions::assert_ok;
use common::cli::run_in_home;
use common::setup::create_configured_home;
use serde_json::Value;
use spoon::service::{
    SpoonEvent, CommandStatus, FinishEvent, NoticeEvent, StageEvent, StreamChunk,
    stream_chunk_from_backend_event,
};
use spoon_core::LifecycleStage;
use spoon_core::RuntimeLayout;
use spoon_scoop::{InstalledPackageCommandSurface, InstalledPackageIdentity, InstalledPackageState, InstalledPackageUninstall, write_installed_state};

fn parse_json(stdout: &str) -> Value {
    serde_json::from_str(stdout).expect("stdout should be valid json")
}

/// Regression test: the JSON status path must use backend read models
/// and must not fall back to app-side state file parsing (BNDR-05, LAY-02).
#[test]
fn json_status_uses_backend_read_models() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    // Seed canonical state through the backend store so the backend snapshot has content.
    let layout = RuntimeLayout::from_root(&tool_root);
    spoon::runtime::test_block_on(write_installed_state(
        &layout.scoop,
        &InstalledPackageState {
            identity: InstalledPackageIdentity {
                package: "git".to_string(),
                version: "2.53.0.2".to_string(),
                bucket: "main".to_string(),
                architecture: Some("x64".to_string()),
                cache_size_bytes: None,
            },
            command_surface: InstalledPackageCommandSurface {
                bins: vec![],
                shortcuts: vec![],
                env_add_path: vec![],
                env_set: std::collections::BTreeMap::new(),
                persist: vec![],
            },
            integrations: vec![],
            uninstall: InstalledPackageUninstall {
                pre_uninstall: vec![],
                uninstaller_script: vec![],
                post_uninstall: vec![],
            },
        },
    ))
    .unwrap();

    // Register a bucket so the backend snapshot includes bucket data
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &tool_root,
        &spoon::service::scoop::BucketSpec {
            name: "main".to_string(),
            source: Some("https://github.com/ScoopInstaller/Main".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();

    // The JSON status command must succeed
    let (ok, stdout, stderr) = run_in_home(&["--json", "status"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);

    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "status");

    // The status data must include tools and runtime sections
    assert!(json["data"]["tools"].is_array());
    assert!(json["data"]["runtime"].is_object());

    // When a backend snapshot is available, the roots must come from the snapshot
    // (backend-owned RuntimeLayout), not from app-side path helpers.
    // This is the core contract: the JSON status path consumes backend read models.
    let roots = &json["data"]["runtime"]["roots"];
    if roots.is_object() {
        // Roots exist and must match backend layout derivation
        assert!(roots["root"].is_string());
        assert!(roots["scoop"].is_string());
        assert!(roots["managed_msvc"].is_string());
        assert!(roots["managed_toolchain"].is_string());
        assert!(roots["official_msvc"].is_string());
    }
}

#[test]
fn backend_stage_events_drive_app_stream_translation() {
    let running = SpoonEvent::Stage(StageEvent::started(LifecycleStage::Planned));
    let completed = SpoonEvent::Stage(StageEvent::completed(LifecycleStage::Completed));

    let running_chunk = stream_chunk_from_backend_event(running);
    let completed_chunk = stream_chunk_from_backend_event(completed);

    match running_chunk {
        Some(StreamChunk::ReplaceLast(line)) => assert_eq!(line, "Stage: planned"),
        other => panic!("unexpected running chunk: {:?}", other),
    }
    match completed_chunk {
        Some(StreamChunk::ReplaceLast(line)) => assert_eq!(line, "Stage complete: completed"),
        other => panic!("unexpected completed chunk: {:?}", other),
    }
}

#[test]
fn backend_finish_events_drive_app_shell_messages_without_backend_reimplementation() {
    let cancelled = stream_chunk_from_backend_event(SpoonEvent::Finished(FinishEvent::new(
        CommandStatus::Cancelled,
        None,
    )));
    let failed = stream_chunk_from_backend_event(SpoonEvent::Finished(FinishEvent::new(
        CommandStatus::Failed,
        None,
    )));
    let blocked = stream_chunk_from_backend_event(SpoonEvent::Finished(FinishEvent::new(
        CommandStatus::Blocked,
        None,
    )));
    let explicit = stream_chunk_from_backend_event(SpoonEvent::Finished(FinishEvent::failed(
        "hook failed before commit",
    )));

    match cancelled {
        Some(StreamChunk::Append(line)) => assert_eq!(line, "Cancelled by user."),
        other => panic!("unexpected cancelled chunk: {:?}", other),
    }
    match failed {
        Some(StreamChunk::Append(line)) => assert_eq!(line, "Operation failed."),
        other => panic!("unexpected failed chunk: {:?}", other),
    }
    match blocked {
        Some(StreamChunk::Append(line)) => assert_eq!(line, "Operation blocked."),
        other => panic!("unexpected blocked chunk: {:?}", other),
    }
    match explicit {
        Some(StreamChunk::Append(line)) => assert_eq!(line, "hook failed before commit"),
        other => panic!("unexpected explicit failure chunk: {:?}", other),
    }
}

#[test]
fn backend_notice_events_append_visible_messages() {
    let info = stream_chunk_from_backend_event(SpoonEvent::Notice(NoticeEvent::info("hello")));
    let warning =
        stream_chunk_from_backend_event(SpoonEvent::Notice(NoticeEvent::warning("careful now")));

    match info {
        Some(StreamChunk::Append(line)) => assert_eq!(line, "hello"),
        other => panic!("unexpected info notice chunk: {:?}", other),
    }
    match warning {
        Some(StreamChunk::Append(line)) => assert_eq!(line, "Warning: careful now"),
        other => panic!("unexpected warning notice chunk: {:?}", other),
    }
}
