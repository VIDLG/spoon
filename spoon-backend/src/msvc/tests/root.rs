use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use super::super::{
    ToolchainAction, ToolchainFlags, handle_manifest_refresh_failure,
    install_toolchain_async_with_context, install_toolchain_streaming_with_context,
    latest_toolchain_version_label_with_context, managed_toolchain_flags_with_context,
    runtime_state_path, uninstall_toolchain_with_context, update_toolchain_async_with_context,
    user_facing_toolchain_label, validate_toolchain_with_context, write_managed_toolchain_wrappers,
};
use crate::msvc::paths::{msvc_manifest_root, shims_root};
use crate::tests::{block_on, temp_dir};
use crate::{BackendContext, BackendError, BackendEvent};
use sha2::{Digest, Sha256};

mod config {
    use std::path::{Path, PathBuf};

    use crate::msvc::paths::{msvc_cache_root, msvc_state_root, msvc_toolchain_root, shims_root};
    pub fn enable_test_mode() {
        let _ = ();
    }

    pub fn msvc_state_root_from(tool_root: &Path) -> PathBuf {
        msvc_state_root(tool_root)
    }

    pub fn msvc_cache_root_from(tool_root: &Path) -> PathBuf {
        msvc_cache_root(tool_root)
    }

    pub fn msvc_toolchain_root_from(tool_root: &Path) -> PathBuf {
        msvc_toolchain_root(tool_root)
    }

    pub fn shims_root_from(tool_root: &Path) -> PathBuf {
        shims_root(tool_root)
    }
}

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

fn write_small_zip(path: &Path, entry_name: &str, bytes: &[u8]) {
    use std::io::Write;

    let file = fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file(entry_name, options).unwrap();
    zip.write_all(bytes).unwrap();
    zip.finish().unwrap();
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn seed_managed_state(tool_root: &Path, msvc: &str, sdk: &str) {
    let state_root = config::msvc_state_root_from(tool_root);
    fs::create_dir_all(&state_root).unwrap();
    fs::write(
        runtime_state_path(tool_root),
        serde_json::json!({
            "toolchain_root": config::msvc_toolchain_root_from(tool_root),
            "wrappers_root": config::shims_root_from(tool_root),
            "runtime": "managed"
        })
        .to_string(),
    )
    .unwrap();
    fs::write(
        state_root.join("installed.json"),
        serde_json::json!({
            "msvc": msvc,
            "sdk": sdk
        })
        .to_string(),
    )
    .unwrap();
}

fn seed_msvc_policy_command_profile(tool_root: &Path, command_profile: &str) {
    let _ = (tool_root, command_profile);
}

fn test_context(tool_root: &Path) -> BackendContext<()> {
    test_context_with_profile(tool_root, "default")
}

fn test_context_with_profile(tool_root: &Path, command_profile: &str) -> BackendContext<()> {
    BackendContext::new(
        tool_root.to_path_buf(),
        None,
        true,
        "auto",
        command_profile,
        (),
    )
}

async fn install_toolchain_async(
    tool_root: &Path,
) -> crate::Result<crate::msvc::MsvcOperationOutcome> {
    let context = test_context(tool_root);
    install_toolchain_async_with_context(&context).await
}

async fn update_toolchain_async(
    tool_root: &Path,
) -> crate::Result<crate::msvc::MsvcOperationOutcome> {
    let context = test_context(tool_root);
    update_toolchain_async_with_context(&context).await
}

async fn validate_toolchain(tool_root: &Path) -> crate::Result<crate::msvc::MsvcOperationOutcome> {
    let context = test_context(tool_root);
    validate_toolchain_with_context(&context).await
}

async fn managed_toolchain_flags(tool_root: &Path) -> crate::Result<ToolchainFlags> {
    let context = test_context(tool_root);
    managed_toolchain_flags_with_context(&context).await
}

async fn install_toolchain_streaming<F>(
    tool_root: &Path,
    cancel: Option<&crate::CancellationToken>,
    emit: &mut F,
) -> crate::Result<crate::msvc::MsvcOperationOutcome>
where
    F: FnMut(BackendEvent),
{
    let context = test_context(tool_root);
    install_toolchain_streaming_with_context(&context, cancel, emit).await
}

async fn uninstall_toolchain(tool_root: &Path) -> crate::Result<crate::msvc::MsvcOperationOutcome> {
    let context = test_context(tool_root);
    uninstall_toolchain_with_context(&context).await
}

fn latest_toolchain_version_label(tool_root: Option<&Path>) -> Option<String> {
    let root = tool_root?;
    let context = test_context(root);
    latest_toolchain_version_label_with_context(&context)
}

fn read_canonical_state_for_test(
    tool_root: &Path,
) -> Option<crate::msvc::MsvcCanonicalState> {
    let layout = crate::layout::RuntimeLayout::from_root(tool_root);
    block_on(crate::msvc::read_canonical_state(&layout))
}

#[test]
fn user_facing_toolchain_label_strips_internal_prefixes() {
    assert_eq!(
        user_facing_toolchain_label("msvc-14.44.35207 + sdk-10.0.26100.15"),
        "14.44.35207 + 10.0.26100.15"
    );
}

#[test]
fn latest_toolchain_version_prefers_cached_manifest_over_driver_list() {
    let tool_root = temp_dir("manifest-latest");
    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    fs::create_dir_all(&manifest_root).unwrap();
    fs::write(
            manifest_root.join("latest.json"),
            r#"{
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
                      "url": "https://example.invalid/sdk-tools.msi",
                      "fileName": "Installers\\Windows SDK for Windows Store Apps Tools-x86_en-us.msi",
                      "sha256": "abc"
                    }
                  ]
                }
              ]
            }"#,
        )
        .unwrap();
    let latest = latest_toolchain_version_label(Some(&tool_root)).expect("latest");
    assert_eq!(latest, "14.44.35207 + 10.0.26100.1");
}

#[test]
fn install_toolchain_prefers_cached_manifest_target_over_driver_list() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("install-manifest-target");
    let payload_root = temp_dir("install-manifest-payloads");
    fs::create_dir_all(&payload_root).unwrap();
    let sdk_payload = payload_root.join("sdk-tools.msi");
    let sdk_bytes = b"fake sdk payload bytes";
    fs::write(&sdk_payload, sdk_bytes).unwrap();
    let sdk_url = format!(
        "file:///{}",
        sdk_payload.display().to_string().replace('\\', "/")
    );
    let sdk_sha = sha256_hex(sdk_bytes);

    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    fs::create_dir_all(&manifest_root).unwrap();
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
        .unwrap();

    let result = block_on(install_toolchain_async(&tool_root)).expect("install toolchain");
    assert!(result.is_success(), "{:?}", result.output);
    assert!(
            result
                .output
                .iter()
                .any(|line| line.contains("Installed latest MSVC toolchain target directly with spoon: msvc-14.44.35207 + sdk-10.0.26100.1")),
            "{:?}",
            result.output
        );
    let canonical = read_canonical_state_for_test(&tool_root).expect("canonical state");
    assert_eq!(canonical.runtime_kind, crate::msvc::MsvcRuntimeKind::Managed);
    assert!(canonical.installed);
    assert_eq!(canonical.version.as_deref(), Some("14.44.35207"));
    assert_eq!(canonical.sdk_version.as_deref(), Some("10.0.26100.1"));
    assert_eq!(
        canonical.last_operation,
        Some(crate::msvc::MsvcOperationKind::Install)
    );
    assert_eq!(
        canonical.last_stage,
        Some(crate::msvc::MsvcLifecycleStage::Completed)
    );
}

