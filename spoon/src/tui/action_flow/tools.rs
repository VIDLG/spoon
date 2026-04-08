use std::path::Path;

use anyhow::Result;

use crate::actions::{self, ToolAction};
use crate::logger;
use crate::runtime;
use crate::service::{CancellationToken, StreamChunk};

use super::super::{
    ActionOutcome, BackgroundAction, BackgroundEvent, Modal, OutputState, ToolManagerState,
    action_past, action_title,
};
use super::complete_background_action;

pub(crate) enum ToolsActionStart {
    Started(Modal, Option<BackgroundAction>),
    Hint(String),
}

pub(crate) fn start_tools_action(
    state: &mut ToolManagerState,
    action: ToolAction,
    install_root: Option<&Path>,
    action_running: bool,
) -> Result<ToolsActionStart> {
    let any_selected = state.selected.iter().any(|flag| *flag);
    let chosen = state.selected_tools_for_action(action);

    if action_running {
        return Ok(ToolsActionStart::Hint(
            "Another tools action is still running. Please wait for it to finish.".to_string(),
        ));
    }

    if chosen.is_empty() {
        let message = if any_selected {
            format!("Selected tools cannot be {}.", action_past(action))
        } else {
            format!("Current tool cannot be {}.", action_past(action))
        };
        return Ok(ToolsActionStart::Hint(format!(
            "{message} Tip: move to an actionable tool, or select multiple with Space."
        )));
    }

    let install_root_buf = install_root.map(Path::to_path_buf);
    let action_label = action_title(action).to_string();
    let tool_count = chosen.len();
    let cancel: CancellationToken = CancellationToken::new();
    logger::tui_tools_action_start(action_title(action), chosen.iter().map(|tool| tool.key));

    let cancel_for_task = cancel.clone();
    let rx = runtime::spawn_with_sender(move |tx| {
        let (result_lines, result_status, append_lines) = actions::execute_tool_action_streaming(
            action,
            &chosen,
            install_root_buf.as_deref(),
            Some(&cancel_for_task),
            |chunk| {
                let _ = tx.send(match chunk {
                    StreamChunk::Append(line) => BackgroundEvent::AppendLine(line),
                    StreamChunk::ReplaceLast(line) => BackgroundEvent::ReplaceLastLine(line),
                });
            },
        )
        .map(|results| {
            let (lines, status) = actions::summarize_streamed_command_results(results);
            (lines, status, true)
        })
        .unwrap_or_else(|err| {
            (
                vec![format!("Error: {err}")],
                "action failed".to_string(),
                true,
            )
        });
        complete_background_action(
            &tx,
            ActionOutcome {
                title: format!("{} tools", action_label),
                status: result_status,
                lines: result_lines,
                append_lines,
                follow_up: None,
            },
            true,
        );
    });

    Ok(ToolsActionStart::Started(
        Modal::Output(OutputState {
            title: format!("{} tools", action_title(action)),
            status: match action {
                ToolAction::Install => "installing tools".to_string(),
                ToolAction::Update => "updating tools".to_string(),
                ToolAction::Uninstall => "uninstalling tools".to_string(),
            },
            lines: vec![
                format!(
                    "Running {} for {} tool(s)...",
                    action_title(action),
                    tool_count
                ),
                "Press Esc or q to cancel the running action.".to_string(),
            ],
            scroll: 0,
            max_scroll: 0,
            page_step: 10,
            auto_scroll: true,
            snap_to_bottom_on_render: false,
            running: true,
            follow_up: None,
        }),
        Some(BackgroundAction { rx, cancel }),
    ))
}
