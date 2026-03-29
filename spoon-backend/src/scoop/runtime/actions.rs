use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use async_recursion::async_recursion;
use tokio::fs;

use crate::Result;
use crate::{
    BackendContext, BackendError, BackendEvent, CancellationToken, CommandStatus,
    PackageIntegrationPort, SystemPort,
};
use crate::layout::RuntimeLayout;
use crate::scoop::state::{
    InstalledPackageState, read_installed_state, remove_installed_state, write_installed_state,
};

use super::super::buckets;
use super::super::cache::package_cache_size;
use super::super::extract::{
    extract_archive_to_root, materialize_installer_payloads_to_root, refresh_current_entry,
};
use super::super::paths;
use super::super::paths::{
    package_app_root, package_current_root, package_persist_root, package_state_path,
    package_version_root,
};
use super::super::planner::{ScoopPackageAction, ScoopPackagePlan};
use super::download::ensure_downloaded_archive;
use super::execution::ContextRuntimeHost;
use super::hooks::{HookContext, execute_hook_scripts};
use super::integration::{apply_package_integrations, helper_executable_path};
use super::persist::{restore_persist_entries_into_root, sync_persist_entries_from_root};
use super::source::{dependency_lookup_key, parse_selected_source, selected_architecture_key};
use super::surface::{
    installed_targets_exist, installer_layout_error, load_manifest_value, remove_shims,
    remove_shortcuts, write_shims, write_shortcuts,
};
use super::{
    NoopScoopRuntimeHost, ScoopRuntimeHost, ensure_scoop_shims_activated_with_context,
    ensure_scoop_shims_activated_with_host,
};

async fn install_package(
    tool_root: &Path,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    let mut visited = BTreeSet::new();
    install_package_with_dependencies(tool_root, plan, proxy, cancel, host, emit, &mut visited)
        .await
}