#[test]
fn install_toolchain_does_not_commit_canonical_state_when_payload_hash_is_invalid() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("install-invalid-hash");
    let payload_root = temp_dir("install-invalid-hash-payloads");
    fs::create_dir_all(&payload_root).unwrap();
    let sdk_payload = payload_root.join("sdk-tools.msi");
    let sdk_bytes = b"fake sdk payload bytes";
    fs::write(&sdk_payload, sdk_bytes).unwrap();
    let sdk_url = format!(
        "file:///{}",
        sdk_payload.display().to_string().replace('\\', "/")
    );

    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    fs::create_dir_all(&manifest_root).unwrap();
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
                            "sha256": "deadbeef"
                        }
                    ]
                }
            ]
        })
        .to_string(),
    )
    .unwrap();

    let err = block_on(install_toolchain_async(&tool_root)).expect_err("install should fail");
    assert!(
        err.to_string().contains("invalid payload sha256")
            || err.to_string().contains("invalid package"),
        "{err:#}"
    );
    assert!(
        read_canonical_state_for_test(&tool_root).is_none(),
        "failed install must not leave canonical state behind"
    );
}

#[test]
fn install_toolchain_caches_selected_payload_archives() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("install-cache-payloads");
    let payload_root = temp_dir("payload-source");
    fs::create_dir_all(&payload_root).unwrap();
    let crt_payload = payload_root.join("crt-headers.msi");
    let sdk_payload = payload_root.join("sdk-tools.msi");
    let crt_bytes = b"fake crt payload bytes";
    let sdk_bytes = b"fake sdk payload bytes";
    fs::write(&crt_payload, crt_bytes).unwrap();
    fs::write(&sdk_payload, sdk_bytes).unwrap();
    let crt_sha = sha256_hex(crt_bytes);
    let sdk_sha = sha256_hex(sdk_bytes);
    let crt_url = format!(
        "file:///{}",
        crt_payload.display().to_string().replace('\\', "/")
    );
    let sdk_url = format!(
        "file:///{}",
        sdk_payload.display().to_string().replace('\\', "/")
    );

    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    fs::create_dir_all(&manifest_root).unwrap();
    fs::write(
            manifest_root.join("latest.json"),
            format!(
                r#"{{
              "packages": [
                {{
                  "id": "Microsoft.VC.14.44.35207.CRT.Headers.base",
                  "version": "14.44.35207",
                  "language": "neutral",
                  "payloads": [
                    {{
                      "url": "{}",
                      "fileName": "Installers\\crt-headers.msi",
                      "sha256": "{}"
                    }}
                  ]
                }},
                {{
                  "id": "Microsoft.VC.14.44.35207.Tools.HostX64.TargetX64.base",
                  "version": "14.44.35207",
                  "language": "neutral",
                  "payloads": []
                }},
                {{
                  "id": "WindowsSdkPackageB",
                  "version": "10.0.26100.1",
                  "language": "en-US",
                  "payloads": [
                    {{
                      "url": "{}",
                      "fileName": "Installers\\Windows SDK for Windows Store Apps Tools-x86_en-us.msi",
                      "sha256": "{}"
                    }}
                  ]
                }}
              ]
            }}"#,
                crt_url,
                crt_sha,
                sdk_url,
                sdk_sha
            ),
        )
        .unwrap();

    let result = block_on(install_toolchain_async(&tool_root)).expect("install toolchain");
    assert!(result.is_success(), "{:?}", result.output);
    assert!(
        result
            .output
            .iter()
            .any(|line| line.contains("Caching 2 MSVC payload archives under")),
        "{:?}",
        result.output
    );
    assert!(
        result.output.iter().any(|line| line.contains(
            "Cached payload plan for msvc-14.44.35207 + sdk-10.0.26100.1 (downloaded 2, reused 0)."
        )),
        "{:?}",
        result.output
    );
    assert!(
        result
            .output
            .iter()
            .any(|line| line.contains("Skipped MSI media inspection for 2 unreadable payload(s).")),
        "{:?}",
        result.output
    );
    let payload_cache = config::msvc_cache_root_from(&tool_root).join("archives");
    assert_eq!(fs::read_dir(&payload_cache).unwrap().count(), 2);
}

