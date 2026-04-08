use crossterm::event::KeyCode;
#[path = "../common/mod.rs"]
mod common;

use common::constants::DEFAULT_WAIT;
use common::tui::open_global_form;
use spoon::config;
use spoon::editor;
use spoon::tui::test_support::Harness;
use spoon_core::RuntimeLayout;
use std::fs;

fn set_available_test_editor() {
    config::save_global_config(&config::GlobalConfig {
        editor: "powershell.exe".to_string(),
        proxy: String::new(),
        root: String::new(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();
}

#[test]
fn configure_global_form_opens_read_only_view() {
    let mut app = Harness::new();
    set_available_test_editor();

    app.press(KeyCode::Down).unwrap();
    open_global_form(&mut app);
    let rendered = app.render_text(120, 36).join("\n");
    assert!(rendered.contains("Current configuration"), "{rendered}");
    assert!(rendered.contains("root: unset"), "{rendered}");

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), None);
}

#[test]
fn configure_global_form_wraps_subtitle_on_narrow_windows() {
    let mut app = Harness::new();
    set_available_test_editor();

    app.press(KeyCode::Down).unwrap();
    open_global_form(&mut app);

    let lines = app.render_text(100, 32);
    assert!(
        lines
            .iter()
            .any(|line| line.contains("Inspect the current Spoon-owned"))
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("open the real config file"))
            || lines
                .iter()
                .any(|line| line.contains("when you want to edit it.")),
    );
}

#[test]
fn form_header_flows_into_current_configuration_panel() {
    let mut app = Harness::new();
    set_available_test_editor();

    app.press(KeyCode::Down).unwrap();
    open_global_form(&mut app);

    let lines = app.render_text(120, 36);
    let subtitle_line = lines
        .iter()
        .enumerate()
        .find_map(|(index, line)| {
            line.contains("config file when you want to edit it.")
                .then_some(index)
        })
        .expect("subtitle line");
    let detail_line = lines
        .iter()
        .enumerate()
        .skip(subtitle_line + 1)
        .find_map(|(index, line)| line.contains("Current configuration").then_some(index))
        .expect("current configuration line");

    assert!(detail_line > subtitle_line + 1, "{lines:#?}");
}

#[test]
fn configure_git_form_reads_values_from_gitconfig_file() {
    let mut app = Harness::new();
    set_available_test_editor();

    let gitconfig = config::git_config_path();
    fs::write(
        &gitconfig,
        "[user]\n\tname = Test User\n\temail = test@example.com\n[init]\n\tdefaultBranch = trunk\n[http]\n\tproxy = http://127.0.0.1:7890\n[https]\n\tproxy = http://127.0.0.1:7890\n",
    )
    .unwrap();

    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Enter).unwrap();

    assert_eq!(app.modal_name(), Some("Configuration"));
    let rendered = app.render_text(160, 44).join("\n");
    assert!(rendered.contains("Current configuration"), "{rendered}");
    assert!(
        rendered.contains("native proxy: http://127.0.0.1:7890"),
        "{rendered}"
    );
    assert!(rendered.contains(".gitconfig"), "{rendered}");
}

#[test]
fn form_enter_opens_editor_output_when_editor_is_available() {
    let mut app = Harness::new();
    set_available_test_editor();

    app.press(KeyCode::Down).unwrap();
    open_global_form(&mut app);
    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_title(), Some("Open Global".to_string()));
}

#[test]
fn form_browse_shortcut_e_opens_editor_output() {
    let mut app = Harness::new();
    set_available_test_editor();

    app.press(KeyCode::Down).unwrap();
    open_global_form(&mut app);
    app.press(KeyCode::Char('e')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_title(), Some("Open Global".to_string()));
}

#[test]
fn form_browse_shortcut_o_opens_folder_output() {
    let mut app = Harness::new();
    set_available_test_editor();

    app.press(KeyCode::Down).unwrap();
    open_global_form(&mut app);
    app.press(KeyCode::Char('o')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(
        app.output_title(),
        Some("Open folder for Global".to_string())
    );
}

#[test]
fn closing_form_output_returns_to_the_same_form() {
    let mut app = Harness::new();
    set_available_test_editor();

    app.press(KeyCode::Down).unwrap();
    open_global_form(&mut app);
    assert_eq!(app.modal_name(), Some("Configuration"));
    assert_eq!(app.form_title(), Some("Global"));

    app.press(KeyCode::Char('o')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(
        app.output_title(),
        Some("Open folder for Global".to_string())
    );

    assert!(app.wait_until(DEFAULT_WAIT, |h| { h.output_running() == Some(false) }));

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), Some("Configuration"));
    assert_eq!(app.form_title(), Some("Global"));
}

#[test]
fn form_open_editor_shows_running_output_before_completion() {
    let mut app = Harness::new();
    set_available_test_editor();

    app.press(KeyCode::Down).unwrap();
    open_global_form(&mut app);
    app.press_without_settle(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_title(), Some("Open Global".to_string()));
    assert_eq!(app.output_running(), Some(true));

    assert!(app.wait_until(DEFAULT_WAIT, |h| { h.output_running() == Some(false) }));
}

