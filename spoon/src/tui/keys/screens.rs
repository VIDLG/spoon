use std::path::Path;

use anyhow::Result;
use crossterm::event::KeyCode;

use crate::actions::ToolAction;
use crate::editor;
use crate::packages;
use crate::view::build_tool_detail_model;

use super::super::action_flow::{ToolsActionStart, open_config_target_modal, start_tools_action};
use super::super::{
    EditorSetupState, FormState, Modal, Screen, ToolDetailState, ToolManagerState, ToolsKeyOutcome,
    config_menu_items, new_config_menu,
};

pub(super) fn handle_tools_key(
    state: &mut ToolManagerState,
    code: KeyCode,
    install_root: Option<&Path>,
    action_running: bool,
) -> Result<ToolsKeyOutcome> {
    let len = state.statuses.len();
    let index = state.selected_index().unwrap_or(0);
    let mut outcome = ToolsKeyOutcome {
        next_modal: None,
        next_screen: None,
        background_action: None,
        next_hint: None,
        request_status_refresh: false,
    };

    match code {
        KeyCode::Esc => {
            outcome.next_screen = Some(new_config_menu());
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if len > 0 && index + 1 < len {
                state.table_state.select(Some(index + 1));
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if len > 0 && index > 0 {
                state.table_state.select(Some(index - 1));
            }
        }
        KeyCode::Char('r') => {
            state.refresh_fast(install_root);
            outcome.request_status_refresh = true;
        }
        KeyCode::Char(' ') => {
            if len > 0 {
                state.selected[index] = !state.selected[index];
            }
        }
        KeyCode::Enter => {
            if len > 0 {
                let model = build_tool_detail_model(&state.statuses[index], &state.statuses);
                outcome.next_modal = Some(Modal::ToolDetail(ToolDetailState { scroll: 0, model }));
            }
        }
        KeyCode::Char('a') => state.toggle_select_all(),
        KeyCode::Char('m') => state.select_installable(),
        KeyCode::Char('p') => state.select_installed(),
        KeyCode::Char('c') => state.selected.iter_mut().for_each(|flag| *flag = false),
        KeyCode::Char('i') | KeyCode::Char('I') => {
            match start_tools_action(state, ToolAction::Install, install_root, action_running)? {
                ToolsActionStart::Started(modal, background) => {
                    outcome.next_modal = Some(modal);
                    outcome.background_action = background;
                }
                ToolsActionStart::Hint(hint) => outcome.next_hint = Some(hint),
            }
        }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            match start_tools_action(state, ToolAction::Update, install_root, action_running)? {
                ToolsActionStart::Started(modal, background) => {
                    outcome.next_modal = Some(modal);
                    outcome.background_action = background;
                }
                ToolsActionStart::Hint(hint) => outcome.next_hint = Some(hint),
            }
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            match start_tools_action(state, ToolAction::Uninstall, install_root, action_running)? {
                ToolsActionStart::Started(modal, background) => {
                    outcome.next_modal = Some(modal);
                    outcome.background_action = background;
                }
                ToolsActionStart::Hint(hint) => outcome.next_hint = Some(hint),
            }
        }
        _ => {}
    }

    Ok(outcome)
}

pub(super) fn handle_config_menu_key(
    state: &mut ratatui::widgets::ListState,
    code: KeyCode,
) -> (Option<Screen>, Option<Modal>) {
    let items = config_menu_items();
    let selected = state.selected().unwrap_or(0);
    match code {
        KeyCode::Esc => (Some(new_config_menu()), Some(Modal::QuitConfirm)),
        KeyCode::Down | KeyCode::Char('j') => {
            state.select(Some((selected + 1) % items.len()));
            (None, None)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.select(Some((selected + items.len() - 1) % items.len()));
            (None, None)
        }
        KeyCode::Enter => match items[selected].1 {
            super::super::ConfigMenuAction::Editor => (
                None,
                Some(Modal::EditorSetup(EditorSetupState {
                    selected: 0,
                    current_command: editor::default_editor_status().command,
                    requested_kind: None,
                    notice: None,
                })),
            ),
            super::super::ConfigMenuAction::Global => open_config_target_modal(
                super::super::ConfigKind::Global,
                Modal::Form(FormState::from_global()),
            ),
            super::super::ConfigMenuAction::Package(package_key) => {
                let descriptor = packages::config_target_descriptors()
                    .into_iter()
                    .find(|descriptor| descriptor.package_key == package_key)
                    .expect("registered config target descriptor");
                open_config_target_modal(
                    super::super::ConfigKind::Package(package_key),
                    Modal::Form(FormState::from_package(
                        package_key,
                        descriptor.detail_title,
                    )),
                )
            }
        },
        _ => (None, None),
    }
}
