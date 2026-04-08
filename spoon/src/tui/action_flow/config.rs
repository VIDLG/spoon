use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::config;
use crate::editor;
use crate::launcher;
use crate::logger;
use crate::packages;
use crate::runtime;

use super::super::{
    ActionOutcome, BackgroundAction, ConfigKind, EditorSetupState, FormState, Modal, OutputState,
};
use super::complete_background_action;

pub(crate) enum FormOutcome {
    None,
    Close,
    Background(OutputState, BackgroundAction),
    EditorSetup(EditorSetupState),
}

pub(crate) fn handle_form_key(form: &mut FormState, key: KeyEvent) -> Result<FormOutcome> {
    let code = key.code;
    let editable = is_editable_target(form.kind);

    match code {
        KeyCode::Esc => Ok(FormOutcome::Close),
        KeyCode::Down | KeyCode::Tab | KeyCode::Up | KeyCode::BackTab => Ok(FormOutcome::None),
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if !editable {
                Ok(FormOutcome::None)
            } else if let Some(output) = maybe_editor_setup_output(form.kind) {
                Ok(FormOutcome::EditorSetup(output))
            } else {
                start_open_form_in_editor(form)
            }
        }
        KeyCode::Char('o') | KeyCode::Char('O') => {
            if editable {
                start_open_form_in_explorer(form)
            } else {
                Ok(FormOutcome::None)
            }
        }
        KeyCode::Enter => {
            if !editable {
                Ok(FormOutcome::None)
            } else if let Some(output) = maybe_editor_setup_output(form.kind) {
                Ok(FormOutcome::EditorSetup(output))
            } else {
                start_open_form_in_editor(form)
            }
        }
        _ => Ok(FormOutcome::None),
    }
}

pub(crate) fn is_editable_target(kind: ConfigKind) -> bool {
    match kind {
        ConfigKind::Global => true,
        ConfigKind::Package(package_key) => packages::config_target_descriptor(package_key)
            .map(|descriptor| descriptor.editable)
            .unwrap_or(false),
    }
}

pub(crate) fn maybe_editor_setup_output(kind: ConfigKind) -> Option<EditorSetupState> {
    let status = editor::default_editor_status();
    if status.available {
        None
    } else {
        Some(EditorSetupState {
            selected: 0,
            current_command: status.command,
            requested_kind: Some(kind),
            notice: None,
        })
    }
}

pub(crate) fn open_config_target_modal(
    target_kind: ConfigKind,
    modal: Modal,
) -> (Option<super::super::Screen>, Option<Modal>) {
    let status = editor::default_editor_status();
    if status.available {
        (None, Some(modal))
    } else {
        (
            None,
            Some(Modal::EditorSetup(EditorSetupState {
                selected: 0,
                current_command: status.command,
                requested_kind: Some(target_kind),
                notice: None,
            })),
        )
    }
}

pub(crate) fn form_for_kind(kind: ConfigKind) -> FormState {
    match kind {
        ConfigKind::Global => FormState::from_global(),
        ConfigKind::Package(package_key) => {
            let descriptor = packages::config_target_descriptor(package_key)
                .expect("registered package config target");
            FormState::from_package(package_key, descriptor.detail_title)
        }
    }
}

pub(crate) fn config_kind_label(kind: ConfigKind) -> String {
    match kind {
        ConfigKind::Global => "Global settings".to_string(),
        ConfigKind::Package(package_key) => {
            let descriptor = packages::config_target_descriptor(package_key)
                .expect("registered package config target");
            format!("{} config", descriptor.detail_title)
        }
    }
}