#[async_recursion(?Send)]
async fn install_package_with_dependencies(
    tool_root: &Path,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
    visited: &mut BTreeSet<String>,
) -> Result<Vec<String>> {
    let resolved = plan
        .resolved_manifest
        .as_ref()
        .ok_or_else(|| BackendError::Other("package manifest could not be resolved".to_string()))?;
    let manifest = load_manifest_value(&resolved.manifest_path).await?;
    let source = parse_selected_source(&manifest)?;
    let layout = RuntimeLayout::from_root(tool_root);
    let existing_state = read_installed_state(&layout, &plan.package_name).await;
    let current_root = package_current_root(tool_root, &plan.package_name);
    if let Some(existing) = &existing_state
        && current_root.exists()
        && existing.version == source.version
    {
        let verb = if matches!(plan.action, ScoopPackageAction::Update) {
            "already up to date"
        } else {
            "already installed"
        };
        return Ok(vec![format!(
            "Scoop package '{}' is {} (version {}).",
            plan.package_name, verb, source.version
        )]);
    }
    let hook_context = HookContext {
        app: plan.package_name.clone(),
        bucket: plan
            .resolved_manifest
            .as_ref()
            .map(|resolved| resolved.bucket.name.clone()),
        buckets_dir: paths::scoop_root(tool_root).join("buckets"),
    };
    for dependency in &source.depends {
        let dependency_name = dependency_lookup_key(dependency);
        if !visited.insert(dependency_name.clone()) {
            continue;
        }
        if read_installed_state(&layout, &dependency_name)
            .await
            .is_some()
            || package_current_root(tool_root, &dependency_name).exists()
        {
            tracing::info!("Dependency '{}' is already installed.", dependency_name);
            continue;
        }
        tracing::info!(
            "Installing dependency '{}' for '{}'.",
            dependency_name,
            plan.package_name
        );
        let dependency_plan = super::super::planner::plan_package_action(
            "install",
            &dependency_name,
            &dependency_name,
            Some(tool_root),
        );
        if dependency_plan.resolved_manifest.is_none() {
            return Err(BackendError::Other(format!(
                "dependency '{}' required by '{}' could not be resolved",
                dependency, plan.package_name
            )));
        }
        install_package_with_dependencies(
            tool_root,
            &dependency_plan,
            proxy,
            cancel,
            host,
            emit,
            visited,
        )
        .await?;
    }
    if let Some(previous) = existing_state {
        sync_persist_entries_from_root(
            &package_current_root(tool_root, &plan.package_name),
            &package_persist_root(tool_root, &plan.package_name),
            &previous.persist,
            emit,
        )
        .await?;
        remove_shortcuts(&previous.shortcuts, host).await?;
    }
    let version_root = package_version_root(tool_root, &plan.package_name, &source.version);
    let persist_root = package_persist_root(tool_root, &plan.package_name);
    let shims_root = paths::shims_root(tool_root);
    let mut archive_paths = Vec::new();
    for payload in &source.payloads {
        archive_paths.push(
            ensure_downloaded_archive(
                tool_root,
                &plan.package_name,
                &source,
                payload,
                proxy,
                cancel,
                emit,
            )
            .await?,
        );
    }
    let primary_archive = archive_paths.first().map(PathBuf::as_path);
    if source.installer_script.is_empty() {
        extract_archive_to_root(tool_root, &archive_paths, &source, &version_root, emit).await?;
    } else {
        materialize_installer_payloads_to_root(&archive_paths, &source, &version_root, emit)
            .await?;
    }
    execute_hook_scripts(
        &source.pre_install,
        "pre_install",
        &version_root,
        &persist_root,
        primary_archive,
        Some(&hook_context),
        &source.version,
        None,
        None,
        emit,
    )?;
    if !source.installer_script.is_empty() {
        let dark_helper = helper_executable_path(tool_root, "dark");
        let innounp_helper = helper_executable_path(tool_root, "innounp");
        execute_hook_scripts(
            &source.installer_script,
            "installer",
            &version_root,
            &persist_root,
            primary_archive,
            Some(&hook_context),
            &source.version,
            dark_helper.as_deref(),
            innounp_helper.as_deref(),
            emit,
        )?;
    }
    restore_persist_entries_into_root(&version_root, &persist_root, &source.persist, emit).await?;
    refresh_current_entry(&version_root, &current_root, emit).await?;
    if !installed_targets_exist(&plan.package_name, &current_root, &source, host) {
        return Err(installer_layout_error(&current_root, &source));
    }
    let aliases = write_shims(
        &plan.package_name,
        &shims_root,
        &current_root,
        &persist_root,
        &source,
        host,
        emit,
    )
    .await?;
    let shortcuts = write_shortcuts(&current_root, &persist_root, &source, host, emit).await?;
    execute_hook_scripts(
        &source.post_install,
        "post_install",
        &current_root,
        &persist_root,
        primary_archive,
        Some(&hook_context),
        &source.version,
        None,
        None,
        emit,
    )?;
    let integrations =
        apply_package_integrations(host, &plan.package_name, &current_root, &persist_root, emit)
            .await?;
    let cache_size_bytes = match package_cache_size(tool_root, &plan.package_name).await {
        Ok(size) => Some(size),
        Err(err) => {
            tracing::warn!(
                package = %plan.package_name,
                error = %err,
                "Failed to measure Scoop package cache size"
            );
            None
        }
    };
    let bucket = plan
        .resolved_manifest
        .as_ref()
        .map(|resolved| resolved.bucket.name.clone())
        .unwrap_or_default();
    let architecture = selected_architecture_key();
    write_installed_state(
        &layout,
        &InstalledPackageState {
            package: plan.package_name.clone(),
            version: source.version.clone(),
            bucket,
            architecture: Some(architecture.to_string()),
            cache_size_bytes,
            bins: aliases.clone(),
            shortcuts: shortcuts.clone(),
            env_add_path: source.env_add_path.clone(),
            env_set: source.env_set.clone(),
            persist: source.persist.clone(),
            integrations: integrations.clone(),
            pre_uninstall: source.pre_uninstall.clone(),
            uninstaller_script: source.uninstaller_script.clone(),
            post_uninstall: source.post_uninstall.clone(),
        },
    )
    .await?;
    let mut output = vec![
        format!(
            "Installed Scoop package '{}' into {}",
            plan.package_name,
            version_root.display()
        ),
        format!("Updated current entry: {}", current_root.display()),
        format!(
            "Installed state written to {}",
            package_state_path(tool_root, &plan.package_name).display()
        ),
    ];
    if !source.env_add_path.is_empty() {
        output.push(format!(
            "Recorded env_add_path entries: {}",
            source.env_add_path.join(", ")
        ));
    }
    if !source.env_set.is_empty() {
        output.push(format!(
            "Recorded env_set entries: {}",
            source
                .env_set
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !source.persist.is_empty() {
        output.push(format!(
            "Recorded persist entries: {}",
            source
                .persist
                .iter()
                .map(|entry| entry.relative_path.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !source.depends.is_empty() {
        output.push(format!(
            "Resolved dependencies: {}",
            source.depends.join(", ")
        ));
    }
    if !source.installer_script.is_empty() {
        output.push(format!(
            "Applied installer script lines: {}",
            source.installer_script.len()
        ));
    }
    if !shortcuts.is_empty() {
        output.push(format!(
            "Created shortcuts: {}",
            shortcuts
                .iter()
                .map(|entry| entry.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !integrations.is_empty() {
        output.extend(
            integrations
                .iter()
                .map(|(key, value)| format!("Applied integration: {key} = {value}")),
        );
    }
    Ok(output)
}

async fn uninstall_package(
    tool_root: &Path,
    package_name: &str,
    host: &dyn ScoopRuntimeHost,
) -> Result<Vec<String>> {
    let state = read_installed_state(&RuntimeLayout::from_root(tool_root), package_name).await;
    if let Some(state) = &state {
        let current_root = package_current_root(tool_root, package_name);
        let persist_root = package_persist_root(tool_root, package_name);
        let hook_context = HookContext {
            app: package_name.to_string(),
            bucket: Some(state.bucket.clone()),
            buckets_dir: paths::scoop_root(tool_root).join("buckets"),
        };
        let mut sink = |chunk: BackendEvent| {
            let _ = chunk;
        };
        execute_hook_scripts(
            &state.pre_uninstall,
            "pre_uninstall",
            &current_root,
            &persist_root,
            None,
            Some(&hook_context),
            &state.version,
            None,
            None,
            &mut sink,
        )?;
        execute_hook_scripts(
            &state.uninstaller_script,
            "uninstaller",
            &current_root,
            &persist_root,
            None,
            Some(&hook_context),
            &state.version,
            None,
            None,
            &mut sink,
        )?;
        let mut sink = |_chunk: BackendEvent| {};
        sync_persist_entries_from_root(&current_root, &persist_root, &state.persist, &mut sink)
            .await?;
        remove_shims(tool_root, &state.bins).await?;
        remove_shortcuts(&state.shortcuts, host).await?;
        execute_hook_scripts(
            &state.post_uninstall,
            "post_uninstall",
            &current_root,
            &persist_root,
            None,
            Some(&hook_context),
            &state.version,
            None,
            None,
            &mut sink,
        )?;
    }
    let package_root = package_app_root(tool_root, package_name);
    if package_root.exists() {
        fs::remove_dir_all(&package_root).await.map_err(|err| {
            BackendError::Other(format!(
                "failed to remove {}: {err}",
                package_root.display()
            ))
        })?;
    }
    remove_installed_state(&RuntimeLayout::from_root(tool_root), package_name).await?;
    Ok(vec![format!("Removed Scoop package '{}'.", package_name)])
}

pub async fn execute_package_action_streaming_with_host(
    tool_root: &Path,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    match plan.action {
        ScoopPackageAction::Install | ScoopPackageAction::Update => {
            for line in ensure_scoop_shims_activated_with_host(tool_root, host).await? {
                tracing::info!("{line}");
            }
            install_package(tool_root, plan, proxy, cancel, host, emit).await
        }
        ScoopPackageAction::Uninstall => {
            uninstall_package(tool_root, &plan.package_name, host).await
        }
        ScoopPackageAction::Other => Err(BackendError::Other(
            "unsupported Scoop package action".to_string(),
        )),
    }
}

pub async fn execute_package_action_streaming(
    tool_root: &Path,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    let host = NoopScoopRuntimeHost;
    execute_package_action_streaming_with_host(tool_root, plan, proxy, cancel, &host, emit).await
}

pub async fn execute_package_action_streaming_with_context<P>(
    context: &BackendContext<P>,
    plan: &ScoopPackagePlan,
    cancel: Option<&CancellationToken>,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>>
where
    P: SystemPort + PackageIntegrationPort,
{
    let host = ContextRuntimeHost::new(context);
    match plan.action {
        ScoopPackageAction::Install | ScoopPackageAction::Update => {
            for line in ensure_scoop_shims_activated_with_context(context).await? {
                tracing::info!("{line}");
            }
            install_package(
                &context.root,
                plan,
                context.proxy.as_deref().unwrap_or(""),
                cancel,
                &host,
                emit,
            )
            .await
        }
        ScoopPackageAction::Uninstall => {
            uninstall_package(&context.root, &plan.package_name, &host).await
        }
        ScoopPackageAction::Other => Err(BackendError::Other(
            "unsupported Scoop package action".to_string(),
        )),
    }
}

fn apply_stream_chunk(
    output: &mut Vec<String>,
    chunk: BackendEvent,
    emit: &mut Option<&mut dyn FnMut(BackendEvent)>,
) {
    if let BackendEvent::Finished(finish) = &chunk
        && let Some(message) = &finish.message
    {
        output.push(message.clone());
    }
    if let Some(emit) = emit.as_deref_mut() {
        emit(chunk);
    }
}

pub async fn execute_package_action_outcome_streaming_with_host(
    tool_root: &Path,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    host: &dyn ScoopRuntimeHost,
    mut emit: Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<super::super::ScoopPackageOperationOutcome> {
    let mut output = Vec::new();
    if matches!(
        plan.action,
        super::super::planner::ScoopPackageAction::Install
            | super::super::planner::ScoopPackageAction::Update
    ) && buckets::load_buckets_from_registry(tool_root)
        .await
        .is_empty()
    {
        buckets::ensure_main_bucket_ready(tool_root, proxy).await?;
    }
    if let Some(line) = plan.resolution_line() {
        tracing::info!("{line}");
        output.push(line);
    }
    let command_line = plan.command_line();
    tracing::info!("{command_line}");
    output.push(command_line);
    let mut runtime_emit = |chunk| apply_stream_chunk(&mut output, chunk, &mut emit);
    let runtime_lines = execute_package_action_streaming_with_host(
        tool_root,
        plan,
        proxy,
        cancel,
        host,
        &mut runtime_emit,
    )
    .await?;
    for line in runtime_lines {
        tracing::info!("{line}");
        output.push(line);
    }
    Ok(super::super::package_operation_outcome(
        tool_root,
        plan.action.as_str(),
        &plan.package_name,
        &plan.display_name,
        CommandStatus::Success,
        plan.title(),
        output,
        emit.is_some(),
    )
    .await)
}

pub async fn execute_package_action_outcome_streaming(
    tool_root: &Path,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<super::super::ScoopPackageOperationOutcome> {
    let host = NoopScoopRuntimeHost;
    execute_package_action_outcome_streaming_with_host(tool_root, plan, proxy, cancel, &host, emit)
        .await
}

pub async fn execute_package_action_outcome_streaming_with_context<P>(
    context: &BackendContext<P>,
    plan: &ScoopPackagePlan,
    cancel: Option<&CancellationToken>,
    mut emit: Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<super::super::ScoopPackageOperationOutcome>
where
    P: SystemPort + PackageIntegrationPort,
{
    let mut output = Vec::new();
    if matches!(
        plan.action,
        super::super::planner::ScoopPackageAction::Install
            | super::super::planner::ScoopPackageAction::Update
    ) && buckets::load_buckets_from_registry(&context.root)
        .await
        .is_empty()
    {
        buckets::ensure_main_bucket_ready_with_context(context).await?;
    }
    if let Some(line) = plan.resolution_line() {
        tracing::info!("{line}");
        output.push(line);
    }
    let command_line = plan.command_line();
    tracing::info!("{command_line}");
    output.push(command_line);
    let mut runtime_emit = |chunk| apply_stream_chunk(&mut output, chunk, &mut emit);
    let runtime_lines =
        execute_package_action_streaming_with_context(context, plan, cancel, &mut runtime_emit)
            .await?;
    for line in runtime_lines {
        tracing::info!("{line}");
        output.push(line);
    }
    Ok(super::super::package_operation_outcome(
        &context.root,
        plan.action.as_str(),
        &plan.package_name,
        &plan.display_name,
        CommandStatus::Success,
        plan.title(),
        output,
        emit.is_some(),
    )
    .await)
}
