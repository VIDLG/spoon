use std::collections::BTreeSet;
use std::path::Path;

use async_recursion::async_recursion;
use tokio::fs;

use crate::Result;
use crate::{
    BackendContext, BackendError, BackendEvent, CancellationToken, CommandStatus,
    SystemPort,
};
use crate::control_plane::{acquire_lock, begin_operation, complete_operation, release_lock, set_operation_stage};
use crate::control_plane::sqlite::db_path_for_layout;
use crate::event::{LifecycleStage, StageEvent};
use crate::layout::RuntimeLayout;
use crate::scoop::state::{
    InstalledPackageState, read_installed_state,
};

use super::super::buckets;
use super::super::ports::ScoopIntegrationPort;
use super::super::cache::package_cache_size;
use super::super::extract::{
    refresh_current_entry,
};
use super::super::lifecycle::acquire::acquire_payloads;
use super::super::lifecycle::integrate::run_integrations;
use super::super::lifecycle::materialize::materialize_payloads;
use super::super::lifecycle::persist::{restore_persist_entries, sync_persist_entries};
use super::super::lifecycle::planner::plan_package_lifecycle;
use super::super::lifecycle::reapply::reapply as reapply_lifecycle;
use super::super::lifecycle::state::{commit_installed_state, remove_installed_state as remove_lifecycle_state};
use super::super::lifecycle::surface::{apply_install_surface, remove_surface};
use super::super::lifecycle::uninstall::uninstall as uninstall_lifecycle;
use super::super::paths;
use super::super::paths::{
    package_app_root, package_current_root, package_persist_root, package_version_root,
};
use super::super::planner::{ScoopPackageAction, ScoopPackagePlan};
use super::execution::ContextRuntimeHost;
use super::hooks::{HookContext, execute_hook_scripts};
use super::integration::helper_executable_path;
use super::source::{dependency_lookup_key, selected_architecture_key};
use super::surface::{
    installed_targets_exist, installer_layout_error,
};
use super::{
    NoopScoopRuntimeHost, ScoopRuntimeHost, ensure_scoop_shims_activated_with_context,
    ensure_scoop_shims_activated_with_host,
};

fn emit_stage(emit: &mut dyn FnMut(BackendEvent), stage: LifecycleStage) {
    let event = if matches!(stage, LifecycleStage::Completed) {
        StageEvent::completed(stage)
    } else {
        StageEvent::started(stage)
    };
    emit(BackendEvent::Stage(event));
}

async fn effective_runtime_plan(
    tool_root: &Path,
    proxy: &str,
    plan: &ScoopPackagePlan,
) -> Result<ScoopPackagePlan> {
    if !matches!(
        plan.action,
        ScoopPackageAction::Install | ScoopPackageAction::Update
    ) {
        return Ok(plan.clone());
    }
    if buckets::load_buckets_from_registry(tool_root).await.is_empty() {
        buckets::ensure_main_bucket_ready(tool_root, proxy).await?;
    }
    if plan.resolved_manifest.is_some() {
        return Ok(plan.clone());
    }
    Ok(super::super::planner::plan_package_action(
        plan.action.as_str(),
        &plan.display_name,
        &plan.package_name,
        Some(tool_root),
    ))
}

async fn install_package(
    tool_root: &Path,
    layout: &RuntimeLayout,
    operation_id: i64,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    let mut visited = BTreeSet::new();
    set_operation_stage(layout, operation_id, LifecycleStage::Acquiring).await?;
    emit_stage(emit, LifecycleStage::Acquiring);
    install_package_with_dependencies(
        tool_root,
        layout,
        operation_id,
        plan,
        proxy,
        cancel,
        host,
        emit,
        &mut visited,
    )
        .await
}

