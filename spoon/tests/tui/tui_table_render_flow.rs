use crossterm::event::KeyCode;
#[path = "../common/mod.rs"]
mod common;

use common::constants::DEFAULT_WAIT;
use common::fixtures::{select_tool_by_key, unique_temp_dir};
use common::tui::open_tools;
use spoon::config;
use spoon::status::ToolStatus;
use spoon::tool;
use spoon::tui::test_support::Harness;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn tools_table_shows_version_and_latest_columns() {
    let mut app = Harness::new();

    open_tools(&mut app);
    let rendered = app.render_text(140, 40).join("\n");

    assert!(rendered.contains("TAG"));
    assert!(rendered.contains("VER"));
    assert!(rendered.contains("LATEST"));
    assert!(rendered.contains("SIZE"));
}

#[test]
fn tools_table_shows_editor_tag_for_editor_tools() {
    let mut app = Harness::new();

    open_tools(&mut app);
    let rendered = app.render_text(140, 40).join("\n");

    assert!(rendered.contains("EDITOR"), "{rendered}");
    assert!(rendered.contains("STATUS"), "{rendered}");
}

#[test]
fn tools_table_lists_cmake_and_ninja_helper_tools() {
    let mut app = Harness::new();

    open_tools(&mut app);
    let rendered = app.render_text(170, 40).join("\n");

    assert!(rendered.contains("CMake"), "{rendered}");
    assert!(rendered.contains("Ninja"), "{rendered}");
    assert!(rendered.contains("HELPER"), "{rendered}");
}

#[test]
fn tools_table_shows_managed_and_external_ownership() {
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

    let scoop_shims = config::shims_root_from(&tool_root);
    std::fs::create_dir_all(&scoop_shims).unwrap();
    std::fs::write(scoop_shims.join("scoop.cmd"), "@echo off\r\n").unwrap();
    std::fs::write(scoop_shims.join("jq.exe"), "").unwrap();

    let external_dir = std::env::temp_dir().join(format!(
        "spoon-tools-external-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&external_dir);
    std::fs::create_dir_all(&external_dir).unwrap();
    std::fs::write(external_dir.join("fd.exe"), "").unwrap();

    let original_path = std::env::var("PATH").unwrap_or_default();
    unsafe {
        std::env::set_var(
            "PATH",
            format!("{};{}", external_dir.display(), original_path),
        );
    }

    open_tools(&mut app);
    app.press(KeyCode::Char('r')).unwrap();
    let rendered = app.render_text(160, 40).join("\n");

    unsafe {
        std::env::set_var("PATH", original_path);
    }
    let _ = std::fs::remove_dir_all(external_dir);

    assert!(rendered.contains("managed"), "{rendered}");
    assert!(rendered.contains("external"), "{rendered}");
}

#[test]
fn tools_table_hides_latest_when_same_as_current() {
    let mut app = Harness::new();
    open_tools(&mut app);
    let jq_tool = tool::all_tools()
        .into_iter()
        .find(|tool| tool.key == "jq")
        .expect("jq tool");
    app.set_tool_statuses_for_test(vec![ToolStatus {
        tool: jq_tool,
        path: Some(std::path::PathBuf::from("C:/fake/jq.exe")),
        version: Some("1.8.1".to_string()),
        latest_version: Some("1.8.1".to_string()),
        installed_size_bytes: None,
        update_available: false,
        expected_dir: None,
        available: true,
        broken: false,
    }]);
    assert!(
        app.wait_until(DEFAULT_WAIT, |h| h.tool_version("jq").flatten().as_deref() == Some("1.8.1")),
        "jq version never became visible"
    );
    select_tool_by_key(&mut app, "jq");
    let rendered = app.render_text(170, 40).join("\n");

    assert!(rendered.contains("1.8.1"), "{rendered}");
    assert!(
        !rendered.contains("1.8.1 1.8.1"),
        "latest column should not repeat current version:\n{rendered}"
    );
}

#[test]
fn tools_table_shows_size_for_managed_tool() {
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

    let jq_root = config::scoop_root_from(&tool_root)
        .join("apps")
        .join("jq")
        .join("current");
    let jq_shims = config::shims_root_from(&tool_root);
    std::fs::create_dir_all(&jq_root).unwrap();
    std::fs::create_dir_all(&jq_shims).unwrap();
    std::fs::write(jq_root.join("jq.exe"), vec![0_u8; 2048]).unwrap();
    std::fs::write(jq_shims.join("jq.exe"), vec![0_u8; 16]).unwrap();

    open_tools(&mut app);
    app.press(KeyCode::Char('r')).unwrap();
    assert!(
        app.wait_until(DEFAULT_WAIT, |h| {
            h.tool_installed_size_bytes("jq") == Some(Some(2048))
        }),
        "managed size never became visible"
    );
    select_tool_by_key(&mut app, "jq");
    let rendered = app.render_text(170, 40).join("\n");

    assert!(rendered.contains("SIZE"), "{rendered}");
    assert!(rendered.contains("2.0K"), "{rendered}");
}

#[test]
fn top_tabs_render_without_frame_and_keep_separator() {
    let mut app = Harness::new();

    let rendered = app.render_text(140, 40).join("\n");
    assert!(rendered.contains("1 Configure | 2 Tools"), "{rendered}");
    assert!(rendered.contains("────"), "{rendered}");
    assert!(!rendered.contains("┌Spoon"), "{rendered}");
}

#[test]
fn tools_table_sorts_by_tag_then_name() {
    let mut app = Harness::new();

    open_tools(&mut app);
    assert_eq!(app.selected_tool_key(), Some("msvc"));

    app.press(KeyCode::Down).unwrap();
    assert_eq!(app.selected_tool_key(), Some("claude"));

    app.press(KeyCode::Down).unwrap();
    assert_eq!(app.selected_tool_key(), Some("codex"));
}

#[test]
fn tools_page_keeps_msvc_and_core_tools_first() {
    let mut app = Harness::new();

    open_tools(&mut app);
    assert_eq!(app.selected_tool_key(), Some("msvc"));

    app.press(KeyCode::Down).unwrap();
    assert_eq!(app.selected_tool_key(), Some("claude"));
}