#[test]
fn install_toolchain_prepares_extracted_zip_payloads_in_cache() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("install-extract-payloads");
    let payload_root = temp_dir("zip-payload-source");
    fs::create_dir_all(&payload_root).unwrap();
    let vsix_payload = payload_root.join("tools-base.vsix");
    let sdk_payload = payload_root.join("sdk-tools.msi");
    let inner_bytes = b"fake tool payload";
    write_small_zip(&vsix_payload, "Contents/tool.txt", inner_bytes);
    let sdk_bytes = b"fake sdk payload bytes";
    fs::write(&sdk_payload, sdk_bytes).unwrap();
    let vsix_sha = sha256_hex(&fs::read(&vsix_payload).unwrap());
    let sdk_sha = sha256_hex(sdk_bytes);
    let vsix_url = format!(
        "file:///{}",
        vsix_payload.display().to_string().replace('\\', "/")
    );
    let sdk_url = format!(
        "file:///{}",
        sdk_payload.display().to_string().replace('\\', "/")
    );

    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    fs::create_dir_all(&manifest_root).unwrap();
    fs::write(
            manifest_root.join("latest.json"),
            serde_json::json!({
                "packages": [
                    {
                        "id": "Microsoft.VC.14.44.35207.Tools.HostX64.TargetX64.base",
                        "version": "14.44.35207",
                        "language": "neutral",
                        "payloads": [
                            {
                                "url": vsix_url,
                                "fileName": "Installers\\tools-base.vsix",
                                "sha256": vsix_sha
                            }
                        ]
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
        .unwrap();

    let result = block_on(install_toolchain_async(&tool_root)).expect("install toolchain");
    assert!(result.is_success(), "{:?}", result.output);
    assert!(
        result.output.iter().any(|line| line
            .contains("Prepared MSI media metadata (inspected 0, reused 0, external cabs 0).")),
        "{:?}",
        result.output
    );
    assert!(
        result
            .output
            .iter()
            .any(|line| line.contains("Skipped MSI media inspection for 1 unreadable payload(s).")),
        "{:?}",
        result.output
    );
    assert!(
        result.output.iter().any(|line| line
            .contains("Prepared extracted archive payloads (extracted 1, reused 0, skipped 1).")),
        "{:?}",
        result.output
    );
    let extracted_root = config::msvc_cache_root_from(&tool_root)
        .join("expanded")
        .join("archives")
        .join(vsix_sha);
    assert!(extracted_root.join(".complete").exists());
    assert!(extracted_root.join("Contents").join("tool.txt").exists());
}

#[test]
fn install_toolchain_reports_zero_msi_metadata_when_plan_has_no_msi_payloads() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("install-no-msi");
    let payload_root = temp_dir("no-msi-payload-source");
    fs::create_dir_all(&payload_root).unwrap();
    let vsix_payload = payload_root.join("tools-base.vsix");
    let sdk_zip_payload = payload_root.join("sdk-tools.zip");
    write_small_zip(&vsix_payload, "Contents/tool.txt", b"fake tool payload");
    write_small_zip(&sdk_zip_payload, "sdk/tool.txt", b"fake sdk zip payload");
    let vsix_sha = sha256_hex(&fs::read(&vsix_payload).unwrap());
    let sdk_zip_sha = sha256_hex(&fs::read(&sdk_zip_payload).unwrap());
    let vsix_url = format!(
        "file:///{}",
        vsix_payload.display().to_string().replace('\\', "/")
    );
    let sdk_zip_url = format!(
        "file:///{}",
        sdk_zip_payload.display().to_string().replace('\\', "/")
    );

    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    fs::create_dir_all(&manifest_root).unwrap();
    fs::write(
            manifest_root.join("latest.json"),
            serde_json::json!({
                "packages": [
                    {
                        "id": "Microsoft.VC.14.44.35207.Tools.HostX64.TargetX64.base",
                        "version": "14.44.35207",
                        "language": "neutral",
                        "payloads": [
                            {
                                "url": vsix_url,
                                "fileName": "Installers\\tools-base.vsix",
                                "sha256": vsix_sha
                            }
                        ]
                    },
                    {
                        "id": "WindowsSdkPackageB",
                        "version": "10.0.26100.1",
                        "language": "en-US",
                        "payloads": [
                            {
                                "url": sdk_zip_url,
                                "fileName": "Installers\\Windows SDK for Windows Store Apps Tools-x86_en-us.zip",
                                "sha256": sdk_zip_sha
                            }
                        ]
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();

    let result = block_on(install_toolchain_async(&tool_root)).expect("install toolchain");
    assert!(result.is_success(), "{:?}", result.output);
    assert!(
        result.output.iter().any(|line| line
            .contains("Prepared MSI media metadata (inspected 0, reused 0, external cabs 0).")),
        "{:?}",
        result.output
    );
    assert!(
            result
                .output
                .iter()
                .any(|line| line.contains("Prepared external CAB payload cache plan for msvc-14.44.35207 + sdk-10.0.26100.1 (selected 0).")),
            "{:?}",
            result.output
        );
}

#[test]
fn install_toolchain_caches_companion_cab_payloads_from_cached_msi_metadata() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("install-cab-companions");
    let payload_root = temp_dir("cab-payload-source");
    fs::create_dir_all(&payload_root).unwrap();
    let msi_payload = payload_root.join("sdk-tools.msi");
    let cab_payload = payload_root.join("sdk-tools.cab");
    let msi_bytes = b"fake sdk payload bytes";
    let cab_bytes = b"fake cab payload bytes";
    fs::write(&msi_payload, msi_bytes).unwrap();
    fs::write(&cab_payload, cab_bytes).unwrap();
    let msi_sha = sha256_hex(msi_bytes);
    let cab_sha = sha256_hex(cab_bytes);
    let msi_url = format!(
        "file:///{}",
        msi_payload.display().to_string().replace('\\', "/")
    );
    let cab_url = format!(
        "file:///{}",
        cab_payload.display().to_string().replace('\\', "/")
    );

    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    fs::create_dir_all(&manifest_root).unwrap();
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
                                "url": msi_url,
                                "fileName": "Installers\\Windows SDK for Windows Store Apps Tools-x86_en-us.msi",
                                "sha256": msi_sha
                            },
                            {
                                "url": cab_url,
                                "fileName": "Installers\\sdk-tools.cab",
                                "sha256": cab_sha
                            }
                        ]
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();

    let metadata_root = config::msvc_cache_root_from(&tool_root)
        .join("metadata")
        .join("msi");
    fs::create_dir_all(&metadata_root).unwrap();
    fs::write(
        metadata_root.join(format!("{msi_sha}.txt")),
        "sdk-tools.cab\n",
    )
    .unwrap();

    let result = block_on(install_toolchain_async(&tool_root)).expect("install toolchain");
    assert!(result.is_success(), "{:?}", result.output);
    assert!(
            result
                .output
                .iter()
                .any(|line| line.contains("Prepared external CAB payload cache plan for msvc-14.44.35207 + sdk-10.0.26100.1 (selected 1).")),
            "{:?}",
            result.output
        );
    assert!(
        result.output.iter().any(|line| line.contains(
            "Prepared MSI staging dirs for external CABs (staged 1, reused 0, skipped 0)."
        )),
        "{:?}",
        result.output
    );
    let payload_cache = config::msvc_cache_root_from(&tool_root).join("archives");
    assert!(fs::read_dir(&payload_cache).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("sdk-tools.cab")
    }));
    let staging_dir = config::msvc_cache_root_from(&tool_root)
        .join("stage")
        .join("msi")
        .join(msi_sha);
    assert!(staging_dir.join("sdk-tools.cab").exists());
    assert!(staging_dir.join(".complete").exists());
}

#[test]
fn install_toolchain_builds_install_image_from_extracted_payloads() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("install-image");
    let payload_root = temp_dir("install-image-payload-source");
    fs::create_dir_all(&payload_root).unwrap();
    let vsix_payload = payload_root.join("tools-base.vsix");
    write_small_zip(&vsix_payload, "Contents/tool.txt", b"fake tool payload");
    let sdk_payload = payload_root.join("sdk-tools.msi");
    let cab_payload = payload_root.join("sdk-tools.cab");
    fs::write(&sdk_payload, b"fake sdk payload bytes").unwrap();
    fs::write(&cab_payload, b"fake cab payload bytes").unwrap();
    let vsix_sha = sha256_hex(&fs::read(&vsix_payload).unwrap());
    let sdk_sha = sha256_hex(b"fake sdk payload bytes");
    let cab_sha = sha256_hex(b"fake cab payload bytes");

    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    fs::create_dir_all(&manifest_root).unwrap();
    fs::write(
            manifest_root.join("latest.json"),
            serde_json::json!({
                "packages": [
                    {
                        "id": "Microsoft.VC.14.44.35207.Tools.HostX64.TargetX64.base",
                        "version": "14.44.35207",
                        "language": "neutral",
                        "payloads": [
                            {
                                "url": format!("file:///{}", vsix_payload.display().to_string().replace('\\', "/")),
                                "fileName": "Installers\\tools-base.vsix",
                                "sha256": vsix_sha
                            }
                        ]
                    },
                    {
                        "id": "WindowsSdkPackageB",
                        "version": "10.0.26100.1",
                        "language": "en-US",
                        "payloads": [
                            {
                                "url": format!("file:///{}", sdk_payload.display().to_string().replace('\\', "/")),
                                "fileName": "Installers\\Windows SDK for Windows Store Apps Tools-x86_en-us.msi",
                                "sha256": sdk_sha
                            },
                            {
                                "url": format!("file:///{}", cab_payload.display().to_string().replace('\\', "/")),
                                "fileName": "Installers\\sdk-tools.cab",
                                "sha256": cab_sha
                            }
                        ]
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();

    let metadata_root = config::msvc_cache_root_from(&tool_root)
        .join("metadata")
        .join("msi");
    fs::create_dir_all(&metadata_root).unwrap();
    fs::write(
        metadata_root.join(format!("{sdk_sha}.txt")),
        "sdk-tools.cab\n",
    )
    .unwrap();

    let extracted_msi_root = config::msvc_cache_root_from(&tool_root)
        .join("expanded")
        .join("msi")
        .join(&sdk_sha);
    fs::create_dir_all(&extracted_msi_root).unwrap();
    fs::write(extracted_msi_root.join("sdk.txt"), b"fake sdk extracted").unwrap();
    fs::write(extracted_msi_root.join(".complete"), b"ok").unwrap();

    let result = block_on(install_toolchain_async(&tool_root)).expect("install toolchain");
    assert!(result.is_success(), "{:?}", result.output);
    assert!(
        result.output.iter().any(|line| line
            .contains("Prepared install image from extracted payloads (copied 2, skipped 0).")),
        "{:?}",
        result.output
    );
    assert!(
        result
            .output
            .iter()
            .any(|line| line.contains("Cleaned transient MSVC install-image cache after install")),
        "{:?}",
        result.output
    );
    assert!(
        result
            .output
            .iter()
            .any(|line| line.contains("Retained MSI extraction cache under")),
        "{:?}",
        result.output
    );
    assert!(
        result
            .output
            .iter()
            .any(|line| line.contains("Retained MSI staging cache under")),
        "{:?}",
        result.output
    );

    let image_root = config::msvc_cache_root_from(&tool_root).join("image");
    assert!(!image_root.exists());
    assert!(
        config::msvc_cache_root_from(&tool_root)
            .join("expanded")
            .exists()
    );
    assert!(
        config::msvc_cache_root_from(&tool_root)
            .join("stage")
            .exists()
    );

    let toolchain_root = config::msvc_toolchain_root_from(&tool_root);
    assert!(toolchain_root.join("tool.txt").exists());
    assert!(toolchain_root.join("sdk.txt").exists());
    let installed = config::msvc_state_root_from(&tool_root).join("installed.json");
    let installed_content = fs::read_to_string(installed).unwrap();
    assert!(installed_content.contains("msvc-14.44.35207"));
    assert!(installed_content.contains("sdk-10.0.26100.1"));
}

#[test]
fn install_toolchain_keeps_selected_target_arch_only_in_install_image() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("install-image-merge");
    let payload_root = temp_dir("install-image-merge-payloads");
    fs::create_dir_all(&payload_root).unwrap();
    let arm_vsix_payload = payload_root.join("tools-arm.vsix");
    let x64_vsix_payload = payload_root.join("tools-x64.vsix");
    let sdk_zip_payload = payload_root.join("sdk-tools.zip");
    write_small_zip(
        &arm_vsix_payload,
        "Contents/VC/Tools/MSVC/14.44.35207/bin/Hostx64/arm/cl.exe",
        b"fake arm cl",
    );
    write_small_zip(
        &x64_vsix_payload,
        "Contents/VC/Tools/MSVC/14.44.35207/bin/Hostx64/x64/cl.exe",
        b"fake x64 cl",
    );
    write_small_zip(&sdk_zip_payload, "sdk/tool.txt", b"fake sdk zip payload");
    let arm_sha = sha256_hex(&fs::read(&arm_vsix_payload).unwrap());
    let x64_sha = sha256_hex(&fs::read(&x64_vsix_payload).unwrap());
    let sdk_zip_sha = sha256_hex(&fs::read(&sdk_zip_payload).unwrap());

    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    fs::create_dir_all(&manifest_root).unwrap();
    fs::write(
            manifest_root.join("latest.json"),
            serde_json::json!({
                "packages": [
                    {
                        "id": "Microsoft.VC.14.44.35207.Tools.HostX64.TargetARM.base",
                        "version": "14.44.35207",
                        "language": "neutral",
                        "payloads": [
                            {
                                "url": format!("file:///{}", arm_vsix_payload.display().to_string().replace('\\', "/")),
                                "fileName": "Installers\\tools-arm.vsix",
                                "sha256": arm_sha
                            }
                        ]
                    },
                    {
                        "id": "Microsoft.VC.14.44.35207.Tools.HostX64.TargetX64.base",
                        "version": "14.44.35207",
                        "language": "neutral",
                        "payloads": [
                            {
                                "url": format!("file:///{}", x64_vsix_payload.display().to_string().replace('\\', "/")),
                                "fileName": "Installers\\tools-x64.vsix",
                                "sha256": x64_sha
                            }
                        ]
                    },
                    {
                        "id": "WindowsSdkPackageB",
                        "version": "10.0.26100.1",
                        "language": "en-US",
                        "payloads": [
                            {
                                "url": format!("file:///{}", sdk_zip_payload.display().to_string().replace('\\', "/")),
                                "fileName": "Installers\\Windows SDK for Windows Store Apps Tools-x86_en-us.zip",
                                "sha256": sdk_zip_sha
                            }
                        ]
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();

    let result = block_on(install_toolchain_async(&tool_root)).expect("install toolchain");
    assert!(result.is_success(), "{:?}", result.output);

    let image_root = config::msvc_cache_root_from(&tool_root).join("image");
    assert!(!image_root.exists());

    let toolchain_root = config::msvc_toolchain_root_from(&tool_root);
    assert!(
        !toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("arm")
            .join("cl.exe")
            .exists()
    );
    assert!(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("cl.exe")
            .exists()
    );
}

#[test]
fn install_toolchain_rebuilds_install_image_from_current_extracted_payloads() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("install-image-rebuild");
    let payload_root = temp_dir("install-image-rebuild-payloads");
    fs::create_dir_all(&payload_root).unwrap();
    let desktop_headers_payload = payload_root.join("desktop-headers.msi");
    let store_headers_payload = payload_root.join("store-headers.msi");
    let desktop_headers_bytes = b"fake desktop headers msi";
    let store_headers_bytes = b"fake store headers msi";
    fs::write(&desktop_headers_payload, desktop_headers_bytes).unwrap();
    fs::write(&store_headers_payload, store_headers_bytes).unwrap();
    let desktop_sha = sha256_hex(desktop_headers_bytes);
    let store_sha = sha256_hex(store_headers_bytes);
    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    fs::create_dir_all(&manifest_root).unwrap();
    fs::write(
            manifest_root.join("latest.json"),
            serde_json::json!({
                "packages": [
                    {
                        "id": "Microsoft.VC.14.44.17.14.Tools.HostX64.TargetX64.base",
                        "version": "14.44.17.14",
                        "language": "neutral",
                        "payloads": []
                    },
                    {
                        "id": "WindowsSdkPackageA",
                        "version": "10.0.26100.1",
                        "language": "en-US",
                        "payloads": [
                            {
                                "url": format!("file:///{}", desktop_headers_payload.display().to_string().replace('\\', "/")),
                                "fileName": "Installers\\Windows SDK Desktop Headers x64-x86_en-us.msi",
                                "sha256": desktop_sha
                            },
                            {
                                "url": format!("file:///{}", store_headers_payload.display().to_string().replace('\\', "/")),
                                "fileName": "Installers\\Windows SDK for Windows Store Apps Headers-x86_en-us.msi",
                                "sha256": store_sha
                            }
                        ]
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();

    let old_image_root = config::msvc_cache_root_from(&tool_root).join("image");
    fs::create_dir_all(old_image_root.join("stale")).unwrap();
    fs::write(old_image_root.join("stale").join("obsolete.txt"), b"old").unwrap();

    let desktop_extract = config::msvc_cache_root_from(&tool_root)
        .join("expanded")
        .join("msi")
        .join(&desktop_sha);
    fs::create_dir_all(
        desktop_extract
            .join("Windows Kits")
            .join("10")
            .join("Include")
            .join("10.0.26100.0")
            .join("shared"),
    )
    .unwrap();
    fs::write(
        desktop_extract
            .join("Windows Kits")
            .join("10")
            .join("Include")
            .join("10.0.26100.0")
            .join("shared")
            .join("winapifamily.h"),
        b"",
    )
    .unwrap();
    fs::write(desktop_extract.join(".complete"), b"ok").unwrap();

    let store_extract = config::msvc_cache_root_from(&tool_root)
        .join("expanded")
        .join("msi")
        .join(&store_sha);
    fs::create_dir_all(
        store_extract
            .join("Windows Kits")
            .join("10")
            .join("Include")
            .join("10.0.26100.0")
            .join("um"),
    )
    .unwrap();
    fs::write(
        store_extract
            .join("Windows Kits")
            .join("10")
            .join("Include")
            .join("10.0.26100.0")
            .join("um")
            .join("Windows.h"),
        b"",
    )
    .unwrap();
    fs::write(store_extract.join(".complete"), b"ok").unwrap();

    let result = block_on(install_toolchain_async(&tool_root)).expect("install toolchain");
    assert!(result.is_success(), "{:?}", result.output);

    let image_root = config::msvc_cache_root_from(&tool_root).join("image");
    assert!(!image_root.exists());

    let toolchain_root = config::msvc_toolchain_root_from(&tool_root);
    assert!(
        toolchain_root
            .join("Windows Kits")
            .join("10")
            .join("Include")
            .join("10.0.26100.0")
            .join("shared")
            .join("winapifamily.h")
            .exists()
    );
    assert!(
        toolchain_root
            .join("Windows Kits")
            .join("10")
            .join("Include")
            .join("10.0.26100.0")
            .join("um")
            .join("Windows.h")
            .exists()
    );
}

#[test]
fn install_toolchain_uses_cached_manifest_without_runtime() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("managed");
    let payload_root = temp_dir("managed-payload-source");
    fs::create_dir_all(&payload_root).unwrap();
    let sdk_payload = payload_root.join("sdk-tools.msi");
    let sdk_bytes = b"fake sdk payload bytes";
    fs::write(&sdk_payload, sdk_bytes).unwrap();
    let sdk_url = format!(
        "file:///{}",
        sdk_payload.display().to_string().replace('\\', "/")
    );
    let sdk_sha = sha256_hex(sdk_bytes);
    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    fs::create_dir_all(&manifest_root).unwrap();
    fs::write(
            manifest_root.join("latest.json"),
            serde_json::json!({
                "packages": [
                    {
                        "id": "Microsoft.VC.14.44.17.14.Tools.HostX64.TargetX64.base",
                        "version": "14.44.17.14",
                        "language": "neutral",
                        "payloads": []
                    },
                    {
                        "id": "WindowsSdkPackageB",
                        "version": "10.0.22621.7",
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
        .unwrap();

    let result = block_on(install_toolchain_async(&tool_root)).expect("install toolchain");
    assert!(result.is_success(), "{:?}", result.output);
    assert!(
        result.output.iter().any(
            |line| line.contains("Installed latest MSVC toolchain target directly with spoon:")
        ),
        "{:?}",
        result.output
    );
    assert!(
        result
            .output
            .iter()
            .any(|line| line.contains("Wrote managed runtime state into")),
        "{:?}",
        result.output
    );
    assert!(
        result
            .output
            .iter()
            .any(|line| line.contains("Managed wrappers are materialized under")),
        "{:?}",
        result.output
    );
    assert!(
        config::msvc_state_root_from(&tool_root)
            .join("runtime.json")
            .exists()
    );
}

#[test]
fn update_toolchain_uses_cached_manifest() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("update");
    let payload_root = temp_dir("update-payload-source");
    fs::create_dir_all(&payload_root).unwrap();
    let sdk_payload = payload_root.join("sdk-tools.msi");
    let sdk_bytes = b"fake sdk payload bytes";
    fs::write(&sdk_payload, sdk_bytes).unwrap();
    let sdk_url = format!(
        "file:///{}",
        sdk_payload.display().to_string().replace('\\', "/")
    );
    let sdk_sha = sha256_hex(sdk_bytes);
    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    fs::create_dir_all(&manifest_root).unwrap();
    fs::write(
            manifest_root.join("latest.json"),
            serde_json::json!({
                "packages": [
                    {
                        "id": "Microsoft.VC.14.44.17.14.Tools.HostX64.TargetX64.base",
                        "version": "14.44.17.14",
                        "language": "neutral",
                        "payloads": []
                    },
                    {
                        "id": "WindowsSdkPackageB",
                        "version": "10.0.22621.7",
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
        .unwrap();

    let result = block_on(update_toolchain_async(&tool_root)).expect("update toolchain");
    assert!(result.is_success(), "{:?}", result.output);
    assert!(
        result.output.iter().any(
            |line| line.contains("Installed latest MSVC toolchain target directly with spoon:")
        ),
        "{:?}",
        result.output
    );
}

#[test]
fn update_toolchain_is_noop_when_cached_target_matches_installed_state() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("update-noop");
    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    let toolchain_root = config::msvc_toolchain_root_from(&tool_root);
    fs::create_dir_all(&manifest_root).unwrap();
    fs::create_dir_all(&toolchain_root).unwrap();
    seed_managed_state(&tool_root, "msvc-14.44.17.14", "sdk-10.0.22621.7");
    fs::write(
            manifest_root.join("latest.json"),
            serde_json::json!({
                "packages": [
                    {
                        "id": "Microsoft.VC.14.44.17.14.Tools.HostX64.TargetX64.base",
                        "version": "14.44.17.14",
                        "language": "neutral",
                        "payloads": []
                    },
                    {
                        "id": "WindowsSdkPackageB",
                        "version": "10.0.22621.7",
                        "language": "en-US",
                        "payloads": [
                            {
                                "fileName": "Installers\\Windows SDK for Windows Store Apps Tools-x86_en-us.msi"
                            }
                        ]
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();

    let result = block_on(update_toolchain_async(&tool_root)).expect("update toolchain noop");
    assert!(result.is_success(), "{:?}", result.output);
    assert!(
        result
            .output
            .iter()
            .any(|line| line.contains("Managed MSVC toolchain is already up to date:")),
        "{:?}",
        result.output
    );
    assert!(
        result
            .output
            .iter()
            .all(|line| !line
                .contains("Installed latest MSVC toolchain target directly with spoon:")),
        "{:?}",
        result.output
    );
}

#[test]
fn install_tolerates_manifest_refresh_failure_but_update_requires_it() {
    let mut install_lines = Vec::new();
    let mut install_emit: Option<&mut dyn FnMut(BackendEvent)> = None;
    handle_manifest_refresh_failure(
        ToolchainAction::Install,
        &mut install_lines,
        &mut install_emit,
        BackendError::Other("network down".to_string()),
    )
    .expect("install should tolerate refresh failure");
    assert!(
        install_lines
            .iter()
            .any(|line| line.contains("Warning: failed to refresh managed MSVC manifest cache:")),
        "{install_lines:?}"
    );

    let mut update_lines = Vec::new();
    let mut update_emit: Option<&mut dyn FnMut(BackendEvent)> = None;
    let err = handle_manifest_refresh_failure(
        ToolchainAction::Update,
        &mut update_lines,
        &mut update_emit,
        BackendError::Other("network down".to_string()),
    )
    .expect_err("update should require manifest refresh");
    assert!(
        err.to_string()
            .contains("failed to refresh latest managed MSVC manifest for update"),
        "{err:#}"
    );
    assert!(update_lines.is_empty(), "{update_lines:?}");
}

#[test]
fn uninstall_toolchain_removes_managed_wrappers() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("uninstall-wrappers");
    let toolchain_root = config::msvc_toolchain_root_from(&tool_root);
    fs::create_dir_all(&toolchain_root).unwrap();
    seed_managed_state(&tool_root, "msvc-14.44.17.14", "sdk-10.0.22621.7");
    fs::create_dir_all(shims_root(&tool_root)).unwrap();
    fs::write(shims_root(&tool_root).join("spoon-cl.cmd"), "@echo off\r\n").unwrap();
    fs::write(
        shims_root(&tool_root).join("spoon-link.cmd"),
        "@echo off\r\n",
    )
    .unwrap();
    fs::write(
        shims_root(&tool_root).join("spoon-lib.cmd"),
        "@echo off\r\n",
    )
    .unwrap();

    let result = block_on(uninstall_toolchain(&tool_root)).expect("uninstall toolchain");
    assert!(result.is_success(), "{:?}", result.output);
    assert!(!shims_root(&tool_root).join("spoon-cl.cmd").exists());
    assert!(!shims_root(&tool_root).join("spoon-link.cmd").exists());
    assert!(!shims_root(&tool_root).join("spoon-lib.cmd").exists());
    let canonical = read_canonical_state_for_test(&tool_root).expect("canonical state");
    assert!(!canonical.installed);
    assert_eq!(
        canonical.last_operation,
        Some(crate::msvc::MsvcOperationKind::Uninstall)
    );
}

#[test]
fn write_managed_toolchain_wrappers_includes_auxiliary_tools_when_present() {
    let tool_root = temp_dir("managed-aux-wrappers");
    seed_msvc_policy_command_profile(&tool_root, "extended");
    let bin_root = tool_root.join("fake-bin");
    fs::create_dir_all(&bin_root).unwrap();
    let include_root = tool_root.join("include");
    let lib_root = tool_root.join("lib");
    fs::create_dir_all(&include_root).unwrap();
    fs::create_dir_all(&lib_root).unwrap();

    let cl = bin_root.join("cl.exe");
    let link = bin_root.join("link.exe");
    let librarian = bin_root.join("lib.exe");
    let rc = bin_root.join("rc.exe");
    let mt = bin_root.join("mt.exe");
    let nmake = bin_root.join("nmake.exe");
    let dumpbin = bin_root.join("dumpbin.exe");
    for path in [&cl, &link, &librarian, &rc, &mt, &nmake, &dumpbin] {
        fs::write(path, b"").unwrap();
    }

    let flags = ToolchainFlags {
        compiler: cl,
        linker: link,
        librarian,
        resource_compiler: Some(rc),
        manifest_tool: Some(mt),
        nmake: Some(nmake),
        dumpbin: Some(dumpbin),
        include_dirs: vec![include_root],
        lib_dirs: vec![lib_root],
        path_dirs: vec![bin_root],
    };

    let lines = write_managed_toolchain_wrappers(&tool_root, "extended", &flags)
        .expect("write auxiliary wrappers");
    for name in [
        "spoon-cl.cmd",
        "spoon-link.cmd",
        "spoon-lib.cmd",
        "spoon-rc.cmd",
        "spoon-mt.cmd",
        "spoon-nmake.cmd",
        "spoon-dumpbin.cmd",
    ] {
        assert!(shims_root(&tool_root).join(name).exists(), "missing {name}");
        assert!(lines.iter().any(|line| line.contains(name)), "{lines:?}");
    }

    let _ = fs::remove_dir_all(tool_root);
}

#[test]
fn write_managed_toolchain_wrappers_default_profile_keeps_only_core_tools() {
    let tool_root = temp_dir("managed-default-wrappers");
    seed_msvc_policy_command_profile(&tool_root, "default");
    let bin_root = tool_root.join("fake-bin");
    fs::create_dir_all(&bin_root).unwrap();
    let include_root = tool_root.join("include");
    let lib_root = tool_root.join("lib");
    fs::create_dir_all(&include_root).unwrap();
    fs::create_dir_all(&lib_root).unwrap();

    let cl = bin_root.join("cl.exe");
    let link = bin_root.join("link.exe");
    let librarian = bin_root.join("lib.exe");
    let rc = bin_root.join("rc.exe");
    let mt = bin_root.join("mt.exe");
    let nmake = bin_root.join("nmake.exe");
    let dumpbin = bin_root.join("dumpbin.exe");
    for path in [&cl, &link, &librarian, &rc, &mt, &nmake, &dumpbin] {
        fs::write(path, b"").unwrap();
    }

    let flags = ToolchainFlags {
        compiler: cl,
        linker: link,
        librarian,
        resource_compiler: Some(rc),
        manifest_tool: Some(mt),
        nmake: Some(nmake),
        dumpbin: Some(dumpbin),
        include_dirs: vec![include_root],
        lib_dirs: vec![lib_root],
        path_dirs: vec![bin_root],
    };

    let lines = write_managed_toolchain_wrappers(&tool_root, "default", &flags)
        .expect("write default wrappers");
    for name in ["spoon-cl.cmd", "spoon-link.cmd", "spoon-lib.cmd"] {
        assert!(shims_root(&tool_root).join(name).exists(), "missing {name}");
        assert!(lines.iter().any(|line| line.contains(name)), "{lines:?}");
    }
    for name in [
        "spoon-rc.cmd",
        "spoon-mt.cmd",
        "spoon-nmake.cmd",
        "spoon-dumpbin.cmd",
    ] {
        assert!(
            !shims_root(&tool_root).join(name).exists(),
            "unexpected {name}"
        );
    }
}

#[test]
fn install_toolchain_streaming_emits_stage_lines_during_execution() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("streaming-progress");
    let payload_root = temp_dir("streaming-payload-source");
    fs::create_dir_all(&payload_root).unwrap();
    let sdk_payload = payload_root.join("sdk-tools.msi");
    let sdk_bytes = b"fake sdk payload bytes";
    fs::write(&sdk_payload, sdk_bytes).unwrap();
    let sdk_url = format!(
        "file:///{}",
        sdk_payload.display().to_string().replace('\\', "/")
    );
    let sdk_sha = sha256_hex(sdk_bytes);
    let manifest_root = msvc_manifest_root(&tool_root).join("vs");
    fs::create_dir_all(&manifest_root).unwrap();
    fs::write(
            manifest_root.join("latest.json"),
            serde_json::json!({
                "packages": [
                    {
                        "id": "Microsoft.VC.14.44.17.14.Tools.HostX64.TargetX64.base",
                        "version": "14.44.17.14",
                        "language": "neutral",
                        "payloads": []
                    },
                    {
                        "id": "WindowsSdkPackageB",
                        "version": "10.0.22621.7",
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
        .unwrap();

    let metadata_root = config::msvc_cache_root_from(&tool_root)
        .join("metadata")
        .join("msi");
    fs::create_dir_all(&metadata_root).unwrap();
    fs::write(
        metadata_root.join(format!("{sdk_sha}.txt")),
        "# no external cabs\n",
    )
    .unwrap();

    let staging_root = config::msvc_cache_root_from(&tool_root)
        .join("stage")
        .join("msi")
        .join(&sdk_sha);
    fs::create_dir_all(&staging_root).unwrap();
    fs::write(staging_root.join(".complete"), b"ok").unwrap();

    let mut streamed = Vec::new();
    let result = block_on(install_toolchain_streaming(
        &tool_root,
        None,
        &mut |chunk| streamed.push(chunk),
    ))
    .expect("streaming install toolchain");
    assert!(result.is_success(), "{:?}", result.output);
    assert!(
        streamed.iter().any(|chunk| matches!(
            chunk,
            BackendEvent::Progress(progress) if progress.label.contains("Caching payload 1/1:")
        )),
        "{streamed:?}"
    );
    assert!(
        streamed.iter().any(|chunk| matches!(
            chunk,
            BackendEvent::Progress(progress) if progress.label.contains("Extracting MSI payload 1/1:")
        )),
        "{streamed:?}"
    );
}

#[test]
fn validate_toolchain_prefers_native_host_target_compiler() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("validate-preferred-native");
    let toolchain_root = config::msvc_toolchain_root_from(&tool_root);
    seed_managed_state(&tool_root, "14.44.35207", "10.0.26100.1");
    fs::create_dir_all(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostarm64")
            .join("arm"),
    )
    .unwrap();
    fs::create_dir_all(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64"),
    )
    .unwrap();
    fs::create_dir_all(toolchain_root.join("include").join("shared")).unwrap();
    fs::create_dir_all(toolchain_root.join("include").join("ucrt")).unwrap();
    fs::create_dir_all(toolchain_root.join("include").join("um")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("ucrt").join("x64")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("um").join("x64")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("vc").join("x64")).unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostarm64")
            .join("arm")
            .join("cl.cmd"),
        "@echo off\r\nexit /b 1\r\n",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostarm64")
            .join("arm")
            .join("link.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("lib.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("lib.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("lib.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("lib.exe"),
        b"",
    )
    .unwrap();
    fs::write(
            toolchain_root
                .join("VC")
                .join("Tools")
                .join("MSVC")
                .join("14.44.35207")
                .join("bin")
                .join("Hostx64")
                .join("x64")
                .join("cl.cmd"),
            "@echo off\r\ncopy /Y \"%SystemRoot%\\System32\\whoami.exe\" hello.exe >nul\r\nexit /b 0\r\n",
        )
        .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("link.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("lib.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root.join("include").join("ucrt").join("stdio.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("include")
            .join("ucrt")
            .join("corecrt.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("include")
            .join("shared")
            .join("winapifamily.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root.join("include").join("um").join("Windows.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("ucrt")
            .join("x64")
            .join("ucrt.lib"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("um")
            .join("x64")
            .join("kernel32.lib"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("vc")
            .join("x64")
            .join("libcmt.lib"),
        b"",
    )
    .unwrap();

    let result = block_on(validate_toolchain(&tool_root)).expect("validate toolchain");
    assert!(result.is_success(), "{:?}", result.output);
    let wrapper = fs::read_to_string(shims_root(&tool_root).join("spoon-cl.cmd")).unwrap();
    assert!(
        wrapper.contains("Hostx64") && wrapper.contains("\\x64\\") && wrapper.contains("cl.cmd"),
        "{wrapper}"
    );
    assert!(
        result
            .output
            .iter()
            .any(|line| line.contains("Ran managed validation sample successfully")),
        "{:?}",
        result.output
    );
}

#[test]
fn validate_toolchain_keeps_artifacts_by_default_for_inspection() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("validate-keep-artifacts");
    let toolchain_root = config::msvc_toolchain_root_from(&tool_root);
    seed_managed_state(&tool_root, "msvc-14.44.35207", "sdk-10.0.26100.15");
    fs::create_dir_all(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64"),
    )
    .unwrap();
    fs::create_dir_all(toolchain_root.join("include").join("ucrt")).unwrap();
    fs::create_dir_all(toolchain_root.join("include").join("shared")).unwrap();
    fs::create_dir_all(toolchain_root.join("include").join("um")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("ucrt").join("x64")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("um").join("x64")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("vc").join("x64")).unwrap();
    fs::write(
            toolchain_root
                .join("VC")
                .join("Tools")
                .join("MSVC")
                .join("14.44.35207")
                .join("bin")
                .join("Hostx64")
                .join("x64")
                .join("cl.cmd"),
            "@echo off\r\ncopy /Y \"%SystemRoot%\\System32\\whoami.exe\" hello.exe >nul\r\nexit /b 0\r\n",
        )
        .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("link.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("lib.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root.join("include").join("ucrt").join("stdio.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("include")
            .join("ucrt")
            .join("corecrt.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("include")
            .join("shared")
            .join("winapifamily.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root.join("include").join("um").join("Windows.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("ucrt")
            .join("x64")
            .join("ucrt.lib"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("um")
            .join("x64")
            .join("kernel32.lib"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("um")
            .join("x64")
            .join("user32.lib"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("vc")
            .join("x64")
            .join("libcmt.lib"),
        b"",
    )
    .unwrap();

    let result = block_on(validate_toolchain(&tool_root)).expect("validate keep artifacts");
    assert!(result.is_success(), "{:?}", result.output);
    let canonical = read_canonical_state_for_test(&tool_root).expect("canonical state");
    assert_eq!(
        canonical.last_operation,
        Some(crate::msvc::MsvcOperationKind::Validate)
    );
    assert_eq!(
        canonical.validation_status,
        Some(crate::msvc::MsvcValidationStatus::Valid)
    );
    assert!(
        result
            .output
            .iter()
            .any(|line| line.contains("Kept validation workspace")),
        "{:?}",
        result.output
    );
    let validate_root = config::msvc_cache_root_from(&tool_root).join("validate");
    assert!(validate_root.join("cpp").join("hello.cpp").exists());
    assert!(validate_root.join("cpp").join("hello.exe").exists());
    assert!(
        validate_root
            .join("rust")
            .join("src")
            .join("main.rs")
            .exists()
    );
    assert!(validate_root.join("rust").join("build.rs").exists());
    assert!(
        validate_root
            .join("rust")
            .join("native")
            .join("helper.c")
            .exists()
    );
}

#[test]
fn validate_toolchain_writes_reusable_build_and_run_scripts() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("validate-scripts");
    let toolchain_root = config::msvc_toolchain_root_from(&tool_root);
    seed_managed_state(&tool_root, "msvc-14.44.35207", "sdk-10.0.26100.15");
    fs::create_dir_all(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64"),
    )
    .unwrap();
    fs::create_dir_all(toolchain_root.join("include").join("ucrt")).unwrap();
    fs::create_dir_all(toolchain_root.join("include").join("shared")).unwrap();
    fs::create_dir_all(toolchain_root.join("include").join("um")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("ucrt").join("x64")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("um").join("x64")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("vc").join("x64")).unwrap();
    fs::write(
            toolchain_root
                .join("VC")
                .join("Tools")
                .join("MSVC")
                .join("14.44.35207")
                .join("bin")
                .join("Hostx64")
                .join("x64")
                .join("cl.cmd"),
            "@echo off\r\ncopy /Y \"%SystemRoot%\\System32\\whoami.exe\" hello.exe >nul\r\nexit /b 0\r\n",
        )
        .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("link.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("lib.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root.join("include").join("ucrt").join("stdio.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("include")
            .join("ucrt")
            .join("corecrt.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("include")
            .join("shared")
            .join("winapifamily.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root.join("include").join("um").join("Windows.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("ucrt")
            .join("x64")
            .join("ucrt.lib"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("um")
            .join("x64")
            .join("kernel32.lib"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("um")
            .join("x64")
            .join("user32.lib"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("vc")
            .join("x64")
            .join("libcmt.lib"),
        b"",
    )
    .unwrap();
    let result = block_on(validate_toolchain(&tool_root)).expect("validate toolchain with scripts");
    assert!(result.is_success(), "{:?}", result.output);
    let validate_root = config::msvc_cache_root_from(&tool_root).join("validate");
    assert!(
        validate_root.join("cpp").join("build.cmd").exists(),
        "{}",
        validate_root.display()
    );
    assert!(
        validate_root.join("rust").join("build.cmd").exists(),
        "{}",
        validate_root.display()
    );
    assert!(
        !validate_root.join("cpp").join("run.cmd").exists(),
        "{}",
        validate_root.display()
    );
    let build_script = fs::read_to_string(validate_root.join("cpp").join("build.cmd")).unwrap();
    assert!(
        build_script.contains("spoon-cl.cmd")
            && build_script.contains("user32.lib")
            && build_script.contains("hello.cpp"),
        "{build_script}"
    );
    let rust_build_script =
        fs::read_to_string(validate_root.join("rust").join("build.cmd")).unwrap();
    assert!(
        rust_build_script.contains("cargo")
            && rust_build_script.contains("SPOON_VALIDATE_SPOON_CL"),
        "{rust_build_script}"
    );
}

#[test]
fn validate_toolchain_writes_visible_sample_source() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("validate-visible-sample");
    let toolchain_root = config::msvc_toolchain_root_from(&tool_root);
    seed_managed_state(&tool_root, "msvc-14.44.35207", "sdk-10.0.26100.15");
    fs::create_dir_all(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64"),
    )
    .unwrap();
    fs::create_dir_all(toolchain_root.join("include").join("ucrt")).unwrap();
    fs::create_dir_all(toolchain_root.join("include").join("shared")).unwrap();
    fs::create_dir_all(toolchain_root.join("include").join("um")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("ucrt").join("x64")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("um").join("x64")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("vc").join("x64")).unwrap();
    fs::write(
            toolchain_root
                .join("VC")
                .join("Tools")
                .join("MSVC")
                .join("14.44.35207")
                .join("bin")
                .join("Hostx64")
                .join("x64")
                .join("cl.cmd"),
            "@echo off\r\ncopy /Y \"%SystemRoot%\\System32\\whoami.exe\" hello.exe >nul\r\nexit /b 0\r\n",
        )
        .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("link.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("lib.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root.join("include").join("ucrt").join("stdio.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("include")
            .join("ucrt")
            .join("corecrt.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("include")
            .join("shared")
            .join("winapifamily.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root.join("include").join("um").join("Windows.h"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("ucrt")
            .join("x64")
            .join("ucrt.lib"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("um")
            .join("x64")
            .join("kernel32.lib"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("um")
            .join("x64")
            .join("user32.lib"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("vc")
            .join("x64")
            .join("libcmt.lib"),
        b"",
    )
    .unwrap();

    let result =
        block_on(validate_toolchain(&tool_root)).expect("validate toolchain visible sample");
    assert!(result.is_success(), "{:?}", result.output);
    assert!(
        result
            .output
            .iter()
            .any(|line| line.contains("Using managed wrapper compiler")
                && line.contains("spoon-cl.cmd")),
        "{:?}",
        result.output
    );
    assert!(
        result.output.iter().any(|line| {
            line.contains("Prepared managed wrapper linker")
                && line.contains("spoon-link.cmd")
                && line.contains("cl /link")
        }),
        "{:?}",
        result.output
    );
    let validate_root = config::msvc_cache_root_from(&tool_root).join("validate");
    let source = fs::read_to_string(validate_root.join("cpp").join("hello.cpp")).unwrap();
    assert!(
        source.contains("std::string msg = \"spoon msvc validate\""),
        "{source}"
    );
    assert!(source.contains("std::printf(\"sample=%s"), "{source}");
    assert!(
        source.contains("cpp_runtime=std::string+std::vector ok"),
        "{source}"
    );
    assert!(
        source.contains("win32_api=GetSystemMetrics/GetDesktopWindow ok"),
        "{source}"
    );
    assert!(source.contains("link_check=user32.lib ok"), "{source}");
    let rust_source =
        fs::read_to_string(validate_root.join("rust").join("src").join("main.rs")).unwrap();
    assert!(
        rust_source.contains("sample=spoon msvc validate rust"),
        "{rust_source}"
    );
    assert!(
        rust_source.contains("rust_runtime=Vec<i32>+fmt ok"),
        "{rust_source}"
    );
    assert!(
        rust_source.contains("native_helper=spoon-cl ok"),
        "{rust_source}"
    );
    assert!(
        rust_source.contains("linker_check=spoon-link ok"),
        "{rust_source}"
    );
    let rust_build_rs = fs::read_to_string(validate_root.join("rust").join("build.rs")).unwrap();
    assert!(
        rust_build_rs.contains("SPOON_VALIDATE_SPOON_CL"),
        "{rust_build_rs}"
    );
    assert!(rust_build_rs.contains("helper.c"), "{rust_build_rs}");
    let rust_cargo_config = fs::read_to_string(
        validate_root
            .join("rust")
            .join(".cargo")
            .join("config.toml"),
    )
    .unwrap();
    assert!(
        rust_cargo_config.contains("spoon-link.cmd"),
        "{rust_cargo_config}"
    );
    assert!(
        result
            .output
            .iter()
            .any(|line| line.contains("Using Cargo at")),
        "{:?}",
        result.output
    );
    assert!(
        result.output.iter().any(|line| {
            line.contains("Ran managed Rust validation sample successfully")
                || line.contains("Skipped managed Rust validation execution in test mode")
        }),
        "{:?}",
        result.output
    );
}

#[test]
fn managed_toolchain_flags_include_standard_sdk_layout_dirs_even_without_sentinels() {
    let _lock = env_lock();
    config::enable_test_mode();
    let tool_root = temp_dir("standard-include-layout");
    let toolchain_root = config::msvc_toolchain_root_from(&tool_root);
    seed_managed_state(&tool_root, "msvc-14.44.35207", "sdk-10.0.26100.15");
    fs::create_dir_all(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64"),
    )
    .unwrap();
    fs::create_dir_all(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("include"),
    )
    .unwrap();
    for segment in ["ucrt", "shared", "um", "winrt", "cppwinrt"] {
        fs::create_dir_all(
            toolchain_root
                .join("Windows Kits")
                .join("10")
                .join("Include")
                .join("10.0.26100.0")
                .join(segment),
        )
        .unwrap();
    }
    fs::create_dir_all(toolchain_root.join("lib").join("ucrt").join("x64")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("um").join("x64")).unwrap();
    fs::create_dir_all(toolchain_root.join("lib").join("vc").join("x64")).unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("cl.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("link.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("bin")
            .join("Hostx64")
            .join("x64")
            .join("lib.exe"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("ucrt")
            .join("x64")
            .join("ucrt.lib"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("um")
            .join("x64")
            .join("kernel32.lib"),
        b"",
    )
    .unwrap();
    fs::write(
        toolchain_root
            .join("lib")
            .join("vc")
            .join("x64")
            .join("libcmt.lib"),
        b"",
    )
    .unwrap();

    let flags = block_on(managed_toolchain_flags(&tool_root)).expect("managed flags");
    let includes: Vec<String> = flags
        .include_dirs
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    assert!(
        includes
            .iter()
            .any(|path| path.ends_with("\\VC\\Tools\\MSVC\\14.44.35207\\include"))
    );
    assert!(
        includes
            .iter()
            .any(|path| path.ends_with("\\Windows Kits\\10\\Include\\10.0.26100.0\\shared"))
    );
    assert!(
        includes
            .iter()
            .any(|path| path.ends_with("\\Windows Kits\\10\\Include\\10.0.26100.0\\winrt"))
    );
    assert!(
        includes
            .iter()
            .any(|path| path.ends_with("\\Windows Kits\\10\\Include\\10.0.26100.0\\cppwinrt"))
    );
}
