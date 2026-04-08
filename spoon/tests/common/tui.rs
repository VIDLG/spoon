#![allow(dead_code)]

use crossterm::event::KeyCode;
use spoon::tui::test_support::Harness;

pub fn open_tools(app: &mut Harness) {
    app.press(KeyCode::Right).unwrap();
    assert_eq!(app.screen_name(), "Tools");
}

pub fn open_global_form(app: &mut Harness) {
    app.press(KeyCode::Enter).unwrap();
    assert_eq!(app.modal_name(), Some("Configuration"));
    assert_eq!(app.form_title(), Some("Global"));
}
