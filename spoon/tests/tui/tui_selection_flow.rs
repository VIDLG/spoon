use crossterm::event::KeyCode;
#[path = "../common/mod.rs"]
mod common;

use common::fixtures::{select_tool_by_key, unique_temp_dir};
use common::tui::open_tools;
use spoon::config;
use spoon::tui::test_support::Harness;

#[test]
fn tools_space_selects_and_c_clears() {
    let mut app = Harness::new();

    open_tools(&mut app);
    assert_eq!(app.selected_tool_marked(), Some(false));

    app.press(KeyCode::Char(' ')).unwrap();
    assert_eq!(app.selected_tool_marked(), Some(true));

    app.press(KeyCode::Char('c')).unwrap();
    assert_eq!(app.selected_tool_marked(), Some(false));
}

#[test]
fn tools_a_toggles_select_all() {
    let mut app = Harness::new();

    open_tools(&mut app);
    assert_eq!(app.selected_tool_marked(), Some(false));

    app.press(KeyCode::Char('a')).unwrap();
    assert_eq!(app.selected_tool_marked(), Some(true));

    app.press(KeyCode::Char('a')).unwrap();
    assert_eq!(app.selected_tool_marked(), Some(false));
}

#[test]
fn tools_m_toggles_missing_selection() {
    let mut app = Harness::new();

    open_tools(&mut app);

    app.press(KeyCode::Char('m')).unwrap();
    let first_count = app.selected_tool_count().unwrap_or(0);
    assert!(
        first_count > 0,
        "missing selection should select at least one tool"
    );

    app.press(KeyCode::Char('m')).unwrap();
    assert_eq!(app.selected_tool_count(), Some(0));
}

#[test]
fn tools_p_toggles_installed_selection() {
    let mut app = Harness::new();

    open_tools(&mut app);

    app.press(KeyCode::Char('p')).unwrap();
    let first_count = app.selected_tool_count().unwrap_or(0);
    assert!(
        first_count > 0,
        "installed selection should select at least one tool"
    );

    app.press(KeyCode::Char('p')).unwrap();
    assert_eq!(app.selected_tool_count(), Some(0));
}

#[test]
fn installed_managed_tool_can_start_uninstall_from_tools_page() {
    let tool_root = unique_temp_dir("spoon-harness-scoop");
    let _ = std::fs::remove_dir_all(&tool_root);
    let mut app = Harness::with_install_root(Some(tool_root.clone()));
    config::save_global_config(&config::GlobalConfig {
        editor: String::new(),
        proxy: String::new(),
        root: tool_root.display().to_string(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();

    let shims = spoon_core::RuntimeLayout::from_root(&tool_root).shims;
    std::fs::create_dir_all(&shims).unwrap();
    std::fs::write(shims.join("scoop.cmd"), "@echo off\r\n").unwrap();
    std::fs::write(shims.join("jq.exe"), "").unwrap();

    open_tools(&mut app);
    app.press(KeyCode::Char('r')).unwrap();
    select_tool_by_key(&mut app, "jq");
    assert_eq!(app.selected_tool_detected(), Some(true));

    app.press_without_settle(KeyCode::Char('x')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_title(), Some("Uninstall tools".to_string()));
}
