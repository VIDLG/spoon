use std::path::Path;

use anyhow::Result;

use crate::service::{
    CancellationToken, CommandResult, CommandStatus, PackageRef, StreamChunk, scoop,
};
use crate::status::{self, ToolOwnership, ToolStatus};
use crate::packages::tool::Tool;

use super::super::ToolAction;

fn skipped_result(
    action: ToolAction,
    tool: &'static Tool,
    _reason: &str,
) -> CommandResult {
    let verb = match action {
        ToolAction::Install => "install",
        ToolAction::Update => "update",
        ToolAction::Uninstall => "uninstall",
    };

    CommandResult {
        title: format!("{verb} {}", tool.display_name),
        status: CommandStatus::Success,
    }
}

fn skip_reason(
    action: ToolAction,
    tool: &'static Tool,
    statuses: &[ToolStatus],
) -> Option<&'static str> {
    let status = statuses.iter().find(|item| item.tool.key == tool.key)?;
    let detected = status.is_detected();
    let broken = status.broken;
    let update_needed = status.update_available;
    let ownership = status.ownership();

    match action {
        ToolAction::Install
            if matches!(ownership, ToolOwnership::Managed) && detected && !broken =>
        {
            Some("already installed")
        }
        ToolAction::Update if !detected => Some("not installed"),
        ToolAction::Update if matches!(ownership, ToolOwnership::External) => {
            Some("externally managed")
        }
        ToolAction::Update if !update_needed => Some("already up to date"),
        ToolAction::Uninstall if !detected => Some("not installed"),
        ToolAction::Uninstall if matches!(ownership, ToolOwnership::External) => {
            Some("externally managed")
        }
        _ => None,
    }
}

pub(super) fn execute_scoop_action(
    action: ToolAction,
    tools: &[&'static Tool],
    install_root: Option<&Path>,
) -> Result<Vec<CommandResult>> {
    let statuses = if matches!(action, ToolAction::Update) {
        let mut items = status::collect_statuses(install_root);
        status::populate_update_info(&mut items, install_root);
        items
    } else {
        status::collect_statuses(install_root)
    };

    let mut results = Vec::new();
    for tool in tools {
        if let Some(reason) = skip_reason(action, tool, &statuses) {
            results.push(skipped_result(action, tool, reason));
            continue;
        }

        let mut executed = match action {
            ToolAction::Install => {
                let pkg = PackageRef {
                    display_name: tool.display_name,
                    package_name: tool.package_name,
                };
                scoop::install_tools(&[pkg], install_root)
            }
            ToolAction::Update => {
                let pkg = PackageRef {
                    display_name: tool.display_name,
                    package_name: tool.package_name,
                };
                scoop::update_tools(&[pkg], install_root)
            }
            ToolAction::Uninstall => {
                let pkg = PackageRef {
                    display_name: tool.display_name,
                    package_name: tool.package_name,
                };
                scoop::uninstall_tools(&[pkg], install_root)
            }
        }?;
        results.append(&mut executed);
    }
    Ok(results)
}

pub(super) fn execute_scoop_action_streaming<F>(
    action: ToolAction,
    tools: &[&'static Tool],
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<Vec<CommandResult>>
where
    F: FnMut(StreamChunk),
{
    let statuses = if matches!(action, ToolAction::Update) {
        let mut items = status::collect_statuses(install_root);
        status::populate_update_info(&mut items, install_root);
        items
    } else {
        status::collect_statuses(install_root)
    };

    let mut results = Vec::new();
    for tool in tools {
        if let Some(reason) = skip_reason(action, tool, &statuses) {
            let result = skipped_result(action, tool, reason);
            results.push(result);
            continue;
        }

        let mut executed = match action {
            ToolAction::Install => {
                let pkg = PackageRef {
                    display_name: tool.display_name,
                    package_name: tool.package_name,
                };
                scoop::install_tools_streaming(&[pkg], install_root, cancel, emit)
            }
            ToolAction::Update => {
                let pkg = PackageRef {
                    display_name: tool.display_name,
                    package_name: tool.package_name,
                };
                scoop::update_tools_streaming(&[pkg], install_root, cancel, emit)
            }
            ToolAction::Uninstall => {
                let pkg = PackageRef {
                    display_name: tool.display_name,
                    package_name: tool.package_name,
                };
                scoop::uninstall_tools_streaming(&[pkg], install_root, cancel, emit)
            }
        }?;
        results.append(&mut executed);
    }
    Ok(results)
}
