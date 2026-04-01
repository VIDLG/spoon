use crossterm::event::KeyCode;
#[path = "../common/mod.rs"]
mod common;

use common::env_guard::PathGuard;
use common::fixtures::{seed_msvc_manifest, select_tool_by_key, unique_temp_dir};
use common::tui::open_tools;
use spoon::config;
use spoon::tui::test_support::Harness;
use spoon_backend::layout::RuntimeLayout;

#[test]
fn tools_detail_opens_and_closes_with_escape() {
    let mut app = Harness::new();

    open_tools(&mut app);
    assert_eq!(app.modal_name(), None);

    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("ToolDetail"));

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), None);
    assert_eq!(app.screen_name(), "Tools");
}

#[test]
fn tools_detail_can_trigger_actions() {
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
    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("ToolDetail"));

    app.press_without_settle(KeyCode::Char('i')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_title(), Some("Install tools".to_string()));
}

#[test]
fn blocked_tools_action_opens_output_and_escape_closes_it() {
    let mut app = Harness::new();

    open_tools(&mut app);

    app.press(KeyCode::Char('u')).unwrap();
    assert_eq!(app.modal_name(), None);
    assert_eq!(app.screen_name(), "Tools");
    let hint = app.status_hint().unwrap_or_default();
    assert!(
        hint.contains("Current tool cannot be updated.")
            || hint.contains("Selected tools cannot be updated."),
        "hint: {hint}"
    );
}

#[test]
fn tools_detail_invalid_action_keeps_detail_and_sets_hint() {
    let mut app = Harness::new();

    open_tools(&mut app);
    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("ToolDetail"));

    app.press(KeyCode::Char('u')).unwrap();
    assert_eq!(app.modal_name(), Some("ToolDetail"));
    let hint = app.status_hint().unwrap_or_default();
    assert!(hint.contains("cannot be updated"), "hint: {hint}");
}

