#![allow(dead_code)]

use crossterm::event::KeyCode;
use spoon::service::scoop;
use spoon::tui::test_support::Harness;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn unique_temp_dir(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

pub fn seed_msvc_manifest(tool_root: &str, msvc: &str, sdk: &str) {
    let manifest_root = PathBuf::from(tool_root)
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
                    "id": format!("Microsoft.VC.{}.Tools.HostX64.TargetX64.base", msvc.strip_prefix("msvc-").unwrap_or(msvc)),
                    "version": msvc.strip_prefix("msvc-").unwrap_or(msvc),
                    "language": "neutral",
                    "payloads": []
                },
                {
                    "id": "WindowsSdkPackageB",
                    "version": sdk.strip_prefix("sdk-").unwrap_or(sdk),
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
}

pub fn spawn_slow_payload_server(
    body: Vec<u8>,
    chunk_size: usize,
    delay_ms: u64,
    include_content_length: bool,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind slow payload server");
    let addr = listener.local_addr().expect("slow payload server addr");
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept slow payload client");
        let mut request_buf = [0_u8; 1024];
        let _ = stream.read(&mut request_buf);
        let header = if include_content_length {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n",
                body.len()
            )
        } else {
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n"
                .to_string()
        };
        stream
            .write_all(header.as_bytes())
            .expect("write slow payload headers");
        for chunk in body.chunks(chunk_size.max(1)) {
            if stream.write_all(chunk).is_err() {
                break;
            }
            if stream.flush().is_err() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
    });
    format!("http://{addr}/sdk-tools.msi")
}

pub fn select_tool_by_key(app: &mut Harness, key: &str) {
    for _ in 0..32 {
        app.press(KeyCode::Up).unwrap();
    }
    for _ in 0..32 {
        if app.selected_tool_key() == Some(key) {
            return;
        }
        app.press(KeyCode::Down).unwrap();
    }
    panic!(
        "failed to select tool `{key}`, current selection: {:?}",
        app.selected_tool_key()
    );
}

pub struct RealBackendGuard;

impl RealBackendGuard {
    pub fn enable() -> Self {
        scoop::set_real_backend_test_mode(true);
        Self
    }
}

impl Drop for RealBackendGuard {
    fn drop(&mut self) {
        scoop::set_real_backend_test_mode(false);
    }
}

pub fn wait_for_completed_output(app: &mut Harness, timeout: Duration) {
    let ready = app.wait_until(timeout, |app| {
        app.modal_name() == Some("Output") && app.output_running() == Some(false)
    });
    assert!(
        ready,
        "timed out waiting for completed output: {:?}",
        app.output_lines()
    );
}
