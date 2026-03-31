use crossterm::event::KeyCode;
#[path = "../common/mod.rs"]
mod common;

use common::constants::{
    CHUNK_SMALL, CHUNK_STANDARD, DEFAULT_WAIT, EXTENDED_WAIT, PAYLOAD_CHUNK_DELAY_MS,
    PAYLOAD_LARGE, PAYLOAD_STANDARD,
};
use common::env_guard::EnvVarGuard;
use common::fixtures::{select_tool_by_key, spawn_slow_payload_server, unique_temp_dir};
use common::tui::open_tools;
use spoon::config;
use spoon::tui::test_support::Harness;

fn setup_msvc_download_test(
    payload_size: usize,
    chunk_size: usize,
    include_content_length: bool,
) -> (Harness, EnvVarGuard, EnvVarGuard, EnvVarGuard) {
    let tool_root = unique_temp_dir("spoon-msvc-shell");
    let app = Harness::with_install_root(Some(tool_root.clone()));
    let http_proxy_guard = EnvVarGuard::clear("HTTP_PROXY");
    let https_proxy_guard = EnvVarGuard::clear("HTTPS_PROXY");
    let all_proxy_guard = EnvVarGuard::clear("ALL_PROXY");
    config::save_global_config(&config::GlobalConfig {
        editor: String::new(),
        proxy: String::new(),
        root: tool_root.display().to_string(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();

    let payload_bytes = vec![b'x'; payload_size];
    let payload_sha = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&payload_bytes);
        format!("{:x}", hasher.finalize())
    };
    let payload_url = spawn_slow_payload_server(
        payload_bytes,
        chunk_size,
        PAYLOAD_CHUNK_DELAY_MS,
        include_content_length,
    );

    let manifest_root = tool_root
        .join("msvc")
        .join("managed")
        .join("cache")
        .join("manifest")
        .join("vs");
    std::fs::create_dir_all(&manifest_root).unwrap();
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

    (app, http_proxy_guard, https_proxy_guard, all_proxy_guard)
}

#[test]
fn tools_install_msvc_streams_download_progress_before_completion() {
    let (mut app, _h, _s, _a) = setup_msvc_download_test(PAYLOAD_STANDARD, CHUNK_STANDARD, true);

    open_tools(&mut app);
    select_tool_by_key(&mut app, "msvc");
    assert_eq!(app.selected_tool_key(), Some("msvc"));

    app.press_without_settle(KeyCode::Char('i')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_running(), Some(true));

    assert!(
        app.wait_until(DEFAULT_WAIT, |h| {
            h.output_running() == Some(true)
                && h.output_lines()
                    .unwrap_or_default()
                    .iter()
                    .any(|line| line.starts_with("Download progress "))
        }),
        "MSVC install never surfaced live download progress before completion: {:?}",
        app.output_lines().unwrap_or_default()
    );

    let rendered = app.render_text(160, 44).join("\n");
    assert!(rendered.contains("Progress ["), "{rendered}");
}

#[test]
fn tools_install_msvc_shows_downloaded_bytes_when_total_is_unknown() {
    let (mut app, _h, _s, _a) = setup_msvc_download_test(PAYLOAD_STANDARD, CHUNK_STANDARD, false);

    open_tools(&mut app);
    select_tool_by_key(&mut app, "msvc");
    assert_eq!(app.selected_tool_key(), Some("msvc"));

    app.press_without_settle(KeyCode::Char('i')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_running(), Some(true));

    assert!(
        app.wait_until(DEFAULT_WAIT, |h| {
            h.output_running() == Some(true)
                && h.output_lines()
                    .unwrap_or_default()
                    .iter()
                    .any(|line| line.contains("MB downloaded"))
        }),
        "MSVC install never surfaced byte activity without total size: {:?}",
        app.output_lines().unwrap_or_default()
    );

    let rendered = app.render_text(160, 44).join("\n");
    assert!(rendered.contains("Progress ["), "{rendered}");
    assert!(rendered.contains("MB downloaded"), "{rendered}");
}

#[test]
fn tools_install_msvc_keeps_latest_progress_visible_in_small_output_viewport() {
    let (mut app, _h, _s, _a) = setup_msvc_download_test(PAYLOAD_LARGE, CHUNK_SMALL, true);

    open_tools(&mut app);
    select_tool_by_key(&mut app, "msvc");
    assert_eq!(app.selected_tool_key(), Some("msvc"));

    app.press_without_settle(KeyCode::Char('i')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_running(), Some(true));

    assert!(
        app.wait_until(DEFAULT_WAIT, |h| {
            h.output_running() == Some(true)
                && h.output_lines()
                    .unwrap_or_default()
                    .iter()
                    .any(|line| line.starts_with("Download progress "))
        }),
        "MSVC install should keep progress visible even in a cramped output viewport: {:?}",
        app.output_lines().unwrap_or_default()
    );

    let rendered = app.render_text(100, 22).join("\n");
    assert!(rendered.contains("Progress ["), "{rendered}");
    assert!(rendered.contains("%"), "{rendered}");
    assert!(rendered.contains("MB /"), "{rendered}");
}

#[test]
fn tools_install_msvc_can_be_cancelled_from_running_output() {
    let (mut app, _h, _s, _a) = setup_msvc_download_test(PAYLOAD_STANDARD, CHUNK_STANDARD, true);

    open_tools(&mut app);
    select_tool_by_key(&mut app, "msvc");
    app.press_without_settle(KeyCode::Char('i')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_running(), Some(true));

    assert!(
        app.wait_until(DEFAULT_WAIT, |h| {
            h.output_running() == Some(true)
                && h.output_lines()
                    .unwrap_or_default()
                    .iter()
                    .any(|line| line.starts_with("Download progress "))
        }),
        "MSVC install never started live download before cancellation"
    );

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), Some("CancelRunningConfirm"));

    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert!(
        app.wait_until(EXTENDED_WAIT, |h| {
            h.output_running() == Some(false)
                && h.output_lines()
                    .unwrap_or_default()
                    .iter()
                    .any(|line| line.contains("Cancelled by user."))
        }),
        "running MSVC install was not cancelled | modal={:?} running={:?} lines={:?}",
        app.modal_name(),
        app.output_running(),
        app.output_lines()
    );
}
