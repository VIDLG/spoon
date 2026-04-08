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
        })
    }
}

pub(super) fn execute_native_action_streaming<F>(
    action: ToolAction,
    tool: &'static Tool,
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    emit: F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    if tool.has_managed_toolchain_runtime() {
        execute_msvc_action_streaming(action, tool.display_name, install_root, cancel, emit)
    } else {
        execute_native_action(action, tool, install_root)
    }
}

fn is_official_runtime(root: &Path) -> bool {
    !msvc::runtime_state_path(root).exists() && msvc::official::runtime_state_path(root).exists()
}

fn missing_root_result(action: ToolAction, display_name: &str) -> CommandResult {
    CommandResult {
        title: format!("{:?} {}", action, display_name),
        status: CommandStatus::Blocked,
    }
}

fn cancelled_result(action: ToolAction, display_name: &str) -> CommandResult {
    CommandResult {
        title: format!("{:?} {}", action, display_name),
        status: CommandStatus::Cancelled,
    }
}

fn execute_managed_action(action: ToolAction, root: &Path) -> Result<CommandResult> {
    match action {
        ToolAction::Install => block_on_sync(msvc::install_toolchain(root, None, None)),
        ToolAction::Update => block_on_sync(msvc::update_toolchain(root, None, None)),
        ToolAction::Uninstall => block_on_sync(msvc::uninstall_toolchain(root, None, None)),
    }
}

fn execute_official_action(action: ToolAction, root: &Path) -> Result<CommandResult> {
    let mode = msvc::official::OfficialInstallerMode::Passive;
    match action {
        ToolAction::Install => block_on_sync(msvc::official::install_toolchain(root, mode, None, None)),
        ToolAction::Update => block_on_sync(msvc::official::update_toolchain(root, mode, None, None)),
        ToolAction::Uninstall => block_on_sync(msvc::official::uninstall_toolchain(root, mode, None, None)),
    }
}

fn execute_managed_action_streaming<F>(
    action: ToolAction,
    root: &Path,
    cancel: Option<&CancellationToken>,
    emit: F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    match action {
        ToolAction::Install => block_on_sync(msvc::install_toolchain_with_emit(root, cancel, emit)),
        ToolAction::Update => block_on_sync(msvc::update_toolchain_with_emit(root, cancel, emit)),
        ToolAction::Uninstall => {
            block_on_sync(msvc::uninstall_toolchain(root, cancel, None))
        }
    }
}

fn execute_official_action_streaming<F>(
    action: ToolAction,
    root: &Path,
    cancel: Option<&CancellationToken>,
    emit: F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let mode = msvc::official::OfficialInstallerMode::Passive;
    match action {
        ToolAction::Install => block_on_sync(msvc::official::install_toolchain_with_emit(root, mode, cancel, emit)),
        ToolAction::Update => block_on_sync(msvc::official::update_toolchain_with_emit(root, mode, cancel, emit)),
        ToolAction::Uninstall => {
            block_on_sync(msvc::official::uninstall_toolchain_with_emit(root, mode, cancel, emit))
        }
    }
}

fn execute_msvc_action_streaming<F>(
    action: ToolAction,
    display_name: &str,
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    emit: F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let Some(root) = install_root else {
        return Ok(missing_root_result(action, display_name));
    };
    let result = if is_official_runtime(root) {
        execute_official_action_streaming(action, root, cancel, emit)
    } else {
        execute_managed_action_streaming(action, root, cancel, emit)
    };
    match result {
        Ok(result) => Ok(result),
        Err(err) if err.to_string().eq_ignore_ascii_case("cancelled by user") => {
            Ok(cancelled_result(action, display_name))
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
        return Ok(missing_root_result(action, display_name));
    };
    if is_official_runtime(root) {
        execute_official_action(action, root)
    } else {
        execute_managed_action(action, root)
    }
}