#[test]
fn form_open_editor_preserves_streamed_lines_after_completion() {
    let mut app = Harness::new();
    set_available_test_editor();

    app.press(KeyCode::Down).unwrap();
    open_global_form(&mut app);
    app.press_without_settle(KeyCode::Enter).unwrap();

    assert!(app.wait_until(DEFAULT_WAIT, |h| { h.output_running() == Some(false) }));

    let lines = app.output_lines().unwrap_or_default();
    assert!(!lines.is_empty());
    assert!(
        lines
            .iter()
            .any(|line| line.contains("Started editor for:"))
    );
}

#[test]
fn q_quits_immediately_from_form_modal() {
    let mut app = Harness::new();
    set_available_test_editor();

    app.press(KeyCode::Down).unwrap();
    open_global_form(&mut app);

    let quit = app.press(KeyCode::Char('q')).unwrap();
    assert!(quit);
}

#[test]
fn question_mark_opens_help_from_form_modal() {
    let mut app = Harness::new();
    set_available_test_editor();

    app.press(KeyCode::Down).unwrap();
    open_global_form(&mut app);

    app.press(KeyCode::Char('?')).unwrap();
    assert_eq!(app.modal_name(), Some("Help"));

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), Some("Configuration"));
    assert_eq!(app.form_title(), Some("Global"));
}

#[test]
fn missing_editor_opens_editor_setup_modal() {
    let mut app = Harness::new();
    config::save_global_config(&config::GlobalConfig {
        editor: "definitely-not-a-real-editor-command".to_string(),
        proxy: String::new(),
        root: String::new(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();

    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("EditorSetup"));
}

#[test]
fn editor_setup_navigation_and_escape_work() {
    let mut app = Harness::new();
    config::save_global_config(&config::GlobalConfig {
        editor: "definitely-not-a-real-editor-command".to_string(),
        proxy: String::new(),
        root: String::new(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();

    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("EditorSetup"));

    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), None);
}

#[test]
fn editor_setup_install_opens_running_output_immediately() {
    let mut app = Harness::new();
    editor::set_test_candidate_availability(Some(false));
    config::save_global_config(&config::GlobalConfig {
        editor: "definitely-not-a-real-editor-command".to_string(),
        proxy: String::new(),
        root: String::new(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();

    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("EditorSetup"));

    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_title(), Some("Install editor Zed".to_string()));
    editor::set_test_candidate_availability(None);
}

#[test]
fn editor_setup_clear_default_stays_in_setup() {
    let mut app = Harness::new();
    config::save_global_config(&config::GlobalConfig {
        editor: "zed".to_string(),
        proxy: String::new(),
        root: String::new(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();

    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("EditorSetup"));

    app.press(KeyCode::Char('x')).unwrap();
    assert_eq!(app.modal_name(), Some("EditorSetup"));
}

#[test]
fn editor_setup_uninstall_refreshes_candidate_status_before_return() {
    let mut app = Harness::new();
    editor::set_test_candidate_availability(Some(true));
    let tool_root = std::env::temp_dir().join(format!(
        "spoon-editor-managed-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let managed_nano = RuntimeLayout::from_root(&tool_root)
        .scoop
        .package_current_root("nano")
        .join("nano.exe");
    std::fs::create_dir_all(managed_nano.parent().unwrap()).unwrap();
    std::fs::write(&managed_nano, b"managed nano").unwrap();
    config::save_global_config(&config::GlobalConfig {
        editor: String::new(),
        proxy: String::new(),
        root: tool_root.display().to_string(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();

    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("EditorSetup"));

    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Char('u')).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(
        app.output_title(),
        Some("Uninstall editor Nano".to_string())
    );
    assert!(app.wait_until(DEFAULT_WAIT, |h| { h.output_running() == Some(false) }));

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), Some("EditorSetup"));

    let rendered = app.render_text(120, 36).join("\n");
    assert!(rendered.contains("selected missing"), "{rendered}");
}

#[test]
fn editor_setup_external_provider_is_not_uninstalled() {
    let mut app = Harness::new();
    editor::set_test_candidate_availability(Some(true));
    config::save_global_config(&config::GlobalConfig {
        editor: String::new(),
        proxy: String::new(),
        root: String::new(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();

    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("EditorSetup"));

    for _ in 0..3 {
        let rendered = app.render_text(120, 36).join("\n");
        if rendered.contains("selected external") {
            break;
        }
        app.press(KeyCode::Down).unwrap();
    }
    app.press(KeyCode::Char('u')).unwrap();
    assert_eq!(app.modal_name(), Some("EditorSetup"));

    let rendered = app.render_text(120, 36).join("\n");
    assert!(rendered.contains("selected external"), "{rendered}");
    assert!(
        rendered.contains("another provider; spoon will not uninstall it"),
        "{rendered}"
    );
}
