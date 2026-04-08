mod native;
mod scoop;

use std::path::Path;

use anyhow::Result;

use crate::logger;
use crate::service::CancellationToken;
use crate::service::CommandResult;
use crate::service::StreamChunk;
use crate::packages::tool::{Backend, Tool};

use super::ToolAction;

fn log_tool_action_start(action: ToolAction, tools: &[&'static Tool]) {
    logger::tool_action_start(format!("{:?}", action), tools.iter().map(|tool| tool.key));
}

fn log_tool_action_results(results: &[CommandResult]) {
    logger::command_results(logger::TOOL_ACTION_RESULT, results);
}

fn partition_tools_by_backend(tools: &[&'static Tool]) -> (Vec<&'static Tool>, Vec<&'static Tool>) {
    let mut scoop_tools = Vec::new();
    let mut native_tools = Vec::new();

    for tool in tools {
        match tool.backend {
            Backend::Scoop => scoop_tools.push(*tool),
            Backend::Native => native_tools.push(*tool),
        }
    }

    (scoop_tools, native_tools)
}

pub fn execute_tool_action(
    action: ToolAction,
    tools: &[&'static Tool],
    install_root: Option<&Path>,
) -> Result<Vec<CommandResult>> {
    log_tool_action_start(action, tools);

    let (scoop_tools, native_tools) = partition_tools_by_backend(tools);
    let mut results = Vec::new();

    for tool in native_tools {
        results.push(native::execute_native_action(action, tool, install_root)?);
    }

    if !scoop_tools.is_empty() {
        let mut scoop_results = scoop::execute_scoop_action(action, &scoop_tools, install_root)?;
        results.append(&mut scoop_results);
    }

    log_tool_action_results(&results);
    Ok(results)
}

pub(crate) fn execute_tool_action_streaming<F>(
    action: ToolAction,
    tools: &[&'static Tool],
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    mut emit: F,
) -> Result<Vec<CommandResult>>
where
    F: FnMut(StreamChunk),
{
    log_tool_action_start(action, tools);

    let (scoop_tools, native_tools) = partition_tools_by_backend(tools);
    let mut results = Vec::new();

    for tool in native_tools {
        results.push(native::execute_native_action_streaming(
            action,
            tool,
            install_root,
            cancel,
            &mut emit,
        )?);
    }

    if !scoop_tools.is_empty() {
        let mut scoop_results = scoop::execute_scoop_action_streaming(
            action,
            &scoop_tools,
            install_root,
            cancel,
            &mut emit,
        )?;
        results.append(&mut scoop_results);
    }

    log_tool_action_results(&results);
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::partition_tools_by_backend;
    use crate::packages::tool;

    #[test]
    fn partition_tools_keeps_backend_groups() {
        let git = tool::find_tool("git").expect("git tool");
        let msvc = tool::find_tool("msvc").expect("msvc tool");
        let claude = tool::find_tool("claude").expect("claude tool");

        let (scoop, native) = partition_tools_by_backend(&[git, msvc, claude]);

        let scoop_keys: Vec<_> = scoop.into_iter().map(|tool| tool.key).collect();
        let native_keys: Vec<_> = native.into_iter().map(|tool| tool.key).collect();
        assert_eq!(scoop_keys, vec!["git", "claude"]);
        assert_eq!(native_keys, vec!["msvc"]);
    }
}
