use crossterm::event::KeyCode;
#[path = "../common/mod.rs"]
mod common;

use common::tui::open_tools;
use spoon::tui::test_support::Harness;

#[test]
fn esc_walks_back_through_the_shell() {
    let mut app = Harness::new();

    assert_eq!(app.screen_name(), "Configure");
    assert_eq!(app.modal_name(), None);

    app.press(KeyCode::Right).unwrap();
    assert_eq!(app.screen_name(), "Tools");

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.screen_name(), "Configure");
    assert_eq!(app.modal_name(), None);

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), Some("QuitConfirm"));
}

#[test]
fn q_quits_without_confirmation() {
    let mut app = Harness::new();
    let quit = app.press(KeyCode::Char('q')).unwrap();
    assert!(quit);
}

#[test]
fn configure_selection_is_preserved_across_page_switches() {
    let mut app = Harness::new();

    assert_eq!(app.config_selected_index(), Some(0));
    app.press(KeyCode::Down).unwrap();
    assert_eq!(app.config_selected_index(), Some(1));

    open_tools(&mut app);
    app.press(KeyCode::Left).unwrap();

    assert_eq!(app.screen_name(), "Configure");
    assert_eq!(app.config_selected_index(), Some(1));
}

#[test]
fn quit_confirm_closes_with_escape() {
    let mut app = Harness::new();

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), Some("QuitConfirm"));

    app.press(KeyCode::Esc).unwrap();
    assert_eq!(app.modal_name(), None);
    assert_eq!(app.screen_name(), "Configure");
}

#[test]
fn tools_state_is_preserved_across_page_switches() {
    let mut app = Harness::new();

    open_tools(&mut app);
    app.press(KeyCode::Down).unwrap();
    app.press(KeyCode::Char(' ')).unwrap();

    assert_eq!(app.tools_selected_index(), Some(1));
    assert_eq!(app.selected_tool_marked(), Some(true));

    app.press(KeyCode::Left).unwrap();
    assert_eq!(app.screen_name(), "Configure");

    app.press(KeyCode::Right).unwrap();
    assert_eq!(app.screen_name(), "Tools");
    assert_eq!(app.tools_selected_index(), Some(1));
    assert_eq!(app.selected_tool_marked(), Some(true));
}

#[test]
fn tool_key_pressed_during_page_transition_is_not_dropped() {
    let mut app = Harness::new();

    app.press_without_settle(KeyCode::Right).unwrap();
    assert_eq!(app.screen_name(), "Tools");

    app.press(KeyCode::Down).unwrap();
    assert_eq!(app.tools_selected_index(), Some(1));
}
