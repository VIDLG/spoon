use std::path::Path;

use anyhow::Result;

use crate::runtime::block_on_sync;
use crate::service::msvc;
use crate::service::{CancellationToken, CommandResult, CommandStatus, StreamChunk};
use crate::packages::tool::Tool;

use super::super::ToolAction;

pub(super) fn execute_native_action(
    action: ToolAction,
    tool: &'static Tool,
    install_root: Option<&Path>,
) -> Result<CommandResult> {
    if tool.has_managed_toolchain_runtime() {
        execute_msvc_action(action, tool.display_name, install_root)
    } else {
        Ok(CommandResult {
            title: format!("{:?} {}", action, tool.display_name),
            status: CommandStatus::Failed,
            output: vec![format!(
                "{} is managed by a native toolchain flow that is not wired yet.",
                tool.display_name
            )],
            streamed: false,
        })
    }
}

pub(super) fn execute_native_action_streaming<F>(
    action: ToolAction,
    tool: &'static Tool,
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    if tool.has_managed_toolchain_runtime() {
        execute_msvc_action_streaming(action, tool.display_name, install_root, cancel, emit)
    } else {
        let result = execute_native_action(action, tool, install_root)?;
        for line in &result.output {
            emit(StreamChunk::Append(line.clone()));
        }
        Ok(CommandResult {
            streamed: true,
            ..result
        })
    }
}

fn is_official_runtime(root: &Path) -> bool {
    !msvc::runtime_state_path(root).exists() && msvc::official::runtime_state_path(root).exists()
}

fn missing_root_result(action: ToolAction, display_name: &str, streamed: bool) -> CommandResult {
    CommandResult {
        title: format!("{:?} {}", action, display_name),
        status: CommandStatus::Blocked,
        output: vec![
            "MSVC Toolchain requires a configured root.".to_string(),
            "Set root in spoon config before managing the toolchain.".to_string(),
        ],
        streamed,
    }
}

fn cancelled_result(action: ToolAction, display_name: &str) -> CommandResult {
    CommandResult {
        title: format!("{:?} {}", action, display_name),
        status: CommandStatus::Cancelled,
        output: vec!["Cancelled by user.".to_string()],
        streamed: true,
    }
}

fn emit_result_lines<F>(result: &CommandResult, emit: &mut F)
where
    F: FnMut(StreamChunk),
{
    for line in &result.output {
        emit(StreamChunk::Append(line.clone()));
    }
}

fn execute_managed_action(action: ToolAction, root: &Path) -> Result<CommandResult> {
    match action {
        ToolAction::Install => block_on_sync(msvc::install_toolchain_async(root)),
        ToolAction::Update => block_on_sync(msvc::update_toolchain_async(root)),
        ToolAction::Uninstall => block_on_sync(msvc::uninstall_toolchain(root)),
    }
}

fn execute_official_action(action: ToolAction, root: &Path) -> Result<CommandResult> {
    match action {
        ToolAction::Install => block_on_sync(msvc::official::install_toolchain_async_with_mode(
            root,
            msvc::official::OfficialInstallerMode::Passive,
        )),
        ToolAction::Update => block_on_sync(msvc::official::update_toolchain_async_with_mode(
            root,
            msvc::official::OfficialInstallerMode::Passive,
        )),
        ToolAction::Uninstall => block_on_sync(msvc::official::uninstall_toolchain_async(
            root,
            msvc::official::OfficialInstallerMode::Passive,
        )),
    }
}

fn execute_managed_action_streaming<F>(
    action: ToolAction,
    root: &Path,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    match action {
        ToolAction::Install => block_on_sync(msvc::install_toolchain_streaming(root, cancel, emit)),
        ToolAction::Update => block_on_sync(msvc::update_toolchain_streaming(root, cancel, emit)),
        ToolAction::Uninstall => {
            let result = block_on_sync(msvc::uninstall_toolchain(root))?;
            emit_result_lines(&result, emit);
            Ok(CommandResult {
                streamed: true,
                ..result
            })
        }
    }
}

fn execute_official_action_streaming<F>(
    action: ToolAction,
    root: &Path,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    match action {
        ToolAction::Install => block_on_sync(msvc::official::install_toolchain_streaming(
            root,
            msvc::official::OfficialInstallerMode::Passive,
            cancel,
            emit,
        )),
        ToolAction::Update => block_on_sync(msvc::official::update_toolchain_streaming(
            root,
            msvc::official::OfficialInstallerMode::Passive,
            cancel,
            emit,
        )),
        ToolAction::Uninstall => {
            let result = block_on_sync(msvc::official::uninstall_toolchain_streaming(
                root,
                msvc::official::OfficialInstallerMode::Passive,
                cancel,
                emit,
            ))?;
            emit_result_lines(&result, emit);
            Ok(CommandResult {
                streamed: true,
                ..result
            })
        }
    }
}

fn execute_msvc_action_streaming<F>(
    action: ToolAction,
    display_name: &str,
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let Some(root) = install_root else {
        let result = missing_root_result(action, display_name, true);
        emit_result_lines(&result, emit);
        return Ok(result);
    };
    let result = if is_official_runtime(root) {
        execute_official_action_streaming(action, root, cancel, emit)
    } else {
        execute_managed_action_streaming(action, root, cancel, emit)
    };
    match result {
        Ok(result) => Ok(result),
        Err(err) if err.to_string().eq_ignore_ascii_case("cancelled by user") => {
            let result = cancelled_result(action, display_name);
            emit_result_lines(&result, emit);
            Ok(result)
        }
        Err(err) => Err(err),
    }
}

fn execute_msvc_action(
    action: ToolAction,
    display_name: &str,
    install_root: Option<&Path>,
) -> Result<CommandResult> {
    let Some(root) = install_root else {
        return Ok(missing_root_result(action, display_name, false));
    };
    if is_official_runtime(root) {
        execute_official_action(action, root)
    } else {
        execute_managed_action(action, root)
    }
}
