#[path = "../common/mod.rs"]
mod common;

use common::assertions::{assert_contains, assert_ok, assert_path_exists, assert_path_missing};
use common::cli::{create_test_home, detect_repo_root, run, run_in_home};
use common::constants::{CHUNK_STANDARD, PAYLOAD_CHUNK_DELAY_MS, PAYLOAD_STANDARD};
use common::fixtures::spawn_slow_payload_server;
use common::setup::{create_configured_home, write_test_config};
use spoon::config;
use spoon_core::RuntimeLayout;
use spoon_msvc::{MsvcRuntimeKind, MsvcOperationKind, MsvcValidationStatus, state::MsvcCanonicalState};
use std::process::Command;

fn read_canonical_msvc_state(
    tool_root: &std::path::Path,
) -> Option<MsvcCanonicalState> {
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    spoon_msvc::state::read_canonical_state(&layout)
}

#[test]
fn msvc_status_lists_managed_and_official_runtime_state() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let layout = RuntimeLayout::from_root(&tool_root);
    let managed_state_root = layout.msvc.managed.state_root.clone();
    let official_state_root = layout.msvc.official.state_root.clone();
    let shims_root = layout.shims.clone();
    std::fs::create_dir_all(&managed_state_root).unwrap();
    std::fs::create_dir_all(&official_state_root).unwrap();
    std::fs::create_dir_all(&shims_root).unwrap();
    std::fs::write(
        managed_state_root.join("runtime.json"),
        serde_json::json!({"runtime":"managed"}).to_string(),
    )
    .unwrap();
    std::fs::write(
        managed_state_root.join("installed.json"),
        serde_json::json!({
            "msvc":"msvc-14.44.17.14",
            "sdk":"sdk-10.0.26100.15"
        })
        .to_string(),
    )
    .unwrap();
    std::fs::write(
        official_state_root.join("runtime.json"),
        serde_json::json!({"runtime":"official"}).to_string(),
    )
    .unwrap();
    std::fs::write(
        official_state_root.join("installed.json"),
        serde_json::json!({
            "version":"14.44.35207",
            "sdk_version":"10.0.10240.0"
        })
        .to_string(),
    )
    .unwrap();
    std::fs::write(shims_root.join("spoon-cl.cmd"), "@echo off\r\n").unwrap();
    std::fs::write(shims_root.join("spoon-link.cmd"), "@echo off\r\n").unwrap();
    std::fs::write(shims_root.join("spoon-lib.cmd"), "@echo off\r\n").unwrap();

    let (ok, stdout, stderr) = run_in_home(&["msvc", "status"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "MSVC runtimes:");
    assert_contains(&stdout, "Managed:");
    assert_contains(&stdout, "  status: installed (14.44.17.14 + 10.0.26100.15)");
    assert_contains(&stdout, "Official:");
    assert_contains(&stdout, "  status: installed (14.44.35207 + 10.0.10240.0)");
    assert_contains(&stdout, "  Integration:");
    assert_contains(&stdout, "    Commands:");
    assert_contains(&stdout, "      wrappers: spoon-cl, spoon-link, spoon-lib");
    assert_contains(&stdout, "    Environment:");
    assert_contains(&stdout, "      user PATH entry: <root>/shims");
    assert_contains(&stdout, r"    System:");
    assert_contains(
        &stdout,
        r"      vswhere discovery: C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe",
    );
    assert_contains(
        &stdout,
        r"      shared Windows SDK root: C:\Program Files (x86)\Windows Kits\10",
    );
}

#[test]
fn msvc_install_without_tool_root_reports_prerequisite_block() {
    let (ok, stdout, stderr) = run(&["msvc", "install", "managed"]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "MSVC Toolchain requires a configured root.");
}

