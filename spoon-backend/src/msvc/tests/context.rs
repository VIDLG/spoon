use std::fs;

use crate::BackendContext;
use crate::layout::RuntimeLayout;
use crate::tests::temp_dir;

fn test_context(tool_root: &std::path::Path) -> BackendContext<()> {
    BackendContext::new(tool_root.to_path_buf(), None, true, "auto", "default", ())
}

#[test]
fn msvc_context_drives_status_and_install() {
    let tool_root = temp_dir("msvc-context-status");
    let context = test_context(&tool_root);
    let manifest_root = crate::msvc::paths::msvc_manifest_root(&tool_root).join("vs");
    let payload_root = temp_dir("msvc-context-payloads");
    let sdk_payload = payload_root.join("sdk-tools.msi");
    fs::create_dir_all(&manifest_root).expect("manifest root should be created");
    fs::create_dir_all(&payload_root).expect("payload root should be created");
    fs::write(&sdk_payload, b"fake sdk payload bytes").expect("payload should be written");
    let sdk_url = format!(
        "file:///{}",
        sdk_payload.display().to_string().replace('\\', "/")
    );
    let sdk_sha = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"fake sdk payload bytes");
        format!("{:x}", hasher.finalize())
    };
    fs::write(
        manifest_root.join("latest.json"),
        serde_json::json!({
            "packages": [
                {
                    "id": "Microsoft.VC.14.44.35207.Tools.HostX64.TargetX64.base",
                    "version": "14.44.35207",
                    "language": "neutral",
                    "payloads": []
                },
                {
                    "id": "WindowsSdkPackageB",
                    "version": "10.0.26100.1",
                    "language": "en-US",
                    "payloads": [
                        {
                            "url": sdk_url,
                            "fileName": "Installers\\Windows SDK for Windows Store Apps Tools-x86_en-us.msi",
                            "sha256": sdk_sha
                        }
                    ]
                }
            ]
        })
        .to_string(),
    )
    .expect("manifest should be written");

    let before = crate::tests::block_on(crate::msvc::status_with_context(&context));
    assert_eq!(before.managed.status, "not installed");

    let install =
        crate::tests::block_on(crate::msvc::install_toolchain_async_with_context(&context))
            .expect("managed install should use explicit context");
    assert!(install.is_success(), "{:?}", install.output);
    assert!(
        install
            .output
            .iter()
            .any(|line| line.contains("Installed latest MSVC toolchain target directly with spoon")),
        "{:?}",
        install.output
    );

    let after = crate::tests::block_on(crate::msvc::status_with_context(&context));
    assert!(after.managed.status.contains("installed"));
    assert_eq!(
        after.authoritative_runtime,
        Some(crate::msvc::MsvcRuntimeKind::Managed)
    );
}

#[test]
fn explicit_context_required_for_runtime_ops() {
    let tool_root = temp_dir("msvc-explicit-context");
    let context = test_context(&tool_root);
    assert_eq!(context.root, tool_root);
    assert!(context.test_mode);
    assert_eq!(context.msvc_command_profile, "default");
    assert_eq!(
        context.layout.msvc.managed.root,
        crate::msvc::paths::msvc_root(&tool_root)
    );
}

#[test]
fn detect_runtimes_with_context_reports_managed_and_official_facts() {
    let tool_root = temp_dir("msvc-detect-context");
    let context = test_context(&tool_root);

    let managed_state_root = crate::msvc::paths::msvc_state_root(&tool_root);
    let official_state_root = crate::msvc::paths::official_msvc_state_root(&tool_root);
    fs::create_dir_all(&managed_state_root).expect("managed state root");
    fs::create_dir_all(&official_state_root).expect("official state root");

    fs::write(
        crate::msvc::runtime_state_path(&tool_root),
        serde_json::json!({
            "runtime": "managed"
        })
        .to_string(),
    )
    .expect("managed runtime state");
    fs::write(
        managed_state_root.join("installed.json"),
        serde_json::json!({
            "msvc": "msvc-14.44.35207",
            "sdk": "sdk-10.0.26100.15"
        })
        .to_string(),
    )
    .expect("managed installed state");

    fs::write(
        crate::msvc::official::runtime_state_path(&tool_root),
        serde_json::json!({
            "runtime": "official"
        })
        .to_string(),
    )
    .expect("official runtime state");
    fs::write(
        crate::msvc::official::installed_state_path(&tool_root),
        serde_json::json!({
            "version": "14.44.35207",
            "sdk_version": "10.0.26100.0"
        })
        .to_string(),
    )
    .expect("official installed state");

    let detected = crate::msvc::detect_runtimes_with_context(&context);
    assert!(detected.managed.available);
    assert!(detected.official.available);
    assert_eq!(
        detected.managed.installed_version.as_deref(),
        Some("14.44.35207 + 10.0.26100.15")
    );
    assert_eq!(
        detected.official.installed_version.as_deref(),
        Some("14.44.35207 + 10.0.26100.0")
    );
}

