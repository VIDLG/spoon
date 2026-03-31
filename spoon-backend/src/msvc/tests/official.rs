use std::fs;

use super::{
    OfficialInstalledState, installed_state_path, official_instance_root, probe, runtime_state_path,
};
use crate::tests::temp_dir;

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
