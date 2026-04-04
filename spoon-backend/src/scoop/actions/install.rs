use std::collections::BTreeSet;
use std::path::Path;

use async_recursion::async_recursion;

use crate::control_plane::set_operation_stage;
use crate::db::Db;
use crate::event::LifecycleStage;
use crate::layout::RuntimeLayout;
use crate::scoop::state::{
    InstalledPackageCommandSurface, InstalledPackageIdentity, InstalledPackageState,
    InstalledPackageUninstall, read_installed_state, write_installed_state,
};
use crate::{BackendEvent, CancellationToken, Result};

use super::common::emit_stage;
use super::super::cache::package_cache_size;
use super::super::extract::refresh_current_entry;
use super::super::host::{
    HookExecutionContext, HookPhase, ScoopRuntimeHost, execute_hook_scripts,
    helper_executable_path, installed_targets_exist, installer_layout_error,
};
use super::super::lifecycle::acquire::acquire_assets;
use super::super::lifecycle::integrate::run_integrations;
use super::super::lifecycle::materialize::materialize_assets;
use super::super::lifecycle::persist::{restore_persist_entries, sync_persist_entries};
use super::super::lifecycle::surface::{apply_install_surface, remove_surface};
use super::super::package_source::{current_architecture_key, dependency_lookup_key};
use super::super::planner::{ScoopPackageAction, ScoopPackagePlan, plan_package_lifecycle};
use super::super::ports::AppliedIntegration;