fn start_open_form_in_editor(form: &FormState) -> Result<FormOutcome> {
    let form_follow_up = form.clone();
    let path = ensure_form_target_exists(form.kind)?;
    let kind = form.kind;
    let title = form.title.to_string();
    let display_path = path.display().to_string();
    let rx = runtime::spawn_with_sender(move |tx| {
        let outcome = match editor::open_file_in_default_editor(kind, &path) {
            Ok(launch) => {
                logger::config_target_open_in_editor(&title, &display_path, &launch.command_line);
                ActionOutcome {
                    title: format!("Open {}", title),
                    status: "editor started".to_string(),
                    lines: vec![
                        format!("Started editor for: {}", display_path),
                        format!("Command: {}", launch.command_line),
                        format!("Process id: {}", launch.pid.unwrap_or(0)),
                    ],
                    append_lines: false,
                    follow_up: Some(Box::new(Modal::Form(form_follow_up.clone()))),
                }
            }
            Err(err) => ActionOutcome {
                title: format!("Open {}", title),
                status: "action failed".to_string(),
                lines: vec![format!("Error: failed to start editor: {err}")],
                append_lines: false,
                follow_up: Some(Box::new(Modal::Form(form_follow_up.clone()))),
            },
        };
        complete_background_action(&tx, outcome, false);
    });

    Ok(FormOutcome::Background(
        OutputState {
            title: format!("Open {}", form.title),
            status: "starting editor".to_string(),
            lines: vec![
                format!("Opening {} in editor...", form.title),
                "Waiting for the editor process to start.".to_string(),
            ],
            scroll: 0,
            max_scroll: 0,
            page_step: 10,
            auto_scroll: false,
            snap_to_bottom_on_render: false,
            running: true,
            follow_up: Some(Box::new(Modal::Form(form.clone()))),
        },
        BackgroundAction {
            rx,
            cancel: crate::bridge::CancellationToken::new(),
        },
    ))
}

fn start_open_form_in_explorer(form: &FormState) -> Result<FormOutcome> {
    let form_follow_up = form.clone();
    let path = ensure_form_target_exists(form.kind)?;
    let title = form.title.to_string();
    let display_path = path.display().to_string();
    let explorer_arg = format!("/select,{}", path.display());
    let rx = runtime::spawn_with_sender(move |tx| {
        let outcome = match launcher::reveal_in_explorer(&path) {
            Ok(launch) => {
                logger::config_target_open_in_explorer(&title, &display_path);
                ActionOutcome {
                    title: format!("Open folder for {}", title),
                    status: "explorer started".to_string(),
                    lines: vec![
                        format!("Started Explorer for: {}", display_path),
                        format!("Command: explorer.exe {}", explorer_arg),
                        format!("Process id: {}", launch.pid.unwrap_or(0)),
                    ],
                    append_lines: false,
                    follow_up: Some(Box::new(Modal::Form(form_follow_up.clone()))),
                }
            }
            Err(err) => ActionOutcome {
                title: format!("Open folder for {}", title),
                status: "action failed".to_string(),
                lines: vec![format!("Error: failed to start Explorer: {err}")],
                append_lines: false,
                follow_up: Some(Box::new(Modal::Form(form_follow_up.clone()))),
            },
        };
        complete_background_action(&tx, outcome, false);
    });

    Ok(FormOutcome::Background(
        OutputState {
            title: format!("Open folder for {}", form.title),
            status: "starting explorer".to_string(),
            lines: vec![
                format!("Opening folder for {}...", form.title),
                "Waiting for Explorer to start.".to_string(),
            ],
            scroll: 0,
            max_scroll: 0,
            page_step: 10,
            auto_scroll: false,
            snap_to_bottom_on_render: false,
            running: true,
            follow_up: Some(Box::new(Modal::Form(form.clone()))),
        },
        BackgroundAction {
            rx,
            cancel: crate::bridge::CancellationToken::new(),
        },
    ))
}

fn ensure_form_target_exists(kind: ConfigKind) -> Result<PathBuf> {
    match kind {
        ConfigKind::Global => config::ensure_global_config_exists(),
        ConfigKind::Package(package_key) => {
            match packages::ensure_editable_config_exists(package_key)? {
                Some(path) => Ok(path),
                None => anyhow::bail!(
                    "no editable native config target is registered for package '{package_key}'"
                ),
            }
        }
    }
}
