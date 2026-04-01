use std::fs;
use std::sync::{Mutex, OnceLock};

use super::{OfficialInstalledState, installed_state_path, official_instance_root, probe, runtime_state_path};
use crate::msvc::official::{OfficialInstallerMode, install_toolchain_async_with_mode};
use crate::layout::RuntimeLayout;
use crate::tests::temp_dir;

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[test]
fn probe_reports_unavailable_when_runtime_state_is_missing() {
    let tool_root = temp_dir("official-probe-missing");
    let (root, available, installed) = probe(&tool_root);
    assert_eq!(root, official_instance_root(&tool_root));
    assert!(!available);
    assert!(installed.is_none());
}

#[test]
fn probe_reads_installed_state_label_when_present() {
    let tool_root = temp_dir("official-probe-state");
    fs::create_dir_all(runtime_state_path(&tool_root).parent().unwrap()).unwrap();
    fs::write(runtime_state_path(&tool_root), "{}").unwrap();
    fs::write(
        installed_state_path(&tool_root),
        serde_json::to_vec(&OfficialInstalledState {
            version: Some("14.44.35207".to_string()),
            sdk_version: Some("10.0.26100.0".to_string()),
        })
        .unwrap(),
    )
    .unwrap();

    let (_, available, installed) = probe(&tool_root);
    assert!(available);
    assert_eq!(installed.as_deref(), Some("14.44.35207 + 10.0.26100.0"));

    let _ = fs::remove_dir_all(&tool_root);
}

#[test]
fn official_install_failure_does_not_commit_canonical_state() {
    let _lock = env_lock();
    let tool_root = temp_dir("official-bootstrapper-failure");
    fs::create_dir_all(&tool_root).unwrap();
    let bootstrapper = tool_root.join("fake-vs-buildtools-fail.cmd");
    fs::write(
        &bootstrapper,
        "@echo off\r\nexit /b 1\r\n",
    )
    .unwrap();
    unsafe {
        std::env::set_var(
            "SPOON_TEST_MSVCOFFICIAL_BOOTSTRAPPER_SOURCE",
            bootstrapper.display().to_string(),
        );
    }

    let err = crate::tests::block_on(install_toolchain_async_with_mode(
        &tool_root,
        OfficialInstallerMode::Quiet,
    ))
    .expect_err("official install should fail");
    assert!(
        err.to_string().contains("official MSVC bootstrapper failed"),
        "{err:#}"
    );

    let layout = RuntimeLayout::from_root(&tool_root);
    let canonical = crate::tests::block_on(crate::msvc::read_canonical_state(&layout));
    assert!(
        canonical.is_none(),
        "failed official install must not commit canonical state"
    );

    unsafe {
        std::env::remove_var("SPOON_TEST_MSVCOFFICIAL_BOOTSTRAPPER_SOURCE");
    }
}
