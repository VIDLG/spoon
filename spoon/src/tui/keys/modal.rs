use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};

use crate::actions::ToolAction;
use crate::clipboard;
use crate::view::tool_detail_plain_lines;
use tui_logger::TuiWidgetEvent;

use super::super::action_flow::{
    EditorSetupOutcome, FormOutcome, ToolsActionStart, handle_editor_setup_key, handle_form_key,
    start_tools_action,
};
use super::super::help::help_lines_for_modal;
use super::super::{App, HelpState, Modal, close_modal, new_debug_log_modal, open_modal};
use crate::tui::state::CancelRunningConfirmState;

fn output_clipboard_text(output: &super::super::OutputState) -> String {
    let mut text = String::new();
    text.push_str(&output.title);
    text.push('\n');
    text.push_str("Status: ");
    text.push_str(&output.status);
    if !output.lines.is_empty() {
        text.push_str("\n\n");
        text.push_str(&output.lines.join("\n"));
    }
    text
}

fn detail_clipboard_text(detail: &super::super::ToolDetailState) -> String {
    tool_detail_plain_lines(&detail.model).join("\n")
}

pub(super) fn handle_modal_key(app: &mut App, key: KeyEvent) -> Result<bool> {
    let code = key.code;
    let Some(mut modal) = app.modal.take() else {
        return Ok(false);
    };

    let mut keep_modal = true;
    let mut next_modal = None;
    let mut should_quit = false;
    let mut background_action = None;
    let mut next_hint = None;

    match &mut modal {
        Modal::DebugLog(debug) => match code {
            KeyCode::Esc | KeyCode::Enter => {
                next_modal = debug.follow_up.take().map(|modal| *modal);
                keep_modal = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                debug.widget_state.transition(TuiWidgetEvent::UpKey)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                debug.widget_state.transition(TuiWidgetEvent::DownKey)
            }
            KeyCode::Left | KeyCode::Char('h') => {
                debug.widget_state.transition(TuiWidgetEvent::LeftKey)
            }
            KeyCode::Right | KeyCode::Char('l') => {
                debug.widget_state.transition(TuiWidgetEvent::RightKey)
            }
            KeyCode::Char('+') => debug.widget_state.transition(TuiWidgetEvent::PlusKey),
            KeyCode::Char('-') => debug.widget_state.transition(TuiWidgetEvent::MinusKey),
            KeyCode::Char('H') => debug.widget_state.transition(TuiWidgetEvent::HideKey),
            KeyCode::Char('f') | KeyCode::Char('F') => {
                debug.widget_state.transition(TuiWidgetEvent::FocusKey)
            }
            KeyCode::Char(' ') => debug.widget_state.transition(TuiWidgetEvent::SpaceKey),
            KeyCode::PageUp => debug.widget_state.transition(TuiWidgetEvent::PrevPageKey),
            KeyCode::PageDown => debug.widget_state.transition(TuiWidgetEvent::NextPageKey),
            KeyCode::Char('?') => {
                next_modal = Some(Modal::Help(HelpState {
                    title: "Help - Debug Log".to_string(),
                    lines: help_lines_for_modal(&modal),
                    scroll: 0,
                    follow_up: Some(Box::new(modal.clone())),
                }));
                keep_modal = false;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => should_quit = true,
            _ => {}
        },
        Modal::ToolDetail(detail) => match code {
            KeyCode::Char('D') => {
                let mut debug = match new_debug_log_modal() {
                    Modal::DebugLog(debug) => debug,
                    _ => unreachable!(),
                };
                debug.follow_up = Some(Box::new(modal.clone()));
                next_modal = Some(Modal::DebugLog(debug));
                keep_modal = false;
            }
            KeyCode::Esc | KeyCode::Enter => keep_modal = false,
            KeyCode::Down | KeyCode::Char('j') => detail.scroll = detail.scroll.saturating_add(1),
            KeyCode::Up | KeyCode::Char('k') => detail.scroll = detail.scroll.saturating_sub(1),
            KeyCode::Char('c') | KeyCode::Char('C') => {
                match clipboard::write_text(&detail_clipboard_text(detail)) {
                    Ok(()) => {
                        app.status_hint = Some("Copied tool detail to clipboard.".to_string());
                    }
                    Err(err) => {
                        app.status_hint = Some(format!("Failed to copy tool detail: {err}"));
                    }
                }
            }
            KeyCode::Char('i') | KeyCode::Char('I') => {
                if let super::super::Screen::Tools(state) = &mut app.screen {
                    match start_tools_action(
                        state,
                        ToolAction::Install,
                        app.install_root.as_deref(),
                        app.background_action.is_some(),
                    )? {
                        ToolsActionStart::Started(modal, background) => {
                            next_modal = Some(modal);
                            background_action = background;
                            keep_modal = false;
                        }
                        ToolsActionStart::Hint(hint) => next_hint = Some(hint),
                    }
                }
            }
            KeyCode::Char('u') | KeyCode::Char('U') => {
                if let super::super::Screen::Tools(state) = &mut app.screen {
                    match start_tools_action(
                        state,
                        ToolAction::Update,
                        app.install_root.as_deref(),
                        app.background_action.is_some(),
                    )? {
                        ToolsActionStart::Started(modal, background) => {
                            next_modal = Some(modal);
                            background_action = background;
                            keep_modal = false;
                        }
                        ToolsActionStart::Hint(hint) => next_hint = Some(hint),
                    }
                }
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                if let super::super::Screen::Tools(state) = &mut app.screen {
                    match start_tools_action(
                        state,
                        ToolAction::Uninstall,
                        app.install_root.as_deref(),
                        app.background_action.is_some(),
                    )? {
                        ToolsActionStart::Started(modal, background) => {
                            next_modal = Some(modal);
                            background_action = background;
                            keep_modal = false;
                        }
                        ToolsActionStart::Hint(hint) => next_hint = Some(hint),
                    }
                }
            }
            KeyCode::Char('?') => {
                next_modal = Some(Modal::Help(HelpState {
                    title: "Help - Tool Detail".to_string(),
                    lines: help_lines_for_modal(&modal),
                    scroll: 0,
                    follow_up: Some(Box::new(modal.clone())),
                }));
                keep_modal = false;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => should_quit = true,
            _ => {}
        },
        Modal::Form(form) => {
            if matches!(code, KeyCode::Char('D')) {
                let mut debug = match new_debug_log_modal() {
                    Modal::DebugLog(debug) => debug,
                    _ => unreachable!(),
                };
                debug.follow_up = Some(Box::new(modal.clone()));
                next_modal = Some(Modal::DebugLog(debug));
                keep_modal = false;
            } else if matches!(code, KeyCode::Char('q') | KeyCode::Char('Q')) {
                should_quit = true;
            } else if matches!(code, KeyCode::Char('?')) {
                next_modal = Some(Modal::Help(HelpState {
                    title: format!("Help - {} Form", form.title),
                    lines: help_lines_for_modal(&modal),
                    scroll: 0,
                    follow_up: Some(Box::new(modal.clone())),
                }));
                keep_modal = false;
            } else {
                match handle_form_key(form, key)? {
                    FormOutcome::None => {}
                    FormOutcome::Close => keep_modal = false,
                    FormOutcome::Background(modal, background) => {
                        next_modal = Some(Modal::Output(modal));
                        background_action = Some(background);
                        keep_modal = false;
                    }
                    FormOutcome::EditorSetup(setup) => {
                        next_modal = Some(Modal::EditorSetup(setup));
                        keep_modal = false;
                    }
                }
            }
        }
        Modal::EditorSetup(setup) => {
            if matches!(code, KeyCode::Char('D')) {
                let mut debug = match new_debug_log_modal() {
                    Modal::DebugLog(debug) => debug,
                    _ => unreachable!(),
                };
                debug.follow_up = Some(Box::new(modal.clone()));
                next_modal = Some(Modal::DebugLog(debug));
                keep_modal = false;
            } else if matches!(code, KeyCode::Char('q') | KeyCode::Char('Q')) {
                should_quit = true;
            } else if matches!(code, KeyCode::Char('?')) {
                next_modal = Some(Modal::Help(HelpState {
                    title: "Help - Editor Setup".to_string(),
                    lines: help_lines_for_modal(&modal),
                    scroll: 0,
                    follow_up: Some(Box::new(modal.clone())),
                }));
                keep_modal = false;
            } else {
                match handle_editor_setup_key(setup, code)? {
                    EditorSetupOutcome::None => {}
                    EditorSetupOutcome::Close => keep_modal = false,
                    EditorSetupOutcome::Background(modal, background) => {
                        next_modal = Some(Modal::Output(modal));
                        background_action = Some(background);
                        keep_modal = false;
                    }
                }
            }
        }
        Modal::Output(output) => match code {
            KeyCode::Char('D') => {
                let mut debug = match new_debug_log_modal() {
                    Modal::DebugLog(debug) => debug,
                    _ => unreachable!(),
                };
                debug.follow_up = Some(Box::new(modal.clone()));
                next_modal = Some(Modal::DebugLog(debug));
                keep_modal = false;
            }
            KeyCode::Esc if output.running => {
                next_modal = Some(Modal::CancelRunningConfirm(CancelRunningConfirmState {
                    quit_after_cancel: false,
                    follow_up: Some(Box::new(modal.clone())),
                }));
                keep_modal = false;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') if output.running => {
                next_modal = Some(Modal::CancelRunningConfirm(CancelRunningConfirmState {
                    quit_after_cancel: false,
                    follow_up: Some(Box::new(modal.clone())),
                }));
                keep_modal = false;
            }
            KeyCode::Esc | KeyCode::Enter if !output.running => {
                next_modal = output.follow_up.take().map(|modal| *modal);
                keep_modal = false;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                output.auto_scroll = false;
                output.scroll = output.scroll.min(output.max_scroll).saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                output.auto_scroll = false;
                output.scroll = output.scroll.min(output.max_scroll).saturating_sub(1);
            }
            KeyCode::PageDown => {
                output.auto_scroll = false;
                output.scroll = output
                    .scroll
                    .min(output.max_scroll)
                    .saturating_add(output.page_step.max(1));
            }
            KeyCode::PageUp => {
                output.auto_scroll = false;
                output.scroll = output
                    .scroll
                    .min(output.max_scroll)
                    .saturating_sub(output.page_step.max(1));
            }
            KeyCode::Home => {
                output.auto_scroll = false;
                output.scroll = 0;
            }
            KeyCode::End => {
                output.auto_scroll = false;
                output.scroll = output.max_scroll;
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                match clipboard::write_text(&output_clipboard_text(output)) {
                    Ok(()) => {
                        app.status_hint = Some("Copied full output log to clipboard.".to_string());
                    }
                    Err(err) => {
                        app.status_hint =
                            Some(format!("Failed to copy the full output log: {err}"));
                    }
                }
            }
            KeyCode::Char('?') => {
                next_modal = Some(Modal::Help(HelpState {
                    title: format!("Help - {}", output.title),
                    lines: help_lines_for_modal(&modal),
                    scroll: 0,
                    follow_up: Some(Box::new(modal.clone())),
                }));
                keep_modal = false;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => should_quit = true,
            _ => {}
        },
        Modal::Help(help) => match code {
            KeyCode::Char('D') => {
                let mut debug = match new_debug_log_modal() {
                    Modal::DebugLog(debug) => debug,
                    _ => unreachable!(),
                };
                debug.follow_up = Some(Box::new(modal.clone()));
                next_modal = Some(Modal::DebugLog(debug));
                keep_modal = false;
            }
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('?') => {
                next_modal = help.follow_up.take().map(|modal| *modal);
                keep_modal = false;
            }
            KeyCode::Down | KeyCode::Char('j') => help.scroll = help.scroll.saturating_add(1),
            KeyCode::Up | KeyCode::Char('k') => help.scroll = help.scroll.saturating_sub(1),
            KeyCode::Char('q') | KeyCode::Char('Q') => should_quit = true,
            _ => {}
        },
        Modal::QuitConfirm => match code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Char('q') => {
                should_quit = true
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => keep_modal = false,
            _ => {}
        },
        Modal::CancelRunningConfirm(confirm) => match code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Char('q') => {
                if let Some(background) = app.background_action.as_ref() {
                    background.cancel.cancel();
                }
                if let Some(Modal::Output(output)) = confirm.follow_up.as_deref_mut() {
                    output.status = "cancelling action".to_string();
                }
                app.status_hint = Some("Cancelling running action...".to_string());
                if confirm.quit_after_cancel {
                    should_quit = true;
                } else {
                    next_modal = confirm.follow_up.take().map(|modal| *modal);
                }
                keep_modal = false;
            }
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                next_modal = confirm.follow_up.take().map(|modal| *modal);
                keep_modal = false;
            }
            _ => {}
        },
    }

    if should_quit {
        return Ok(true);
    }
    if let Some(background_action) = background_action {
        app.background_action = Some(background_action);
    }
    if let Some(hint) = next_hint {
        app.status_hint = Some(hint);
    }
    if keep_modal {
        app.modal = Some(modal);
    } else if let Some(next_modal) = next_modal {
        open_modal(app, next_modal);
    } else {
        app.modal = Some(modal);
        close_modal(app);
    }
    Ok(false)
}

pub(super) fn handle_modal_mouse(app: &mut App, mouse: MouseEvent) {
    let Some(modal) = app.modal.as_mut() else {
        return;
    };

    match modal {
        Modal::Output(output) => match mouse.kind {
            MouseEventKind::ScrollDown => {
                output.auto_scroll = false;
                output.scroll = output.scroll.min(output.max_scroll).saturating_add(3);
            }
            MouseEventKind::ScrollUp => {
                output.auto_scroll = false;
                output.scroll = output.scroll.min(output.max_scroll).saturating_sub(3);
            }
            _ => {}
        },
        Modal::Help(help) => match mouse.kind {
            MouseEventKind::ScrollDown => help.scroll = help.scroll.saturating_add(3),
            MouseEventKind::ScrollUp => help.scroll = help.scroll.saturating_sub(3),
            _ => {}
        },
        Modal::ToolDetail(detail) => match mouse.kind {
            MouseEventKind::ScrollDown => detail.scroll = detail.scroll.saturating_add(3),
            MouseEventKind::ScrollUp => detail.scroll = detail.scroll.saturating_sub(3),
            _ => {}
        },
        _ => {}
    }
}