#[test]
fn msvc_install_official_bootstraps_instance_and_state() {
    let temp_home = create_test_home();
    std::fs::create_dir_all(&temp_home).unwrap();
    let tool_root = temp_home.join("tool-root");
    write_test_config(&temp_home, &tool_root, "");
    let bootstrapper = temp_home.join("fake-vs-buildtools.cmd");
    std::fs::write(
        &bootstrapper,
        "@echo off\r\n\
setlocal\r\n\
mkdir \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\" 2>nul\r\n\
mkdir \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\Windows Kits\\10\\Lib\\10.0.26100.0\\um\\x64\" 2>nul\r\n\
type nul > \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\cl.exe\"\r\n\
type nul > \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\link.exe\"\r\n\
exit /b 0\r\n",
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(
        &["msvc", "install", "official"],
        &temp_home,
        &[(
            "SPOON_TEST_MSVCOFFICIAL_BOOTSTRAPPER_SOURCE",
            &bootstrapper.display().to_string(),
        )],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Caching official MSVC bootstrapper");
    assert_contains(&stdout, "Installed official MSVC runtime into");
    assert_path_exists(&RuntimeLayout::from_root(&tool_root).msvc.official.instance_root);
    assert_path_exists(&RuntimeLayout::from_root(&tool_root).msvc.official.state_root.join("runtime.json"));
    let canonical = read_canonical_msvc_state(&tool_root).expect("canonical MSVC state");
    assert_eq!(
        canonical.runtime_kind,
        spoon_msvc::MsvcRuntimeKind::Official
    );
    assert!(canonical.installed);
    assert_eq!(canonical.version.as_deref(), Some("14.44.35207"));
    assert_eq!(canonical.sdk_version.as_deref(), Some("10.0.26100.0"));
    assert_eq!(
        canonical.last_operation,
        Some(spoon_msvc::MsvcOperationKind::Install)
    );
}

#[test]
fn msvc_uninstall_official_removes_instance_and_state() {
    let temp_home = create_test_home();
    let tool_root = temp_home.join("tool-root");
    write_test_config(&temp_home, &tool_root, "");
    let bootstrapper = temp_home.join("fake-vs-buildtools.cmd");
    std::fs::write(
        &bootstrapper,
        "@echo off\r\n\
setlocal\r\n\
if /I \"%1\"==\"uninstall\" (\r\n\
rmdir /S /Q \"%SPOON_OFFICIAL_INSTANCE_ROOT%\" 2>nul\r\n\
exit /b 0\r\n\
)\r\n\
mkdir \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\" 2>nul\r\n\
mkdir \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\Windows Kits\\10\\Lib\\10.0.26100.0\\um\\x64\" 2>nul\r\n\
type nul > \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\cl.exe\"\r\n\
type nul > \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\link.exe\"\r\n\
exit /b 0\r\n",
    )
    .unwrap();
    let layout = RuntimeLayout::from_root(&tool_root);
    let instance_root = layout.msvc.official.instance_root.clone();
    let state_root = layout.msvc.official.state_root.clone();
    let cache_root = layout.msvc.official.cache_root.clone();
    std::fs::create_dir_all(&instance_root).unwrap();
    std::fs::create_dir_all(&state_root).unwrap();
    std::fs::create_dir_all(&cache_root).unwrap();
    std::fs::write(state_root.join("runtime.json"), "{}").unwrap();

    let (ok, stdout, stderr) = run_in_home(
        &["msvc", "uninstall", "official"],
        &temp_home,
        &[(
            "SPOON_TEST_MSVCOFFICIAL_BOOTSTRAPPER_SOURCE",
            &bootstrapper.display().to_string(),
        )],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(
        &stdout,
        "Uninstalled official MSVC runtime through the Microsoft bootstrapper.",
    );
    assert_contains(&stdout, "Official MSVC cache is retained at");
    assert_path_missing(&instance_root);
    assert_path_missing(&state_root);
    assert_path_exists(&cache_root);
    let canonical = read_canonical_msvc_state(&tool_root).expect("canonical MSVC state");
    assert_eq!(
        canonical.runtime_kind,
        spoon_msvc::MsvcRuntimeKind::Official
    );
    assert!(!canonical.installed);
    assert_eq!(
        canonical.last_operation,
        Some(spoon_msvc::MsvcOperationKind::Uninstall)
    );
}

#[test]
fn msvc_update_official_reuses_cached_bootstrapper() {
    let temp_home = create_test_home();
    std::fs::create_dir_all(&temp_home).unwrap();
    let tool_root = temp_home.join("tool-root");
    write_test_config(&temp_home, &tool_root, "");
    let bootstrapper = temp_home.join("fake-vs-buildtools.cmd");
    std::fs::write(
        &bootstrapper,
        "@echo off\r\n\
setlocal\r\n\
mkdir \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\" 2>nul\r\n\
mkdir \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\Windows Kits\\10\\Lib\\10.0.26100.0\\um\\x64\" 2>nul\r\n\
type nul > \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\cl.exe\"\r\n\
type nul > \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\link.exe\"\r\n\
exit /b 0\r\n",
    )
    .unwrap();
    let bootstrapper_env = bootstrapper.display().to_string();

    let (ok, stdout, stderr) = run_in_home(
        &["msvc", "install", "official"],
        &temp_home,
        &[(
            "SPOON_TEST_MSVCOFFICIAL_BOOTSTRAPPER_SOURCE",
            &bootstrapper_env,
        )],
    );
    assert_ok(ok, &stdout, &stderr);

    let (ok, stdout, stderr) = run_in_home(
        &["msvc", "update", "official"],
        &temp_home,
        &[(
            "SPOON_TEST_MSVCOFFICIAL_BOOTSTRAPPER_SOURCE",
            &bootstrapper_env,
        )],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Reused cached official MSVC bootstrapper");
}

#[test]
fn msvc_install_official_passive_records_visible_mode() {
    let temp_home = create_test_home();
    std::fs::create_dir_all(&temp_home).unwrap();
    let tool_root = temp_home.join("tool-root");
    write_test_config(&temp_home, &tool_root, "");
    let bootstrapper = temp_home.join("fake-vs-buildtools.cmd");
    std::fs::write(
        &bootstrapper,
        "@echo off\r\n\
setlocal\r\n\
mkdir \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\" 2>nul\r\n\
mkdir \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\Windows Kits\\10\\Lib\\10.0.26100.0\\um\\x64\" 2>nul\r\n\
type nul > \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\cl.exe\"\r\n\
type nul > \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\link.exe\"\r\n\
exit /b 0\r\n",
    )
    .unwrap();
    let bootstrapper_env = bootstrapper.display().to_string();

    let (ok, stdout, stderr) = run_in_home(
        &["msvc", "install", "official", "--passive"],
        &temp_home,
        &[(
            "SPOON_TEST_MSVCOFFICIAL_BOOTSTRAPPER_SOURCE",
            &bootstrapper_env,
        )],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Official installer mode: passive");
    assert_contains(
        &stdout,
        "Showing official installer UI in passive mode; follow Microsoft setup for detailed progress.",
    );
    let metadata = std::fs::read_to_string(
        RuntimeLayout::from_root(&tool_root)
            .msvc
            .official
            .cache_root
            .join("commands")
            .join("last-command.json"),
    )
    .unwrap();
    assert_contains(&metadata, "--passive");
}

#[test]
fn msvc_install_official_quiet_flag_records_quiet_mode() {
    let temp_home = create_test_home();
    std::fs::create_dir_all(&temp_home).unwrap();
    let tool_root = temp_home.join("tool-root");
    write_test_config(&temp_home, &tool_root, "");
    let bootstrapper = temp_home.join("fake-vs-buildtools.cmd");
    std::fs::write(
        &bootstrapper,
        "@echo off\r\n\
setlocal\r\n\
mkdir \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\" 2>nul\r\n\
mkdir \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\Windows Kits\\10\\Lib\\10.0.26100.0\\um\\x64\" 2>nul\r\n\
type nul > \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\cl.exe\"\r\n\
type nul > \"%SPOON_OFFICIAL_INSTANCE_ROOT%\\VC\\Tools\\MSVC\\14.44.35207\\bin\\Hostx64\\x64\\link.exe\"\r\n\
exit /b 0\r\n",
    )
    .unwrap();
    let bootstrapper_env = bootstrapper.display().to_string();

    let (ok, stdout, stderr) = run_in_home(
        &["msvc", "install", "official", "--quiet"],
        &temp_home,
        &[(
            "SPOON_TEST_MSVCOFFICIAL_BOOTSTRAPPER_SOURCE",
            &bootstrapper_env,
        )],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Official installer mode: quiet");
    let metadata = std::fs::read_to_string(
        RuntimeLayout::from_root(&tool_root)
            .msvc
            .official
            .cache_root
            .join("commands")
            .join("last-command.json"),
    )
    .unwrap();
    assert_contains(&metadata, "--quiet");
}

#[test]
fn msvc_validate_without_runtime_uses_installed_runtime_set() {
    let temp_home = create_test_home();
    std::fs::create_dir_all(&temp_home).unwrap();
    let tool_root = temp_home.join("tool-root");
    write_test_config(&temp_home, &tool_root, "");
    let layout = RuntimeLayout::from_root(&tool_root);
    let official_root = layout.msvc.official.instance_root.clone();
    let official_state = layout.msvc.official.state_root.clone();
    let compiler_root = official_root
        .join("VC")
        .join("Tools")
        .join("MSVC")
        .join("14.44.35207")
        .join("bin")
        .join("Hostx64")
        .join("x64");
    std::fs::create_dir_all(&compiler_root).unwrap();
    std::fs::create_dir_all(&official_state).unwrap();
    std::fs::create_dir_all(
        official_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("include"),
    )
    .unwrap();
    for segment in ["ucrt", "shared", "um"] {
        std::fs::create_dir_all(
            official_root
                .join("Windows Kits")
                .join("10")
                .join("Include")
                .join("10.0.26100.0")
                .join(segment),
        )
        .unwrap();
    }
    std::fs::create_dir_all(
        official_root
            .join("Windows Kits")
            .join("10")
            .join("Lib")
            .join("10.0.26100.0")
            .join("ucrt")
            .join("x64"),
    )
    .unwrap();
    std::fs::create_dir_all(
        official_root
            .join("Windows Kits")
            .join("10")
            .join("Lib")
            .join("10.0.26100.0")
            .join("um")
            .join("x64"),
    )
    .unwrap();
    std::fs::create_dir_all(
        official_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("lib")
            .join("x64"),
    )
    .unwrap();
    std::fs::write(
        compiler_root.join("cl.cmd"),
        "@echo off\r\ncopy /Y \"%SystemRoot%\\System32\\whoami.exe\" hello.exe >nul\r\nexit /b 0\r\n",
    )
    .unwrap();
    std::fs::write(compiler_root.join("link.exe"), b"").unwrap();
    std::fs::write(
        official_root
            .join("Windows Kits")
            .join("10")
            .join("Include")
            .join("10.0.26100.0")
            .join("um")
            .join("Windows.h"),
        b"",
    )
    .unwrap();
    std::fs::write(
        official_root
            .join("Windows Kits")
            .join("10")
            .join("Include")
            .join("10.0.26100.0")
            .join("ucrt")
            .join("stdio.h"),
        b"",
    )
    .unwrap();
    std::fs::write(
        official_root
            .join("Windows Kits")
            .join("10")
            .join("Include")
            .join("10.0.26100.0")
            .join("shared")
            .join("winapifamily.h"),
        b"",
    )
    .unwrap();
    std::fs::write(
        official_root
            .join("Windows Kits")
            .join("10")
            .join("Lib")
            .join("10.0.26100.0")
            .join("ucrt")
            .join("x64")
            .join("ucrt.lib"),
        b"",
    )
    .unwrap();
    std::fs::write(
        official_root
            .join("Windows Kits")
            .join("10")
            .join("Lib")
            .join("10.0.26100.0")
            .join("um")
            .join("x64")
            .join("kernel32.lib"),
        b"",
    )
    .unwrap();
    std::fs::write(
        official_root
            .join("Windows Kits")
            .join("10")
            .join("Lib")
            .join("10.0.26100.0")
            .join("um")
            .join("x64")
            .join("user32.lib"),
        b"",
    )
    .unwrap();
    std::fs::write(
        official_root
            .join("VC")
            .join("Tools")
            .join("MSVC")
            .join("14.44.35207")
            .join("lib")
            .join("x64")
            .join("libcmt.lib"),
        b"",
    )
    .unwrap();
    std::fs::write(
        official_state.join("runtime.json"),
        r#"{"runtime":"official"}"#,
    )
    .unwrap();
    std::fs::write(
        official_state.join("installed.json"),
        r#"{"version":"14.44.35207","sdk_version":"10.0.26100.0"}"#,
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["msvc", "validate"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(
        &stdout,
        "Compiled official C++/Win32 validation sample successfully.",
    );
    assert_contains(&stdout, "Ran official validation sample successfully.");
    let canonical = read_canonical_msvc_state(&tool_root).expect("canonical MSVC state");
    assert_eq!(
        canonical.runtime_kind,
        spoon_msvc::MsvcRuntimeKind::Official
    );
    assert!(canonical.installed);
    assert_eq!(
        canonical.last_operation,
        Some(spoon_msvc::MsvcOperationKind::Validate)
    );
    assert_eq!(
        canonical.validation_status,
        Some(spoon_msvc::MsvcValidationStatus::Valid)
    );
}

#[test]
fn msvc_update_is_noop_when_cached_target_matches_installed_state() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let layout = RuntimeLayout::from_root(&tool_root);
    let manifest_root = layout.msvc.managed.manifest_root.join("vs");
    let state_root = layout.msvc.managed.state_root.clone();
    let toolchain_root = layout.msvc.managed.toolchain_root.clone();
    std::fs::create_dir_all(&manifest_root).unwrap();
    std::fs::create_dir_all(&state_root).unwrap();
    std::fs::create_dir_all(&toolchain_root).unwrap();
    std::fs::write(
        state_root.join("runtime.json"),
        serde_json::json!({
            "toolchain_root": toolchain_root,
            "wrappers_root": layout.shims.clone(),
            "runtime": "managed"
        })
        .to_string(),
    )
    .unwrap();
    std::fs::write(
        state_root.join("installed.json"),
        serde_json::json!({
            "msvc": "msvc-14.44.17.14",
            "sdk": "sdk-10.0.22621.7"
        })
        .to_string(),
    )
    .unwrap();
    std::fs::write(
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

    let (ok, stdout, stderr) = run_in_home(&["msvc", "update", "managed"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Managed MSVC toolchain is already up to date:");
    assert_eq!(
        stdout
            .matches("Managed MSVC toolchain is already up to date:")
            .count(),
        1,
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(!stdout.contains("Installed latest MSVC toolchain target directly with spoon:"));
}

#[test]
fn msvc_install_streams_download_progress_in_cli_output() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    std::fs::create_dir_all(
        tool_root
            .join("msvc")
            .join("managed")
            .join("cache")
            .join("manifest")
            .join("vs"),
    )
    .unwrap();

    let payload_bytes = vec![b'x'; PAYLOAD_STANDARD];
    let payload_sha = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&payload_bytes);
        format!("{:x}", hasher.finalize())
    };
    let payload_url =
        spawn_slow_payload_server(payload_bytes, CHUNK_STANDARD, PAYLOAD_CHUNK_DELAY_MS, true);
    std::fs::write(
        tool_root
            .join("msvc")
            .join("managed")
            .join("cache")
            .join("manifest")
            .join("vs")
            .join("latest.json"),
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
                            "url": payload_url,
                            "fileName": "Installers\\Windows SDK for Windows Store Apps Tools-x86_en-us.msi",
                            "sha256": payload_sha
                        }
                    ]
                }
            ]
        })
        .to_string(),
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(
        &["msvc", "install", "managed"],
        &temp_home,
        &[("HTTP_PROXY", ""), ("HTTPS_PROXY", ""), ("ALL_PROXY", "")],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Download progress ");
}

#[test]
fn msvc_install_reports_downloaded_bytes_when_total_is_unknown() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    std::fs::create_dir_all(
        tool_root
            .join("msvc")
            .join("managed")
            .join("cache")
            .join("manifest")
            .join("vs"),
    )
    .unwrap();

    let payload_bytes = vec![b'x'; PAYLOAD_STANDARD];
    let payload_sha = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&payload_bytes);
        format!("{:x}", hasher.finalize())
    };
    let payload_url =
        spawn_slow_payload_server(payload_bytes, CHUNK_STANDARD, PAYLOAD_CHUNK_DELAY_MS, false);
    std::fs::write(
        tool_root
            .join("msvc")
            .join("managed")
            .join("cache")
            .join("manifest")
            .join("vs")
            .join("latest.json"),
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
                            "url": payload_url,
                            "fileName": "Installers\\Windows SDK for Windows Store Apps Tools-x86_en-us.msi",
                            "sha256": payload_sha
                        }
                    ]
                }
            ]
        })
        .to_string(),
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(
        &["msvc", "install", "managed"],
        &temp_home,
        &[("HTTP_PROXY", ""), ("HTTPS_PROXY", ""), ("ALL_PROXY", "")],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "MB downloaded");
}

#[test]
#[ignore = "real MSVC validation flow; requires a configured managed MSVC toolchain"]
fn capability_validate_msvc_real_managed_toolchain() {
    let global = config::load_global_config();
    assert!(
        !global.root.trim().is_empty(),
        "real MSVC validate test requires a configured root"
    );

    let output = Command::new(env!("CARGO_BIN_EXE_spoon"))
        .args(["msvc", "validate", "managed"])
        .current_dir(detect_repo_root())
        .output()
        .expect("run spoon msvc validate");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\n\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Compiled managed C++/Win32 validation sample successfully"),
        "stdout:\n{stdout}\n\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Ran managed validation sample successfully"),
        "stdout:\n{stdout}\n\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Kept validation workspace"),
        "stdout:\n{stdout}\n\nstderr:\n{stderr}"
    );
}
