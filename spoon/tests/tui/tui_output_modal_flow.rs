use crossterm::event::KeyCode;
use spoon::tui::test_support::Harness;

#[test]
fn output_scroll_works_after_completion() {
    let mut app = Harness::new();
    let lines = (0..80).map(|i| format!("line {i}")).collect::<Vec<_>>();
    app.set_output_modal_for_test("Test Output", lines, false);
    app.set_output_scroll_metrics_for_test(120, 12);

    assert_eq!(app.output_scroll(), Some(0));

    app.press(KeyCode::PageDown).unwrap();
    let after_page_down = app.output_scroll().unwrap_or(0);
    assert!(after_page_down > 0);

    app.press(KeyCode::End).unwrap();
    assert_eq!(app.output_scroll(), Some(120));
}

#[test]
fn output_replace_last_line_updates_progress_in_place() {
    let mut app = Harness::new();
    app.set_output_modal_for_test(
        "Streaming Output",
        vec!["Running...".to_string(), "Downloading 10%".to_string()],
        true,
    );

    app.replace_output_last_line_for_test("Downloading 42%");

    let lines = app.output_lines().unwrap_or_default();
    assert_eq!(
        lines,
        vec!["Running...".to_string(), "Downloading 42%".to_string()]
    );
}

#[test]
fn output_completion_appends_summary_without_losing_streamed_history() {
    let mut app = Harness::new();
    app.set_output_modal_with_state_for_test(
        "Streaming Output",
        vec![
            "Running Update for 1 tool(s)...".to_string(),
            "Found package".to_string(),
            "Downloading 42%".to_string(),
        ],
        true,
        true,
        0,
    );
    app.set_output_scroll_metrics_for_test(120, 12);

    app.complete_output_for_test(
        "action completed",
        vec!["Updated package successfully.".to_string()],
        true,
    );

    let lines = app.output_lines().unwrap_or_default();
    assert_eq!(
        lines,
        vec![
            "Running Update for 1 tool(s)...".to_string(),
            "Found package".to_string(),
            "Downloading 42%".to_string(),
            "Updated package successfully.".to_string(),
        ]
    );
    assert_eq!(app.output_running(), Some(false));
    assert_eq!(app.output_scroll(), Some(0));
}

#[test]
fn output_completion_preserves_backend_stage_lines_and_final_result() {
    let mut app = Harness::new();
    app.set_output_modal_with_state_for_test(
        "Install tools",
        vec![
            "Stage: planned".to_string(),
            "Stage: surface_applying".to_string(),
        ],
        true,
        true,
        0,
    );

    app.complete_output_for_test(
        "action completed",
        vec![
            "Installed Scoop package 'demo' into D:/tools/scoop/apps/demo/1.0.0".to_string(),
            "Installed state written to D:/tools/scoop/state/control-plane.sqlite3".to_string(),
        ],
        true,
    );

    let lines = app.output_lines().unwrap_or_default();
    assert_eq!(
        lines,
        vec![
            "Stage: planned".to_string(),
            "Stage: surface_applying".to_string(),
            "Installed Scoop package 'demo' into D:/tools/scoop/apps/demo/1.0.0".to_string(),
            "Installed state written to D:/tools/scoop/state/control-plane.sqlite3".to_string(),
        ]
    );
    assert_eq!(app.output_running(), Some(false));
}

#[test]
fn output_completion_can_replace_existing_lines_for_non_streaming_actions() {
    let mut app = Harness::new();
    app.set_output_modal_for_test(
        "Open in editor",
        vec!["Starting editor...".to_string()],
        true,
    );

    app.complete_output_for_test(
        "editor started",
        vec!["Started editor for: C:/tmp/config.toml".to_string()],
        false,
    );

    let lines = app.output_lines().unwrap_or_default();
    assert_eq!(
        lines,
        vec!["Started editor for: C:/tmp/config.toml".to_string()]
    );
    assert_eq!(app.output_running(), Some(false));
}

#[test]
fn running_output_help_returns_to_output_on_close() {
    let mut app = Harness::new();
    app.set_output_modal_with_state_for_test(
        "Install tools",
        vec!["Running Install for 1 tool(s)...".to_string()],
        true,
        true,
        0,
    );

    app.press(KeyCode::Char('?')).unwrap();
    assert_eq!(app.modal_name(), Some("Help"));

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_running(), Some(true));
}

#[test]
fn running_output_debug_log_returns_to_output_on_close() {
    let mut app = Harness::new();
    app.set_output_modal_with_state_for_test(
        "Install tools",
        vec!["Running Install for 1 tool(s)...".to_string()],
        true,
        true,
        0,
    );

    app.press(KeyCode::Char('D')).unwrap();
    assert_eq!(app.modal_name(), Some("DebugLog"));

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_running(), Some(true));
}

#[test]
fn running_output_escape_opens_cancel_confirmation() {
    let mut app = Harness::new();
    app.set_output_modal_with_state_for_test(
        "Install tools",
        vec!["Running Install for 1 tool(s)...".to_string()],
        true,
        true,
        0,
    );

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), Some("CancelRunningConfirm"));

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), Some("Output"));
    assert_eq!(app.output_running(), Some(true));
}