pub(super) async fn install_package(
    tool_root: &Path,
    layout: &RuntimeLayout,
    db: &Db,
    test_mode: bool,
    operation_id: i64,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    let mut visited = BTreeSet::new();
    set_operation_stage(db, operation_id, LifecycleStage::Acquiring).await?;
    emit_stage(emit, LifecycleStage::Acquiring);
    install_package_with_dependencies(
        tool_root,
        layout,
        db,
        test_mode,
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
    db: &Db,
    test_mode: bool,
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
    let existing_state = read_installed_state(db, &plan.package_name).await;
    let current_root = layout.scoop.package_current_root(&plan.package_name);
    if let Some(existing) = &existing_state
        && current_root.exists()
        && existing.version() == source.version
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

    for dependency in &source.depends {
        let dependency_name = dependency_lookup_key(dependency);
        if !visited.insert(dependency_name.clone()) {
            continue;
        }
        if read_installed_state(db, &dependency_name).await.is_some()
            || layout.scoop.package_current_root(&dependency_name).exists()
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
            db,
            test_mode,
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
            &layout.scoop.package_current_root(&plan.package_name),
            &layout.scoop.package_persist_root(&plan.package_name),
            &previous.command_surface.persist,
        )
        .await?;
        remove_surface(
            tool_root,
            &previous.command_surface.bins,
            &previous.command_surface.shortcuts,
            test_mode,
        )
        .await?;
    }

    let version_root = layout
        .scoop
        .package_version_root(&plan.package_name, &source.version);
    let persist_root = layout.scoop.package_persist_root(&plan.package_name);
    let shims_root = layout.shims.clone();
    let asset_paths = acquire_assets(
        tool_root,
        &plan.package_name,
        source,
        &source.assets,
        proxy,
        cancel,
        emit,
    )
    .await?;

    set_operation_stage(db, operation_id, LifecycleStage::Materializing).await?;
    emit_stage(emit, LifecycleStage::Materializing);
    let primary_archive =
        materialize_assets(tool_root, &asset_paths, source, &version_root, emit).await?;
    let primary_archive = primary_archive.as_deref();

    set_operation_stage(db, operation_id, LifecycleStage::PreparingHooks).await?;
    emit_stage(emit, LifecycleStage::PreparingHooks);
    execute_hook_scripts(
        &source.pre_install,
        HookPhase::PreInstall,
        &HookExecutionContext {
            command_name: "install",
            version: &source.version,
            install_root: &version_root,
            persist_root: &persist_root,
            archive_path: primary_archive,
            app: Some(&plan.package_name),
            bucket: Some(resolved.bucket.name.as_str()),
            buckets_dir: Some(&layout.scoop.buckets_root),
            dark_helper_path: None,
            innounp_helper_path: None,
        },
    )
    .await?;

    if !source.installer_script.is_empty() {
        let dark_helper = helper_executable_path(tool_root, "dark");
        let innounp_helper = helper_executable_path(tool_root, "innounp");
        execute_hook_scripts(
            &source.installer_script,
            HookPhase::Installer,
            &HookExecutionContext {
                command_name: "install",
                version: &source.version,
                install_root: &version_root,
                persist_root: &persist_root,
                archive_path: primary_archive,
                app: Some(&plan.package_name),
                bucket: Some(resolved.bucket.name.as_str()),
                buckets_dir: Some(&layout.scoop.buckets_root),
                dark_helper_path: dark_helper.as_deref(),
                innounp_helper_path: innounp_helper.as_deref(),
            },
        )
        .await?;
    }

    set_operation_stage(db, operation_id, LifecycleStage::PersistRestoring).await?;
    emit_stage(emit, LifecycleStage::PersistRestoring);
    restore_persist_entries(&version_root, &persist_root, &source.persist).await?;

    set_operation_stage(db, operation_id, LifecycleStage::SurfaceApplying).await?;
    emit_stage(emit, LifecycleStage::SurfaceApplying);
    refresh_current_entry(&version_root, &current_root, emit).await?;
    if !installed_targets_exist(&plan.package_name, &current_root, source, host) {
        return Err(installer_layout_error(&current_root, source));
    }
    let (aliases, shortcuts) = apply_install_surface(
        &plan.package_name,
        &shims_root,
        &current_root,
        &persist_root,
        source,
        test_mode,
        host,
        emit,
    )
    .await?;

    set_operation_stage(db, operation_id, LifecycleStage::PostInstallHooks).await?;
    emit_stage(emit, LifecycleStage::PostInstallHooks);
    execute_hook_scripts(
        &source.post_install,
        HookPhase::PostInstall,
        &HookExecutionContext {
            command_name: "install",
            version: &source.version,
            install_root: &current_root,
            persist_root: &persist_root,
            archive_path: primary_archive,
            app: Some(&plan.package_name),
            bucket: Some(resolved.bucket.name.as_str()),
            buckets_dir: Some(&layout.scoop.buckets_root),
            dark_helper_path: None,
            innounp_helper_path: None,
        },
    )
    .await?;

    set_operation_stage(db, operation_id, LifecycleStage::PostInstallHooks).await?;
    emit_stage(emit, LifecycleStage::Integrating);
    set_operation_stage(db, operation_id, LifecycleStage::Integrating).await?;
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
    let architecture = current_architecture_key();
    set_operation_stage(db, operation_id, LifecycleStage::StateCommitting).await?;
    emit_stage(emit, LifecycleStage::StateCommitting);
    write_installed_state(
        db,
        &InstalledPackageState {
            identity: InstalledPackageIdentity {
                package: plan.package_name.clone(),
                version: source.version.clone(),
                bucket,
                architecture: Some(architecture.to_string()),
                cache_size_bytes,
            },
            command_surface: InstalledPackageCommandSurface {
                bins: aliases.clone(),
                shortcuts: shortcuts.clone(),
                env_add_path: source.env_add_path.clone(),
                env_set: source.env_set.clone(),
                persist: source.persist.clone(),
            },
            integrations: integrations.clone(),
            uninstall: InstalledPackageUninstall {
                pre_uninstall: source.pre_uninstall.clone(),
                uninstaller_script: source.uninstaller_script.clone(),
                post_uninstall: source.post_uninstall.clone(),
            },
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
            layout.scoop.db_path().display()
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
        output.push(format!("Resolved dependencies: {}", source.depends.join(", ")));
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
        output.extend(integrations.iter().map(|AppliedIntegration { key, value }| {
            format!("Applied integration: {key} = {value}")
        }));
    }
    Ok(output)
}
