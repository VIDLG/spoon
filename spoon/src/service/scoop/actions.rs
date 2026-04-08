use std::path::Path;

use anyhow::{Context, Result as AnyResult};

use crate::runtime::block_on_sync;
use crate::service::{CancellationToken, PackageRef, StreamChunk};

use super::runtime;
use super::{
    CommandResult, CommandStatus, RunMode, ScoopPackageActionOutcome, ScoopPackageInstallState,
    command_result, command_result_from_scoop_package_outcome, configured_proxy, infer_tool_root,
    plan_package_action,
};

fn fake_result(
    action: &str,
    display_name: &str,
    package_name: &str,
    tool_root: Option<&Path>,
    streamed: bool,
) -> CommandResult {
    let initial_plan = plan_package_action(action, display_name, package_name, tool_root);
    let mut output = Vec::new();
    if let Some(line) = initial_plan.resolution_line() {
        output.push(line);
    }
    output.push(initial_plan.command_line());
    output.push(format!(
        "Test mode: skipped real Scoop {action} for {display_name}."
    ));
    command_result(
        initial_plan.title(),
        CommandStatus::Success,
        output,
        streamed,
    )
}

fn configured_root_override() -> Option<String> {
    let trimmed = crate::config::load_global_config().root.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn run_scoop(action: &str, pkg: PackageRef) -> AnyResult<CommandResult> {
    run_scoop_streaming(action, pkg, None, Option::<fn(StreamChunk)>::None)
}

pub(crate) fn run_scoop_streaming<F>(
    action: &str,
    pkg: PackageRef,
    cancel: Option<&CancellationToken>,
    mut emit: Option<F>,
) -> AnyResult<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let root_override = configured_root_override();
    let tool_root = infer_tool_root(None, root_override.as_deref());

    if super::should_fake() {
        let result = fake_result(
            action,
            pkg.display_name,
            pkg.package_name,
            tool_root.as_deref(),
            emit.is_some(),
        );
        if let Some(ref mut emit) = emit {
            for line in &result.output {
                emit(StreamChunk::Append(line.clone()));
            }
        }
        return Ok(result);
    }

    let configured_tool_root =
        tool_root.context("Scoop package actions require a configured root")?;
    let plan = plan_package_action(
        action,
        pkg.display_name,
        pkg.package_name,
        Some(&configured_tool_root),
    );
    let mut emit_dyn = emit
        .as_mut()
        .map(|emit| emit as &mut dyn FnMut(StreamChunk));
    let outcome = block_on_sync(runtime::execute_package_action_outcome_streaming(
        &configured_tool_root,
        &plan,
        &configured_proxy(),
        cancel,
        emit_dyn.as_deref_mut(),
    ))?;
    Ok(command_result_from_scoop_package_outcome(outcome))
}

pub(crate) fn run_package_action_streaming<F>(
    action: &str,
    display_name: &str,
    package_name: &str,
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    mut emit: Option<F>,
) -> AnyResult<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let root_override = configured_root_override();
    let root_for_plan = infer_tool_root(install_root, root_override.as_deref());

    if super::should_fake() {
        let result = fake_result(
            action,
            display_name,
            package_name,
            root_for_plan.as_deref(),
            emit.is_some(),
        );
        if let Some(ref mut emit) = emit {
            for line in &result.output {
                emit(StreamChunk::Append(line.clone()));
            }
        }
        return Ok(result);
    }

    let tool_root = root_for_plan.context("Scoop package actions require a configured root")?;
    let plan = plan_package_action(action, display_name, package_name, Some(&tool_root));
    let mut emit_dyn = emit
        .as_mut()
        .map(|emit| emit as &mut dyn FnMut(StreamChunk));
    let outcome = block_on_sync(runtime::execute_package_action_outcome_streaming(
        &tool_root,
        &plan,
        &configured_proxy(),
        cancel,
        emit_dyn.as_deref_mut(),
    ))?;
    Ok(command_result_from_scoop_package_outcome(outcome))
}

fn run_many(
    mode: RunMode,
    packages: &[PackageRef],
    _install_root: Option<&Path>,
) -> AnyResult<Vec<CommandResult>> {
    let mut results = Vec::new();

    for pkg in packages {
        let action = match mode {
            RunMode::Install => run_scoop("install", *pkg),
            RunMode::Update => run_scoop("update", *pkg),
            RunMode::Uninstall => run_scoop("uninstall", *pkg),
        }?;
        results.push(action);
    }

    Ok(results)
}

