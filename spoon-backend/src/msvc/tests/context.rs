use std::fs;

use crate::BackendContext;
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
