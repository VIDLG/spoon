use crossterm::event::KeyCode;
#[path = "../common/mod.rs"]
mod common;

use common::fixtures::unique_temp_dir;
use common::tui::open_tools;
use spoon::config;
use spoon::tui::test_support::Harness;

#[test]
fn editor_setup_is_the_first_config_target() {
    let mut app = Harness::new();

    assert_eq!(app.config_selected_index(), Some(0));
    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("EditorSetup"));
}

#[test]
fn missing_editor_redirects_config_target_to_editor_setup() {
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
fn help_popup_opens_and_closes_on_tools() {
    let mut app = Harness::new();

    open_tools(&mut app);

    app.press(KeyCode::Char('?')).unwrap();
    assert_eq!(app.modal_name(), Some("Help"));

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), None);
    assert_eq!(app.screen_name(), "Tools");
}

#[test]
fn help_popup_opens_and_closes_on_configure() {
    let mut app = Harness::new();

    assert_eq!(app.screen_name(), "Configure");
    app.press(KeyCode::Char('?')).unwrap();
    assert_eq!(app.modal_name(), Some("Help"));

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), None);
    assert_eq!(app.screen_name(), "Configure");
}

#[test]
fn q_quits_immediately_from_help_modal() {
    let mut app = Harness::new();

    app.press(KeyCode::Char('?')).unwrap();
    assert_eq!(app.modal_name(), Some("Help"));

    let quit = app.press(KeyCode::Char('q')).unwrap();
    assert!(quit);
}

#[test]
fn uppercase_d_opens_debug_log_modal() {
    let mut app = Harness::new();
    app.press(KeyCode::Char('D')).unwrap();
    assert_eq!(app.modal_name(), Some("DebugLog"));

    let rendered = app.render_text(140, 40).join("\n");
    assert!(rendered.contains("Debug Log"), "{rendered}");
}

#[test]
fn configure_menu_places_global_before_git() {
    let mut app = Harness::new();

    let rendered = app.render_text(140, 40).join("\n");
    let global_index = rendered
        .find("Global settings (proxy/root)")
        .expect("global target");
    let git_index = rendered.find("Configure Git").expect("git target");

    assert!(global_index < git_index, "{rendered}");
}

#[test]
fn configure_page_uses_single_combined_configure_panel() {
    let mut app = Harness::new();
    let tool_root = unique_temp_dir("spoon-msvc-shell");
    config::save_global_config(&config::GlobalConfig {
        editor: String::new(),
        proxy: String::new(),
        root: tool_root.display().to_string(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();
    app.refresh_config_snapshot_for_test();
    app.press(KeyCode::Char('r')).unwrap();

    let lines = app.render_text(160, 44);
    assert!(
        !lines.iter().any(|line| line.contains("Current Setup")),
        "{lines:#?}"
    );
    assert!(
        !lines.iter().any(|line| line.contains("Configure Targets")),
        "{lines:#?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("Global settings (proxy/root) [ready]")),
        "{lines:#?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("Configure MSVC [policy]")),
        "{lines:#?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains(&format!("root: {} | proxy: unset", tool_root.display()))),
        "{lines:#?}"
    );
    assert!(
        lines.iter().any(|line| line.contains("editor: unset")),
        "{lines:#?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("follow_spoon_proxy:")),
        "{lines:#?}"
    );
    assert!(
        lines.iter().any(|line| line.contains("status:")),
        "{lines:#?}"
    );
}

#[test]
fn git_form_shows_desired_detected_and_config_path() {
    let mut app = Harness::new();
    config::save_global_config(&config::GlobalConfig {
        editor: String::new(),
        proxy: "http://127.0.0.1:7897".to_string(),
        root: String::new(),
        msvc_arch: "auto".to_string(),
    })
    .unwrap();
    config::save_policy_config(&config::PolicyConfig {
        python: config::PythonPolicyConfig::default(),
        git: config::GitPolicyConfig {
            follow_spoon_proxy: true,
            command_profile: "default".to_string(),
        },
        msvc: config::MsvcPolicyConfig::default(),
    })
    .unwrap();
    config::save_git_config(&config::GitConfig {
        user_name: "vision".to_string(),
        user_email: "vision@example.com".to_string(),
        default_branch: "main".to_string(),
        proxy: "http://127.0.0.1:9999".to_string(),
    })
    .unwrap();
    app.refresh_config_snapshot_for_test();

    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.form_title(), Some("Git"));

    let rendered = app.render_text(160, 44).join("\n");
    assert!(rendered.contains("Current configuration"), "{rendered}");
    assert!(rendered.contains("follow_spoon_proxy: true"), "{rendered}");
    assert!(
        rendered.contains("command_profile: default (bash)"),
        "{rendered}"
    );
    assert!(
        rendered.contains("proxy: http://127.0.0.1:9999"),
        "{rendered}"
    );
    assert!(rendered.contains(".gitconfig"), "{rendered}");
    assert!(rendered.contains("Conflicts"), "{rendered}");
}

#[test]
fn claude_and_codex_forms_show_detected_config_details() {
    let mut app = Harness::new();
    config::save_claude_config(&config::ClaudeConfig {
        base_url: "https://claude.example.com".to_string(),
        auth_token: "test-token".to_string(),
    })
    .unwrap();
    config::save_codex_config(&config::CodexConfig {
        base_url: "https://api.example.com".to_string(),
        api_key: "test-key".to_string(),
        model: "gpt-5.2-codex".to_string(),
    })
    .unwrap();
    app.refresh_config_snapshot_for_test();

    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.form_title(), Some("Claude Code"));
    let claude = app.render_text(160, 44).join("\n");
    assert!(claude.contains("Current configuration"), "{claude}");
    assert!(
        claude.contains("base_url: https://claude.example.com"),
        "{claude}"
    );
    assert!(claude.contains("auth_token: present"), "{claude}");
    assert!(claude.contains("settings.json"), "{claude}");

    app.press(KeyCode::Esc).unwrap();
    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.form_title(), Some("Codex"));
    let codex = app.render_text(160, 44).join("\n");
    assert!(codex.contains("Current configuration"), "{codex}");
    assert!(codex.contains("model: gpt-5.2-codex"), "{codex}");
    assert!(
        codex.contains("base_url: https://api.example.com"),
        "{codex}"
    );
    assert!(codex.contains("api_key: present"), "{codex}");
    assert!(codex.contains("config.toml"), "{codex}");
}