pub fn install_tools(
    packages: &[PackageRef],
    install_root: Option<&Path>,
) -> AnyResult<Vec<CommandResult>> {
    run_many(RunMode::Install, packages, install_root)
}

pub fn update_tools(
    packages: &[PackageRef],
    install_root: Option<&Path>,
) -> AnyResult<Vec<CommandResult>> {
    run_many(RunMode::Update, packages, install_root)
}

pub fn uninstall_tools(
    packages: &[PackageRef],
    install_root: Option<&Path>,
) -> AnyResult<Vec<CommandResult>> {
    run_many(RunMode::Uninstall, packages, install_root)
}

pub(crate) fn install_tools_streaming<F>(
    packages: &[PackageRef],
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> AnyResult<Vec<CommandResult>>
where
    F: FnMut(StreamChunk),
{
    run_many_streaming(RunMode::Install, packages, install_root, cancel, emit)
}

pub(crate) fn update_tools_streaming<F>(
    packages: &[PackageRef],
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> AnyResult<Vec<CommandResult>>
where
    F: FnMut(StreamChunk),
{
    run_many_streaming(RunMode::Update, packages, install_root, cancel, emit)
}

pub(crate) fn uninstall_tools_streaming<F>(
    packages: &[PackageRef],
    install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> AnyResult<Vec<CommandResult>>
where
    F: FnMut(StreamChunk),
{
    run_many_streaming(RunMode::Uninstall, packages, install_root, cancel, emit)
}

fn run_many_streaming<F>(
    mode: RunMode,
    packages: &[PackageRef],
    _install_root: Option<&Path>,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> AnyResult<Vec<CommandResult>>
where
    F: FnMut(StreamChunk),
{
    let mut results = Vec::new();

    for pkg in packages {
        let action = match mode {
            RunMode::Install => run_scoop_streaming("install", *pkg, cancel, Some(&mut *emit)),
            RunMode::Update => run_scoop_streaming("update", *pkg, cancel, Some(&mut *emit)),
            RunMode::Uninstall => run_scoop_streaming("uninstall", *pkg, cancel, Some(&mut *emit)),
        }?;
        results.push(action);
    }

    Ok(results)
}

pub fn package_action_result(
    tool_root: &Path,
    action: &str,
    package_name: &str,
    display_name: &str,
    result: &CommandResult,
) -> AnyResult<ScoopPackageActionOutcome> {
    let installed_state = block_on_sync(spoon_scoop::read_installed_state(
        &spoon_core::RuntimeLayout::from_root(tool_root).scoop,
        package_name,
    ))
    .ok()
    .flatten();
    let (installed, installed_version, current) = match installed_state {
        Some(state) => {
            let version = state.version().to_string();
            let layout = spoon_core::RuntimeLayout::from_root(tool_root);
            let current_path = layout.scoop.package_current_root(package_name);
            (true, Some(version), Some(current_path.display().to_string()))
        }
        None => (false, None, None),
    };
    Ok(ScoopPackageActionOutcome {
        kind: "scoop_package_action",
        action: action.to_string(),
        package: super::ScoopActionPackage {
            name: package_name.to_string(),
            display_name: display_name.to_string(),
        },
        success: result.is_success(),
        title: result.title.clone(),
        streamed: result.streamed,
        output: result.output.clone(),
        state: ScoopPackageInstallState {
            installed,
            installed_version,
            current,
        },
    })
}

#[cfg(test)]
mod tests {
    use crate::config;
    use crate::service::{PackageRef, StreamChunk};

    use super::{run_package_action_streaming, run_scoop_streaming};

    #[test]
    fn fake_streaming_install_output_includes_no_update_scoop() {
        config::enable_test_mode();
        let pkg = PackageRef {
            display_name: "uv",
            package_name: "uv",
        };
        let result = run_scoop_streaming("install", pkg, None, Option::<fn(StreamChunk)>::None)
            .expect("fake scoop install");
        assert!(
            result.output.iter().any(|line| line
                .contains("Planned Spoon package action (Scoop): install uv --no-update-scoop")),
            "output: {:?}",
            result.output
        );
    }

    #[test]
    fn fake_package_install_output_includes_no_update_scoop() {
        config::enable_test_mode();
        let result = run_package_action_streaming(
            "install",
            "uv",
            "uv",
            None,
            None,
            Option::<fn(StreamChunk)>::None,
        )
        .expect("fake package install");
        assert!(
            result.output.iter().any(|line| line
                .contains("Planned Spoon package action (Scoop): install uv --no-update-scoop")),
            "output: {:?}",
            result.output
        );
    }
}
