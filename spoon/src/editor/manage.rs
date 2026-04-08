use anyhow::Result;

use crate::config;
use crate::editor::model::EditorCandidate;
use crate::editor::state;
use crate::logger;
use crate::service::StreamChunk;
use crate::service::{CommandResult, CommandStatus, scoop};

pub fn apply_candidate(candidate: EditorCandidate) -> Result<()> {
    let mut global = config::load_global_config();
    global.editor = candidate.command.to_string();
    config::save_global_config(&global)?;
    logger::editor_default_set(candidate.label, candidate.command);
    Ok(())
}

pub fn clear_default_editor() -> Result<()> {
    let mut global = config::load_global_config();
    global.editor.clear();
    config::save_global_config(&global)?;
    logger::editor_default_cleared();
    Ok(())
}

pub fn install_candidate(candidate: EditorCandidate) -> Result<CommandResult> {
    install_candidate_streaming(candidate, |_| {})
}

fn configured_tool_root() -> Option<std::path::PathBuf> {
    let global = config::load_global_config();
    let trimmed = global.root.trim();
    (!trimmed.is_empty()).then(|| std::path::PathBuf::from(trimmed))
}

pub(crate) fn install_candidate_streaming<F>(
    candidate: EditorCandidate,
    mut emit: F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    logger::editor_install_start(candidate.label, candidate.package_name);
    if state::test_mode_enabled() {
        let result = CommandResult {
            title: format!("install editor {}", candidate.label),
            status: CommandStatus::Success,
            output: vec![
                crate::service::scoop::plan_package_action(
                    "install",
                    candidate.label,
                    candidate.package_name,
                    None,
                )
                .command_line(),
                format!(
                    "Test mode: skipped real Scoop install for {}.",
                    candidate.label
                ),
            ],
            streamed: true,
        };
        for line in &result.output {
            emit(StreamChunk::Append(line.clone()));
        }
        if result.is_success() {
            state::set_availability_override(candidate.command, Some(true));
        }
        logger::command_results(logger::EDITOR_ACTION_RESULT, std::slice::from_ref(&result));
        return Ok(result);
    }

    let Some(tool_root) = configured_tool_root() else {
        return Ok(CommandResult {
            title: format!("install editor {}", candidate.label),
            status: CommandStatus::Blocked,
            output: vec![
                "Editor installation requires a configured root.".to_string(),
                "Set root in spoon config before installing Scoop-managed editors.".to_string(),
            ],
            streamed: true,
        });
    };
    let result = scoop::run_package_action_streaming(
        "install",
        candidate.label,
        candidate.package_name,
        Some(&tool_root),
        None,
        Some(emit),
    )?;
    if result.is_success() {
        state::set_availability_override(candidate.command, Some(true));
    }
    logger::command_results(logger::EDITOR_ACTION_RESULT, std::slice::from_ref(&result));
    Ok(result)
}

pub fn uninstall_candidate(candidate: EditorCandidate) -> Result<CommandResult> {
    uninstall_candidate_streaming(candidate, |_| {})
}

pub(crate) fn uninstall_candidate_streaming<F>(
    candidate: EditorCandidate,
    mut emit: F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    logger::editor_uninstall_start(candidate.label, candidate.package_name);
    if state::test_mode_enabled() {
        let result = CommandResult {
            title: format!("uninstall editor {}", candidate.label),
            status: CommandStatus::Success,
            output: vec![
                crate::service::scoop::plan_package_action(
                    "uninstall",
                    candidate.label,
                    candidate.package_name,
                    None,
                )
                .command_line(),
                format!(
                    "Test mode: skipped real Scoop uninstall for {}.",
                    candidate.label
                ),
            ],
            streamed: true,
        };
        for line in &result.output {
            emit(StreamChunk::Append(line.clone()));
        }
        if result.is_success() {
            state::set_availability_override(candidate.command, Some(false));
        }
        logger::command_results(logger::EDITOR_ACTION_RESULT, std::slice::from_ref(&result));
        return Ok(result);
    }

    let Some(tool_root) = configured_tool_root() else {
        return Ok(CommandResult {
            title: format!("uninstall editor {}", candidate.label),
            status: CommandStatus::Blocked,
            output: vec![
                "Editor uninstall requires a configured root.".to_string(),
                "Set root in spoon config before managing Scoop-managed editors.".to_string(),
            ],
            streamed: true,
        });
    };
    let result = scoop::run_package_action_streaming(
        "uninstall",
        candidate.label,
        candidate.package_name,
        Some(&tool_root),
        None,
        Some(emit),
    )?;
    if result.is_success() {
        state::set_availability_override(candidate.command, Some(false));
    }
    logger::command_results(logger::EDITOR_ACTION_RESULT, std::slice::from_ref(&result));
    Ok(result)
}