#[test]
fn running_output_q_opens_cancel_confirmation() {
    let mut app = Harness::new();
    app.set_output_modal_with_state_for_test(
        "Install tools",
        vec!["Running Install for 1 tool(s)...".to_string()],
        true,
        true,
        0,
    );

    app.press(KeyCode::Char('q')).unwrap();
    assert_eq!(app.modal_name(), Some("CancelRunningConfirm"));
}

#[test]
fn manual_output_scroll_is_not_overridden_by_new_lines() {
    let mut app = Harness::new();
    let lines = (0..80).map(|i| format!("line {i}")).collect::<Vec<_>>();
    app.set_output_modal_with_state_for_test("Streaming Output", lines, true, true, 0);
    app.set_output_scroll_metrics_for_test(120, 12);

    app.press(KeyCode::PageDown).unwrap();
    let manual_scroll = app.output_scroll().unwrap_or(0);
    assert!(manual_scroll > 0);

    app.append_output_line_for_test("line 80");
    assert_eq!(app.output_scroll(), Some(manual_scroll));
}

#[test]
fn output_end_scroll_reaches_visual_bottom_in_rendered_harness() {
    let mut app = Harness::new();
    let mut lines = (0..24)
        .map(|i| format!("line {i} {}", "wrap-segment ".repeat(10)))
        .collect::<Vec<_>>();
    lines.push(format!("BOTTOM MARKER {}", "tail ".repeat(16)));
    app.set_output_modal_for_test("Test Output", lines, false);

    let initial = app.render_text(140, 40).join("\n");
    assert!(
        !initial.contains("BOTTOM MARKER"),
        "bottom marker should start below the viewport:\n{initial}"
    );

    app.press(KeyCode::End).unwrap();
    let rendered = app.render_text(140, 40).join("\n");
    assert!(
        rendered.contains("BOTTOM MARKER"),
        "bottom marker should be visible after End:\n{rendered}"
    );
}

#[test]
fn output_copy_logs_writes_full_output_to_clipboard() {
    let mut app = Harness::new();
    app.set_output_modal_for_test(
        "Uninstall tools",
        vec![
            "Removing shim 'git.exe'.".to_string(),
            "Native cleanup completed successfully.".to_string(),
        ],
        false,
    );

    app.press(KeyCode::Char('c')).unwrap();

    assert_eq!(
        app.clipboard_text(),
        Some(
            "Uninstall tools\nStatus: action completed\n\nRemoving shim 'git.exe'.\nNative cleanup completed successfully."
                .to_string()
        )
    );
    assert_eq!(
        app.status_hint(),
        Some("Copied full output log to clipboard.".to_string())
    );
}

#[test]
fn running_output_can_also_be_copied_to_clipboard() {
    let mut app = Harness::new();
    app.set_output_modal_for_test(
        "Install tools",
        vec![
            "Running Install for 1 tool(s)...".to_string(),
            "Downloading 42%".to_string(),
        ],
        true,
    );

    app.press(KeyCode::Char('c')).unwrap();

    assert_eq!(
        app.clipboard_text(),
        Some(
            "Install tools\nStatus: running command\n\nRunning Install for 1 tool(s)...\nDownloading 42%"
                .to_string()
        )
    );
}

#[test]
fn running_output_auto_scroll_keeps_latest_lines_visible() {
    let mut app = Harness::new();
    let lines = (0..18)
        .map(|i| format!("line {i} {}", "wrap ".repeat(10)))
        .collect::<Vec<_>>();
    app.set_output_modal_with_state_for_test("Install tools", lines, true, true, 0);

    let before = app.render_text(140, 40).join("\n");
    assert!(
        !before.contains("LIVE MARKER 2"),
        "latest marker should not exist before appends:\n{before}"
    );

    app.append_output_line_for_test(format!("LIVE MARKER 1 {}", "tail ".repeat(10)));
    let first = app.render_text(140, 40).join("\n");
    assert!(
        first.contains("LIVE MARKER 1"),
        "first live marker should stay visible with auto scroll:\n{first}"
    );

    app.append_output_line_for_test(format!("LIVE MARKER 2 {}", "tail ".repeat(10)));
    let second = app.render_text(140, 40).join("\n");
    assert!(
        second.contains("LIVE MARKER 2"),
        "second live marker should stay visible with auto scroll:\n{second}"
    );
}

#[test]
fn output_modal_uses_single_title_and_no_details_box() {
    let mut app = Harness::new();
    app.set_output_modal_for_test("Install tools", vec!["line one".to_string()], false);

    let rendered = app.render_text(140, 40).join("\n");
    assert!(
        rendered.contains("Output / Install tools"),
        "combined output title should be rendered:\n{rendered}"
    );
    assert!(
        !rendered.contains("Details"),
        "details sub-box should not be rendered:\n{rendered}"
    );
}
#[path = "../common/mod.rs"]
mod common;