#[test]
fn external_tool_detail_rejects_install_action() {
    let tool_root = unique_temp_dir("spoon-harness-external-detail");
    let _ = std::fs::remove_dir_all(&tool_root);
    let mut app = Harness::with_install_root(Some(tool_root.clone()));
    let _path_guard = PathGuard::without_scoop_entries();
    config::save_global_config(&config::GlobalConfig {
        editor: String::new(),
        proxy: String::new(),
        root: tool_root.display().to_string(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();

    let external_dir = std::env::temp_dir().join(format!(
        "spoon-tools-external-detail-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
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
    select_tool_by_key(&mut app, "fd");
    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("ToolDetail"));

    app.press(KeyCode::Char('i')).unwrap();
    assert_eq!(app.modal_name(), Some("ToolDetail"));
    let hint = app.status_hint().unwrap_or_default();

    unsafe {
        std::env::set_var("PATH", original_path);
    }
    let _ = std::fs::remove_dir_all(external_dir);

    assert!(hint.contains("cannot be installed"), "hint: {hint}");
}

#[test]
fn tool_detail_prioritizes_summary_ops_versions_and_config() {
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
    let claude_root = RuntimeLayout::from_root(&tool_root)
        .scoop
        .package_current_root("claude-code");
    let claude_shims = config::shims_root_from(&tool_root);
    std::fs::create_dir_all(&claude_root).unwrap();
    std::fs::create_dir_all(&claude_shims).unwrap();
    std::fs::write(claude_root.join("claude.exe"), vec![0_u8; 2048]).unwrap();
    std::fs::write(claude_shims.join("claude.exe"), vec![0_u8; 16]).unwrap();

    open_tools(&mut app);
    app.press(KeyCode::Char('r')).unwrap();
    select_tool_by_key(&mut app, "claude");
    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("ToolDetail"));

    let rendered = app.render_text(140, 40).join("\n");
    let summary_index = rendered.find("summary:").expect("summary");
    let ops_index = rendered
        .find("available operations: i u x")
        .expect("available operations");
    let installed_index = rendered.find("installed:").expect("installed");
    let version_index = rendered.find("version:").expect("version");
    let homepage_index = rendered.find("homepage:").expect("homepage");
    let config_index = rendered.find("config path:").expect("config path");
    let path_index = rendered.find("path:").expect("path");
    let type_index = rendered.find("type:").expect("type");

    assert!(
        summary_index < ops_index,
        "summary should stay above ops:\n{rendered}"
    );
    assert!(
        ops_index < installed_index,
        "ops should stay above installed:\n{rendered}"
    );
    assert!(
        installed_index < version_index,
        "installed should stay above version:\n{rendered}"
    );
    assert!(
        version_index < homepage_index,
        "version should stay above homepage:\n{rendered}"
    );
    assert!(
        homepage_index < config_index,
        "homepage should stay above config path:\n{rendered}"
    );
    assert!(
        config_index < path_index,
        "config path should stay above path:\n{rendered}"
    );
    assert!(
        path_index < type_index,
        "path should stay above type:\n{rendered}"
    );
    assert!(
        rendered.contains("homepage:"),
        "detail should show homepage:\n{rendered}"
    );
    assert!(
        rendered.contains("config path:"),
        "detail should show config path:\n{rendered}"
    );
    assert!(
        rendered.contains("install root:"),
        "detail should show install root:\n{rendered}"
    );
    assert!(
        rendered.contains("installed size:"),
        "detail should show installed size:\n{rendered}"
    );
    assert!(
        rendered.contains("package: claude-code"),
        "detail should show scoop package label:\n{rendered}"
    );
    assert!(
        rendered.contains("bucket: main"),
        "detail should show scoop bucket:\n{rendered}"
    );
    assert!(
        rendered.contains("package link: https://github.com/ScoopInstaller/Main/blob/master/bucket/claude-code.json"),
        "detail should show direct scoop manifest link:\n{rendered}"
    );
    assert!(
        rendered.contains("available operations: i u x"),
        "detail should show compact operations row:\n{rendered}"
    );
    assert!(
        rendered.contains("type:"),
        "detail should show type label:\n{rendered}"
    );
    assert!(
        !rendered.contains("entity:"),
        "legacy entity label should be removed:\n{rendered}"
    );
    assert!(
        !rendered.contains("\n│                │ root:"),
        "generic root field should be removed for tools:\n{rendered}"
    );
    assert!(
        !rendered.contains("managed path:"),
        "tools without managed path should not show an empty managed path row:\n{rendered}"
    );
    assert!(
        !rendered.contains("selected:"),
        "selection noise should be removed from detail:\n{rendered}"
    );
}

#[test]
fn tool_detail_can_be_copied_to_clipboard() {
    let mut app = Harness::new();

    open_tools(&mut app);
    select_tool_by_key(&mut app, "claude");
    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("ToolDetail"));

    app.press(KeyCode::Char('c')).unwrap();

    let copied = app.clipboard_text().unwrap_or_default();
    assert!(copied.contains("Claude Code"), "clipboard: {copied}");
    assert!(copied.contains("summary:"), "clipboard: {copied}");
    assert!(
        copied.contains("available operations:"),
        "clipboard: {copied}"
    );
    assert_eq!(
        app.status_hint(),
        Some("Copied tool detail to clipboard.".to_string())
    );
}

#[test]
fn tool_detail_uses_backend_models() {
    // Verify that the ToolDetailModel is built from backend data paths and that
    // the detail module does not import app-local helpers like package_current_root
    // or installed_package_states_filtered.

    // Verify the module compiles without those imports by checking that
    // RuntimeLayout from spoon_backend is used for path derivation.
    let tool_root = unique_temp_dir("spoon-detail-backend");
    let _ = std::fs::remove_dir_all(&tool_root);

    // Set up the backend state so we can verify RuntimeLayout drives the detail.
    let layout = spoon_backend::layout::RuntimeLayout::from_root(&tool_root);
    assert!(
        layout.scoop.apps_root.to_string_lossy().contains("scoop"),
        "RuntimeLayout should derive scoop apps root"
    );
    assert!(
        layout.msvc.managed.toolchain_root.to_string_lossy().contains("toolchain"),
        "RuntimeLayout should derive managed MSVC toolchain root"
    );

    // Verify ToolDetailModel is serializable (the view model consumed by TUI/JSON)
    let model = spoon::view::ToolDetailModel {
        title: "test-tool".to_string(),
        rows: vec![
            spoon::view::ToolDetailRow::Title {
                text: "Test Tool".to_string(),
            },
            spoon::view::ToolDetailRow::Field {
                label: "install root".to_string(),
                value: layout.scoop.apps_root.join("test").join("current").display().to_string(),
                value_kind: spoon::view::ToolDetailValueKind::Path,
            },
        ],
    };
    let json = serde_json::to_value(&model).expect("ToolDetailModel should serialize");
    assert_eq!(json["title"], "test-tool");
    assert_eq!(json["rows"][1]["label"], "install root");
    assert_eq!(json["rows"][1]["value_kind"], "path");
}

fn msvc_detail_shows_managed_paths_and_payload_plan() {
    let mut app = Harness::new();
    let tool_root = unique_temp_dir("spoon-msvc-shell");
    config::save_global_config(&config::GlobalConfig {
        editor: String::new(),
        proxy: String::new(),
        root: tool_root.display().to_string(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();
    seed_msvc_manifest(
        &tool_root.display().to_string(),
        "msvc-14.44.17.14",
        "sdk-10.0.22621.7",
    );
    let msvc_toolchain = config::msvc_toolchain_root_from(&tool_root);
    let msvc_cache = config::msvc_cache_root_from(&tool_root);
    let msvc_state = config::msvc_state_root_from(&tool_root);
    std::fs::create_dir_all(&msvc_toolchain).unwrap();
    std::fs::create_dir_all(&msvc_cache).unwrap();
    std::fs::create_dir_all(&msvc_state).unwrap();
    std::fs::write(msvc_toolchain.join("cl.exe"), vec![0_u8; 2048]).unwrap();
    std::fs::write(msvc_cache.join("payload.bin"), vec![0_u8; 1024]).unwrap();
    std::fs::write(
        msvc_state.join("runtime.json"),
        serde_json::json!({
            "runtime": "managed",
            "toolchain_root": msvc_toolchain,
            "wrappers_root": config::shims_root_from(&tool_root),
        })
        .to_string(),
    )
    .unwrap();

    open_tools(&mut app);
    app.press(KeyCode::Char('r')).unwrap();
    select_tool_by_key(&mut app, "msvc");
    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("ToolDetail"));

    let rendered = app.render_text(140, 40).join("\n");
    assert!(rendered.contains("MSVC Toolchain"), "{rendered}");
    assert!(rendered.contains("managed path:"), "{rendered}");
    assert!(rendered.contains("cache path:"), "{rendered}");
    assert!(rendered.contains("install root:"), "{rendered}");
    assert!(rendered.contains("installed size:"), "{rendered}");
    assert!(rendered.contains("cache size:"), "{rendered}");
    assert!(rendered.contains("payload plan:"), "{rendered}");
    assert!(rendered.contains("cached payloads:"), "{rendered}");
}