#[test]
fn msvc_operation_request_and_stage_contract_are_stable() {
    let request =
        crate::msvc::MsvcOperationRequest::install(crate::msvc::MsvcRuntimePreference::Managed);
    assert_eq!(request.operation, crate::msvc::MsvcOperationKind::Install);
    assert_eq!(
        request.runtime_preference,
        crate::msvc::MsvcRuntimePreference::Managed
    );
    assert_eq!(crate::msvc::MsvcRuntimePreference::Auto.as_str(), "auto");
    assert_eq!(
        crate::msvc::MsvcRuntimePreference::Official.as_str(),
        "official"
    );
    assert_eq!(
        crate::msvc::MsvcLifecycleStage::Detecting.as_str(),
        "detecting"
    );
    assert_eq!(
        crate::msvc::MsvcLifecycleStage::StateCommitting.as_str(),
        "state_committing"
    );
}

#[test]
fn canonical_msvc_state_roundtrips_via_sqlite_control_plane() {
    let tool_root = temp_dir("msvc-canonical-state");
    let layout = RuntimeLayout::from_root(&tool_root);
    let state = crate::msvc::MsvcCanonicalState {
        runtime_kind: crate::msvc::MsvcRuntimeKind::Managed,
        installed: true,
        version: Some("14.44.35207".to_string()),
        sdk_version: Some("10.0.26100.15".to_string()),
        last_operation: Some(crate::msvc::MsvcOperationKind::Install),
        last_stage: Some(crate::msvc::MsvcLifecycleStage::Completed),
        validation_status: Some(crate::msvc::MsvcValidationStatus::Valid),
        validation_message: Some("validated successfully".to_string()),
        managed: crate::msvc::ManagedMsvcStateDetail {
            selected_target_arch: Some("x64".to_string()),
        },
        official: crate::msvc::OfficialMsvcStateDetail {
            installer_mode: None,
        },
    };

    crate::tests::block_on(crate::msvc::write_canonical_state(&layout, &state))
        .expect("write canonical msvc state");

    let stored = crate::tests::block_on(crate::msvc::read_canonical_state(&layout))
        .expect("stored canonical msvc state");
    assert_eq!(stored, state);

    crate::tests::block_on(crate::msvc::clear_canonical_state(&layout))
        .expect("clear canonical msvc state");
    assert!(
        crate::tests::block_on(crate::msvc::read_canonical_state(&layout)).is_none(),
        "canonical msvc state should be removable"
    );
}

#[test]
fn status_prefers_canonical_runtime_summary_when_present() {
    let tool_root = temp_dir("msvc-status-canonical");
    let context = test_context(&tool_root);
    let managed_state_root = crate::msvc::paths::msvc_state_root(&tool_root);
    fs::create_dir_all(&managed_state_root).expect("managed state root");
    fs::write(
        managed_state_root.join("installed.json"),
        serde_json::json!({
            "msvc": "msvc-14.44.35207",
            "sdk": "sdk-10.0.26100.15"
        })
        .to_string(),
    )
    .expect("legacy installed state");

    let layout = RuntimeLayout::from_root(&tool_root);
    crate::tests::block_on(crate::msvc::write_canonical_state(
        &layout,
        &crate::msvc::MsvcCanonicalState {
            runtime_kind: crate::msvc::MsvcRuntimeKind::Managed,
            installed: true,
            version: Some("14.50.00000".to_string()),
            sdk_version: Some("10.0.99999.1".to_string()),
            last_operation: Some(crate::msvc::MsvcOperationKind::Update),
            last_stage: Some(crate::msvc::MsvcLifecycleStage::Completed),
            validation_status: Some(crate::msvc::MsvcValidationStatus::Valid),
            validation_message: Some("validated successfully".to_string()),
            managed: crate::msvc::ManagedMsvcStateDetail {
                selected_target_arch: Some("x64".to_string()),
            },
            official: crate::msvc::OfficialMsvcStateDetail::default(),
        },
    ))
    .expect("write canonical state");

    let status = crate::tests::block_on(crate::msvc::status_with_context(&context));
    assert_eq!(
        status.authoritative_runtime,
        Some(crate::msvc::MsvcRuntimeKind::Managed)
    );
    assert_eq!(
        status.managed.installed_version.as_deref(),
        Some("14.50.00000 + 10.0.99999.1")
    );
    assert_eq!(
        status.validation_status,
        Some(crate::msvc::MsvcValidationStatus::Valid)
    );
}

#[test]
fn doctor_reports_canonical_runtime_drift() {
    let tool_root = temp_dir("msvc-doctor-drift");
    let layout = RuntimeLayout::from_root(&tool_root);
    crate::tests::block_on(crate::msvc::write_canonical_state(
        &layout,
        &crate::msvc::MsvcCanonicalState {
            runtime_kind: crate::msvc::MsvcRuntimeKind::Official,
            installed: true,
            version: Some("14.44.35207".to_string()),
            sdk_version: Some("10.0.26100.0".to_string()),
            last_operation: Some(crate::msvc::MsvcOperationKind::Install),
            last_stage: Some(crate::msvc::MsvcLifecycleStage::Completed),
            validation_status: Some(crate::msvc::MsvcValidationStatus::Valid),
            validation_message: None,
            managed: crate::msvc::ManagedMsvcStateDetail::default(),
            official: crate::msvc::OfficialMsvcStateDetail {
                installer_mode: Some("quiet".to_string()),
            },
        },
    ))
    .expect("write canonical state");

    let report = crate::tests::block_on(crate::msvc::doctor(&tool_root));
    assert!(!report.healthy);
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.category == "canonical_runtime_drift"),
        "{report:?}"
    );
}
