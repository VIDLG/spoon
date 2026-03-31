use spoon_backend::msvc::{
    ToolchainTarget, msvc_root, msvc_state_root, msvc_toolchain_root, status,
    write_installed_toolchain_target,
};
use spoon_backend::scoop::shims_root;

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}

#[test]
fn msvc_status_reports_managed_install_with_runtime_state() {
    let root = std::env::temp_dir().join(format!(
        "spoon-backend-msvc-status-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let state_root = msvc_state_root(&root);
    std::fs::create_dir_all(&state_root).unwrap();
    std::fs::write(
        spoon_backend::msvc::runtime_state_path(&root),
        serde_json::json!({
            "toolchain_root": msvc_toolchain_root(&root),
            "wrappers_root": shims_root(&root),
            "runtime": "managed"
        })
        .to_string(),
    )
    .unwrap();
    write_installed_toolchain_target(
        &msvc_root(&root),
        &ToolchainTarget {
            msvc: "msvc-14.44.35207".to_string(),
            sdk: "sdk-10.0.26100.1".to_string(),
        },
    )
    .unwrap();
    std::fs::create_dir_all(shims_root(&root)).unwrap();
    std::fs::write(shims_root(&root).join("spoon-cl.cmd"), "@echo off").unwrap();

    let value = serde_json::to_value(block_on(status(&root))).unwrap();
    assert_eq!(value["kind"], "msvc_status");
    assert_eq!(
        value["managed"]["installed_version"],
        "14.44.35207 + 10.0.26100.1"
    );
    assert_eq!(value["managed"]["runtime_state_present"], true);
    assert!(
        value["managed"]["integration"]["commands"]["wrappers"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    );

    let _ = std::fs::remove_dir_all(root);
}
