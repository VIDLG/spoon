use std::fs;
use std::path::Path;

use crossterm::event::KeyCode;
use spoon::config;
use spoon::tui::test_support::Harness;

#[path = "../common/mod.rs"]
mod common;

use common::constants::{NETWORK_TIMEOUT, SCOOP_INSTALL_TIMEOUT};
use common::fixtures::{select_tool_by_key, unique_temp_dir, wait_for_completed_output};
use common::tui::open_tools;
use common::windows_env::UserEnvGuard;

fn read_real_proxy() -> String {
    let Some(user_profile) = std::env::var_os("USERPROFILE") else {
        return String::new();
    };
    let path = std::path::PathBuf::from(user_profile)
        .join(".spoon")
        .join("config.toml");
    let Ok(content) = fs::read_to_string(path) else {
        return String::new();
    };
    content
        .lines()
        .find_map(|line| {
            let line = line.trim();
            let value = line.strip_prefix("proxy = ")?;
            Some(value.trim_matches('"').to_string())
        })
        .unwrap_or_default()
}

#[test]
#[ignore = "real TUI Scoop flow; uses network and installs into a temp tool_root"]
fn harness_can_drive_real_scoop_install_update_uninstall_flow() {
    let _user_env = UserEnvGuard::capture();
    let proxy = read_real_proxy();
    let tool_root = unique_temp_dir("spoon-harness-scoop");
    let _ = fs::remove_dir_all(&tool_root);

    let mut app = Harness::with_install_root(Some(tool_root.clone()));
    app.enable_real_scoop_backend();

    config::save_global_config(&config::GlobalConfig {
        editor: String::new(),
        proxy,
        root: tool_root.display().to_string(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();

    open_tools(&mut app);
    assert_eq!(app.selected_tool_key(), Some("scoop"));

    app.press_without_settle(KeyCode::Char('i')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_title(), Some("Install tools".to_string()));
    wait_for_completed_output(&mut app, SCOOP_INSTALL_TIMEOUT);
    app.press(KeyCode::Enter).unwrap();

    select_tool_by_key(&mut app, "jq");
    app.press_without_settle(KeyCode::Char('i')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    wait_for_completed_output(&mut app, NETWORK_TIMEOUT);
    app.press(KeyCode::Enter).unwrap();

    select_tool_by_key(&mut app, "jq");
    app.press_without_settle(KeyCode::Char('u')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    wait_for_completed_output(&mut app, NETWORK_TIMEOUT);
    app.press(KeyCode::Enter).unwrap();

    select_tool_by_key(&mut app, "jq");
    app.press_without_settle(KeyCode::Char('x')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    wait_for_completed_output(&mut app, NETWORK_TIMEOUT);
    app.press(KeyCode::Enter).unwrap();

    select_tool_by_key(&mut app, "scoop");
    app.press_without_settle(KeyCode::Char('x')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    wait_for_completed_output(&mut app, NETWORK_TIMEOUT);
    app.press(KeyCode::Enter).unwrap();

    assert!(
        !config::scoop_root_from(Path::new(&tool_root)).exists(),
        "expected Scoop root to be removed: {}",
        config::scoop_root_from(Path::new(&tool_root)).display()
    );
}
