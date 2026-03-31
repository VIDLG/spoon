use crossterm::event::KeyCode;
#[path = "../common/mod.rs"]
mod common;

use common::constants::DEFAULT_WAIT;
use common::env_guard::PathGuard;
use common::fixtures::{select_tool_by_key, unique_temp_dir};
use common::tui::open_tools;
use spoon::config;
use spoon::tui::test_support::Harness;

#[test]
fn q_on_running_output_opens_cancel_confirmation() {
    let tool_root = unique_temp_dir("spoon-harness-scoop");
    let _ = std::fs::remove_dir_all(&tool_root);
    let mut app = Harness::new();
    let _path_guard = PathGuard::without_scoop_entries();
    config::save_global_config(&config::GlobalConfig {
        editor: String::new(),
        proxy: String::new(),
        root: tool_root.display().to_string(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();

    open_tools(&mut app);
    app.press(KeyCode::Char('i')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));

    let quit = app.press(KeyCode::Char('q')).unwrap();
    assert!(!quit);
    assert_eq!(app.modal_name(), Some("CancelRunningConfirm"));
}

#[test]
fn scoop_tool_install_uses_no_update_scoop_in_output() {
    let tool_root = unique_temp_dir("spoon-harness-scoop");
    let _ = std::fs::remove_dir_all(&tool_root);
    let _path_guard = PathGuard::empty();
    let mut app = Harness::with_install_root(Some(tool_root.clone()));
    config::save_global_config(&config::GlobalConfig {
        editor: String::new(),
        proxy: String::new(),
        root: tool_root.display().to_string(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();

    open_tools(&mut app);
    select_tool_by_key(&mut app, "uv");
    assert_eq!(app.selected_tool_key(), Some("uv"));

    app.press_without_settle(KeyCode::Char('i')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert!(
        app.wait_until(DEFAULT_WAIT, |h| {
            h.output_lines().unwrap_or_default().iter().any(|line| {
                line.contains("Planned Spoon package action (Scoop): install uv --no-update-scoop")
            })
        }),
        "lines: {}",
        app.output_lines().unwrap_or_default().join("\n")
    );
    let lines = app.output_lines().unwrap_or_default().join("\n");
    assert!(
        lines.contains("Planned Spoon package action (Scoop): install uv --no-update-scoop"),
        "lines: {lines}"
    );
}

#[test]
fn scoop_download_activity_indicator_renders_in_output_modal() {
    let mut app = Harness::new();
    app.set_output_modal_with_state_for_test(
        "Install glow",
        vec![
            "Planned Spoon package action (Scoop): install glow --no-update-scoop".to_string(),
            "Downloading https://github.com/charmbracelet/glow/releases/download/v2.1.1/glow_2.1.1_Windows_x86_64.zip (6.2 MB)...".to_string(),
        ],
        true,
        true,
        0,
    );

    let rendered = app.render_text(140, 40).join("\n");
    assert!(
        rendered.contains("[package archive in progress (6.2 MB)]"),
        "scoop download indicator should be rendered while a redirected download is active:\n{rendered}"
    );
}

#[test]
fn scoop_download_progress_line_renders_visual_bar_in_output_modal() {
    let mut app = Harness::new();
    app.set_output_modal_with_state_for_test(
        "Install claude-code",
        vec![
            "Planned Spoon package action (Scoop): install claude-code --no-update-scoop"
                .to_string(),
            "Download progress 34% (77.1 MB / 227.8 MB)".to_string(),
        ],
        true,
        true,
        0,
    );

    let rendered = app.render_text(140, 40).join("\n");
    assert!(
        rendered.contains("Progress ["),
        "visual progress bar should be rendered:\n{rendered}"
    );
    assert!(
        rendered.contains("34%"),
        "percent should be visible:\n{rendered}"
    );
    assert!(
        rendered.contains("(77.1 MB / 227.8 MB)"),
        "download summary should stay visible:\n{rendered}"
    );
}
