use anyhow::Result;
use crossterm::event::KeyCode;

use crate::editor;
use crate::logger;
use crate::runtime;
use crate::bridge::StreamChunk;

use super::super::{
    ActionOutcome, BackgroundAction, BackgroundEvent, ConfigKind, EditorSetupState, Modal,
    OutputState,
};
use super::complete_background_action;
use super::config::{config_kind_label, form_for_kind};

pub(crate) enum EditorSetupOutcome {
    None,
    Close,
    Background(OutputState, BackgroundAction),
}

pub(crate) fn handle_editor_setup_key(
    setup: &mut EditorSetupState,
    code: KeyCode,
) -> Result<EditorSetupOutcome> {
    let total = editor::candidates().len();
    match code {
        KeyCode::Esc => Ok(EditorSetupOutcome::Close),
        KeyCode::Down | KeyCode::Tab | KeyCode::Char('j') => {
            setup.selected = (setup.selected + 1) % total;
            setup.notice = None;
            Ok(EditorSetupOutcome::None)
        }
        KeyCode::Up | KeyCode::BackTab | KeyCode::Char('k') => {
            setup.selected = (setup.selected + total - 1) % total;
            setup.notice = None;
            Ok(EditorSetupOutcome::None)
        }
        KeyCode::Enter | KeyCode::Char('i') | KeyCode::Char('I') => {
            let candidate = editor::candidates()[setup.selected];
            if editor::is_candidate_available(candidate) {
                editor::apply_candidate(candidate)?;
                logger::editor_setup_default_inline(candidate.label, candidate.command);
                setup.current_command = candidate.command.to_string();
                setup.notice = Some(format!("Set {} as the default editor.", candidate.label));
                return Ok(EditorSetupOutcome::None);
            }
            let requested_kind = setup.requested_kind;
            let selected = setup.selected;
            let requested_target = requested_kind.map(|kind| format!("{kind:?}"));
            logger::editor_setup_install_request(candidate.label, requested_target.as_deref());
            let rx = runtime::spawn_with_sender(move |tx| {
                let outcome = match editor::install_candidate_streaming(candidate, |chunk| {
                    let _ = tx.send(match chunk {
                        StreamChunk::Append(line) => BackgroundEvent::AppendLine(line),
                        StreamChunk::ReplaceLast(line) => BackgroundEvent::ReplaceLastLine(line),
                    });
                }) {
                    Ok(result) => {
                        let mut lines = Vec::new();
                        let mut success = result.is_success();
                        if success {
                            if let Err(err) = editor::apply_candidate(candidate) {
                                success = false;
                                lines.push(format!("Error: failed to save default editor: {err}"));
                            } else {
                                lines.push(String::new());
                                lines.push(format!(
                                    "Configured {} as the default editor command.",
                                    candidate.command
                                ));
                                if let Some(kind) = requested_kind {
                                    lines.push(format!(
                                        "Close this output to return to {}.",
                                        config_kind_label(kind)
                                    ));
                                }
                            }
                        }
                        ActionOutcome {
                            title: format!("Install editor {}", candidate.label),
                            status: if success {
                                "action completed".to_string()
                            } else {
                                "action failed".to_string()
                            },
                            lines,
                            append_lines: true,
                            follow_up: if success && requested_kind.is_some() {
                                requested_kind
                                    .map(|kind| Box::new(Modal::Form(form_for_kind(kind))))
                            } else {
                                Some(Box::new(editor_setup_follow_up(
                                    selected,
                                    requested_kind,
                                    None,
                                )))
                            },
                        }
                    }
                    Err(err) => ActionOutcome {
                        title: format!("Install editor {}", candidate.label),
                        status: "action failed".to_string(),
                        lines: vec![format!("Error: {err}")],
                        append_lines: true,
                        follow_up: Some(Box::new(editor_setup_follow_up(
                            selected,
                            requested_kind,
                            None,
                        ))),
                    },
                };
                complete_background_action(&tx, outcome, true);
            });
            Ok(EditorSetupOutcome::Background(
                OutputState {
                    title: format!("Install editor {}", candidate.label),
                    status: "installing editor".to_string(),
                    lines: vec![
                        format!("Installing {}...", candidate.label),
                        "Please wait for the installer to finish.".to_string(),
                    ],
                    scroll: 0,
                    max_scroll: 0,
                    page_step: 10,
                    auto_scroll: true,
                    snap_to_bottom_on_render: false,
                    running: true,
                    follow_up: None,
                },
                BackgroundAction {
                    rx,
                    cancel: crate::bridge::CancellationToken::new(),
                },
            ))
        }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            let candidate = editor::candidates()[setup.selected];
            if !editor::is_candidate_managed(candidate) {
                setup.notice = Some(format!(
                    "{} is available from another provider; spoon will not uninstall it.",
                    candidate.label
                ));
                return Ok(EditorSetupOutcome::None);
            }
            let was_default = editor::is_default_candidate(candidate);
            let requested_kind = setup.requested_kind;
            let selected = setup.selected;
            let requested_target = requested_kind.map(|kind| format!("{kind:?}"));
            logger::editor_setup_uninstall_request(candidate.label, requested_target.as_deref());
            let rx = runtime::spawn_with_sender(move |tx| {
                let outcome = match editor::uninstall_candidate_streaming(candidate, |chunk| {
                    let _ = tx.send(match chunk {
                        StreamChunk::Append(line) => BackgroundEvent::AppendLine(line),
                        StreamChunk::ReplaceLast(line) => BackgroundEvent::ReplaceLastLine(line),
                    });
                }) {
                    Ok(result) => {
                        let mut lines = Vec::new();
                        let mut success = result.is_success();
                        if success && was_default {
                            if let Err(err) = editor::clear_default_editor() {
                                success = false;
                                lines.push(format!(
                                    "Error: failed to clear default editor setting: {err}"
                                ));
                            } else {
                                lines.push(String::new());
                                lines.push(
                                    "Cleared the default editor command after uninstall."
                                        .to_string(),
                                );
                            }
                        }
                        ActionOutcome {
                            title: format!("Uninstall editor {}", candidate.label),
                            status: if success {
                                "action completed".to_string()
                            } else {
                                "action failed".to_string()
                            },
                            lines,
                            append_lines: true,
                            follow_up: Some(Box::new(editor_setup_follow_up(
                                selected,
                                requested_kind,
                                None,
                            ))),
                        }
                    }
                    Err(err) => ActionOutcome {
                        title: format!("Uninstall editor {}", candidate.label),
                        status: "action failed".to_string(),
                        lines: vec![format!("Error: {err}")],
                        append_lines: true,
                        follow_up: Some(Box::new(editor_setup_follow_up(
                            selected,
                            requested_kind,
                            None,
                        ))),
                    },
                };
                complete_background_action(&tx, outcome, true);
            });
            Ok(EditorSetupOutcome::Background(
                OutputState {
                    title: format!("Uninstall editor {}", candidate.label),
                    status: "uninstalling editor".to_string(),
                    lines: vec![
                        format!("Uninstalling {}...", candidate.label),
                        "Please wait for the uninstall to finish.".to_string(),
                    ],
                    scroll: 0,
                    max_scroll: 0,
                    page_step: 10,
                    auto_scroll: true,
                    snap_to_bottom_on_render: false,
                    running: true,
                    follow_up: None,
                },
                BackgroundAction {
                    rx,
                    cancel: crate::bridge::CancellationToken::new(),
                },
            ))
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            editor::clear_default_editor()?;
            logger::editor_setup_clear_default_inline();
            setup.notice = Some("Cleared the configured default editor.".to_string());
            Ok(EditorSetupOutcome::None)
        }
        _ => Ok(EditorSetupOutcome::None),
    }
}

fn editor_setup_follow_up(
    selected: usize,
    requested_kind: Option<ConfigKind>,
    notice: Option<String>,
) -> Modal {
    let current_command = editor::default_editor_status().command;
    Modal::EditorSetup(EditorSetupState {
        selected,
        current_command,
        requested_kind,
        notice,
    })
}