#[async_recursion(?Send)]
async fn install_package_with_dependencies(
    tool_root: &Path,
    layout: &RuntimeLayout,
    operation_id: i64,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
    visited: &mut BTreeSet<String>,
) -> Result<Vec<String>> {
    let planned = plan_package_lifecycle(tool_root, plan).await?;
    let resolved = &planned.resolved;
    let source = &planned.source;
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
        bucket: Some(resolved.bucket.name.clone()),
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
        install_package_with_dependencies(
            tool_root,
            layout,
            operation_id,
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
        sync_persist_entries(
            &package_current_root(tool_root, &plan.package_name),
            &package_persist_root(tool_root, &plan.package_name),
            &previous.persist,
            emit,
        )
        .await?;
        remove_surface(tool_root, &previous.bins, &previous.shortcuts, host).await?;
    }
    let version_root = package_version_root(tool_root, &plan.package_name, &source.version);
    let persist_root = package_persist_root(tool_root, &plan.package_name);
    let shims_root = paths::shims_root(tool_root);
    let archive_paths = acquire_payloads(
        tool_root,
        &plan.package_name,
        source,
        &source.payloads,
        proxy,
        cancel,
        emit,
    )
    .await?;
    set_operation_stage(layout, operation_id, LifecycleStage::Materializing).await?;
    emit_stage(emit, LifecycleStage::Materializing);
    let primary_archive =
        materialize_payloads(tool_root, &archive_paths, source, &version_root, emit).await?;
    let primary_archive = primary_archive.as_deref();
    set_operation_stage(layout, operation_id, LifecycleStage::PreparingHooks).await?;
    emit_stage(emit, LifecycleStage::PreparingHooks);
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
    set_operation_stage(layout, operation_id, LifecycleStage::PersistRestoring).await?;
    emit_stage(emit, LifecycleStage::PersistRestoring);
    restore_persist_entries(&version_root, &persist_root, &source.persist, emit).await?;
    set_operation_stage(layout, operation_id, LifecycleStage::SurfaceApplying).await?;
    emit_stage(emit, LifecycleStage::SurfaceApplying);
    refresh_current_entry(&version_root, &current_root, emit).await?;
    if !installed_targets_exist(&plan.package_name, &current_root, &source, host) {
        return Err(installer_layout_error(&current_root, &source));
    }
    let (aliases, shortcuts) = apply_install_surface(
        &plan.package_name,
        &shims_root,
        &current_root,
        &persist_root,
        &source,
        host,
        emit,
    )
    .await?;
    set_operation_stage(layout, operation_id, LifecycleStage::PostInstallHooks).await?;
    emit_stage(emit, LifecycleStage::PostInstallHooks);
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
    set_operation_stage(layout, operation_id, LifecycleStage::PostInstallHooks).await?;
    emit_stage(emit, LifecycleStage::Integrating);
    set_operation_stage(layout, operation_id, LifecycleStage::Integrating).await?;
    let integrations =
        run_integrations(host, &plan.package_name, &current_root, &persist_root, emit).await?;
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
    let bucket = resolved.bucket.name.clone();
    let architecture = selected_architecture_key();
    set_operation_stage(layout, operation_id, LifecycleStage::StateCommitting).await?;
    emit_stage(emit, LifecycleStage::StateCommitting);
    commit_installed_state(
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
            db_path_for_layout(layout).display()
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

pub(crate) async fn uninstall_package(
    tool_root: &Path,
    layout: &RuntimeLayout,
    operation_id: i64,
    package_name: &str,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    let state = read_installed_state(layout, package_name).await;
    if let Some(state) = &state {
        let current_root = package_current_root(tool_root, package_name);
        let persist_root = package_persist_root(tool_root, package_name);
        let hook_context = HookContext {
            app: package_name.to_string(),
            bucket: Some(state.bucket.clone()),
            buckets_dir: paths::scoop_root(tool_root).join("buckets"),
        };
        set_operation_stage(layout, operation_id, LifecycleStage::PreUninstallHooks).await?;
        emit_stage(emit, LifecycleStage::PreUninstallHooks);
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
            emit,
        )?;
        set_operation_stage(layout, operation_id, LifecycleStage::Uninstalling).await?;
        emit_stage(emit, LifecycleStage::Uninstalling);
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
            emit,
        )?;
        set_operation_stage(layout, operation_id, LifecycleStage::PersistSyncing).await?;
        emit_stage(emit, LifecycleStage::PersistSyncing);
        sync_persist_entries(&current_root, &persist_root, &state.persist, emit)
            .await?;
        set_operation_stage(layout, operation_id, LifecycleStage::SurfaceRemoving).await?;
        emit_stage(emit, LifecycleStage::SurfaceRemoving);
        remove_surface(tool_root, &state.bins, &state.shortcuts, host).await?;
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
    set_operation_stage(layout, operation_id, LifecycleStage::StateRemoving).await?;
    emit_stage(emit, LifecycleStage::StateRemoving);
    remove_lifecycle_state(layout, package_name).await?;
    if let Some(state) = &state {
        let current_root = package_current_root(tool_root, package_name);
        let persist_root = package_persist_root(tool_root, package_name);
        let hook_context = HookContext {
            app: package_name.to_string(),
            bucket: Some(state.bucket.clone()),
            buckets_dir: paths::scoop_root(tool_root).join("buckets"),
        };
        set_operation_stage(layout, operation_id, LifecycleStage::PostUninstallHooks).await?;
        emit_stage(emit, LifecycleStage::PostUninstallHooks);
        if let Err(err) = execute_hook_scripts(
            &state.post_uninstall,
            "post_uninstall",
            &current_root,
            &persist_root,
            None,
            Some(&hook_context),
            &state.version,
            None,
            None,
            emit,
        ) {
            tracing::warn!(
                package = %package_name,
                error = %err,
                "post_uninstall hook failed; continuing because it is warning-only"
            );
        }
    }
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
    let effective_plan = effective_runtime_plan(tool_root, proxy, plan).await?;
    match effective_plan.action {
        ScoopPackageAction::Install | ScoopPackageAction::Update => {
            let mut output = Vec::new();
            for line in ensure_scoop_shims_activated_with_host(tool_root, host).await? {
                tracing::info!("{line}");
                output.push(line);
            }
            let layout = RuntimeLayout::from_root(tool_root);
            output.extend(
                install_package(
                    tool_root,
                    &layout,
                    0,
                    &effective_plan,
                    proxy,
                    cancel,
                    host,
                    emit,
                )
                .await?,
            );
            Ok(output)
        }
        ScoopPackageAction::Uninstall => {
            uninstall_lifecycle(tool_root, 0, &effective_plan.package_name, host, emit).await
        }
        ScoopPackageAction::Reapply => {
            reapply_lifecycle(tool_root, &effective_plan.package_name, host, emit).await
        }
        ScoopPackageAction::Other => Err(BackendError::unsupported_operation(
            "Scoop package",
            "action",
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
    P: SystemPort + ScoopIntegrationPort,
{
    let host = ContextRuntimeHost::new(context);
    let effective_plan =
        effective_runtime_plan(&context.root, context.proxy.as_deref().unwrap_or(""), plan).await?;
    match effective_plan.action {
        ScoopPackageAction::Install | ScoopPackageAction::Update => {
            let mut output = Vec::new();
            for line in ensure_scoop_shims_activated_with_context(context).await? {
                tracing::info!("{line}");
                output.push(line);
            }
            output.extend(
                install_package(
                    &context.root,
                    &RuntimeLayout::from_root(&context.root),
                    0,
                    &effective_plan,
                    context.proxy.as_deref().unwrap_or(""),
                    cancel,
                    &host,
                    emit,
                )
                .await?,
            );
            Ok(output)
        }
        ScoopPackageAction::Uninstall => {
            uninstall_lifecycle(&context.root, 0, &effective_plan.package_name, &host, emit).await
        }
        ScoopPackageAction::Reapply => {
            reapply_lifecycle(&context.root, &effective_plan.package_name, &host, emit).await
        }
        ScoopPackageAction::Other => Err(BackendError::unsupported_operation(
            "Scoop package",
            "action",
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
    let layout = RuntimeLayout::from_root(tool_root);
    let effective_plan = effective_runtime_plan(tool_root, proxy, plan).await?;
    let lock_key = format!(
        "scoop:{}:{}",
        effective_plan.action.as_str(),
        effective_plan.package_name
    );
    if !acquire_lock(&layout, &lock_key, plan.action.as_str()).await? {
        return Err(BackendError::OperationLockHeld { lock_key });
    }
    let bucket_name = effective_plan
        .resolved_manifest
        .as_ref()
        .map(|resolved| resolved.bucket.name.as_str());
    let operation_id = begin_operation(
        &layout,
        effective_plan.action.as_str(),
        Some(&effective_plan.package_name),
        bucket_name,
    )
    .await?;
    set_operation_stage(&layout, operation_id, LifecycleStage::Planned).await?;
    if let Some(line) = effective_plan.resolution_line() {
        tracing::info!("{line}");
        output.push(line);
    }
    let command_line = effective_plan.command_line();
    tracing::info!("{command_line}");
    output.push(command_line);
    let runtime_result = {
        let mut runtime_emit = |chunk| apply_stream_chunk(&mut output, chunk, &mut emit);
        emit_stage(&mut runtime_emit, LifecycleStage::Planned);
        execute_package_action_streaming_with_host(
            tool_root,
            &effective_plan,
            proxy,
            cancel,
            host,
            &mut runtime_emit,
        )
        .await
    };

    let final_result = match runtime_result {
        Ok(runtime_lines) => {
            for line in runtime_lines {
                tracing::info!("{line}");
                output.push(line);
            }
            if let Some(emit_fn) = emit.as_deref_mut() {
                emit_stage(emit_fn, LifecycleStage::Completed);
            }
            set_operation_stage(&layout, operation_id, LifecycleStage::Completed).await?;
            complete_operation(&layout, operation_id, "completed", None).await?;
            Ok(super::super::package_operation_outcome(
                tool_root,
                effective_plan.action.as_str(),
                &effective_plan.package_name,
                &effective_plan.display_name,
                CommandStatus::Success,
                effective_plan.title(),
                output,
                emit.is_some(),
            )
            .await)
        }
        Err(err) => {
            complete_operation(&layout, operation_id, "failed", Some(&err.to_string())).await?;
            Err(err)
        }
    };
    release_lock(&layout, &lock_key).await?;
    final_result
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
    P: SystemPort + ScoopIntegrationPort,
{
    let mut output = Vec::new();
    let layout = RuntimeLayout::from_root(&context.root);
    let effective_plan =
        effective_runtime_plan(&context.root, context.proxy.as_deref().unwrap_or(""), plan).await?;
    let lock_key = format!(
        "scoop:{}:{}",
        effective_plan.action.as_str(),
        effective_plan.package_name
    );
    if !acquire_lock(&layout, &lock_key, plan.action.as_str()).await? {
        return Err(BackendError::OperationLockHeld { lock_key });
    }
    let bucket_name = effective_plan
        .resolved_manifest
        .as_ref()
        .map(|resolved| resolved.bucket.name.as_str());
    let operation_id = begin_operation(
        &layout,
        effective_plan.action.as_str(),
        Some(&effective_plan.package_name),
        bucket_name,
    )
    .await?;
    set_operation_stage(&layout, operation_id, LifecycleStage::Planned).await?;
    if let Some(line) = effective_plan.resolution_line() {
        tracing::info!("{line}");
        output.push(line);
    }
    let command_line = effective_plan.command_line();
    tracing::info!("{command_line}");
    output.push(command_line);
    let runtime_result = {
        let mut runtime_emit = |chunk| apply_stream_chunk(&mut output, chunk, &mut emit);
        emit_stage(&mut runtime_emit, LifecycleStage::Planned);
        execute_package_action_streaming_with_context(
            context,
            &effective_plan,
            cancel,
            &mut runtime_emit,
        )
            .await
    };

    let final_result = match runtime_result {
        Ok(runtime_lines) => {
            for line in runtime_lines {
                tracing::info!("{line}");
                output.push(line);
            }
            if let Some(emit_fn) = emit.as_deref_mut() {
                emit_stage(emit_fn, LifecycleStage::Completed);
            }
            set_operation_stage(&layout, operation_id, LifecycleStage::Completed).await?;
            complete_operation(&layout, operation_id, "completed", None).await?;
            Ok(super::super::package_operation_outcome(
                &context.root,
                effective_plan.action.as_str(),
                &effective_plan.package_name,
                &effective_plan.display_name,
                CommandStatus::Success,
                effective_plan.title(),
                output,
                emit.is_some(),
            )
            .await)
        }
        Err(err) => {
            complete_operation(&layout, operation_id, "failed", Some(&err.to_string())).await?;
            Err(err)
        }
    };
    release_lock(&layout, &lock_key).await?;
    final_result
}
