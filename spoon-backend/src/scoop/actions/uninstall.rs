use std::path::Path;

use tokio::fs;

use crate::control_plane::set_operation_stage;
use crate::db::Db;
use crate::event::LifecycleStage;
use crate::layout::RuntimeLayout;
use crate::scoop::state::{read_installed_state, remove_installed_state};
use crate::{BackendError, BackendEvent, Result};

use super::common::emit_stage;
use super::super::host::{HookExecutionContext, HookPhase, ScoopRuntimeHost, execute_hook_scripts};
use super::super::lifecycle::persist::sync_persist_entries;
use super::super::lifecycle::surface::remove_surface;

pub(crate) async fn uninstall_package(
    tool_root: &Path,
    layout: &RuntimeLayout,
    db: &Db,
    test_mode: bool,
    operation_id: i64,
    package_name: &str,
    _host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    let state = read_installed_state(db, package_name).await;
    if let Some(state) = &state {
        let current_root = layout.scoop.package_current_root(package_name);
        let persist_root = layout.scoop.package_persist_root(package_name);

        set_operation_stage(db, operation_id, LifecycleStage::PreUninstallHooks).await?;
        emit_stage(emit, LifecycleStage::PreUninstallHooks);
        execute_hook_scripts(
            &state.uninstall.pre_uninstall,
            HookPhase::PreUninstall,
            &HookExecutionContext {
                command_name: "uninstall",
                version: state.version(),
                install_root: &current_root,
                persist_root: &persist_root,
                archive_path: None,
                app: Some(package_name),
                bucket: Some(state.bucket()),
                buckets_dir: Some(&layout.scoop.buckets_root),
                dark_helper_path: None,
                innounp_helper_path: None,
            },
        )
        .await?;

        set_operation_stage(db, operation_id, LifecycleStage::Uninstalling).await?;
        emit_stage(emit, LifecycleStage::Uninstalling);
        execute_hook_scripts(
            &state.uninstall.uninstaller_script,
            HookPhase::Uninstaller,
            &HookExecutionContext {
                command_name: "uninstall",
                version: state.version(),
                install_root: &current_root,
                persist_root: &persist_root,
                archive_path: None,
                app: Some(package_name),
                bucket: Some(state.bucket()),
                buckets_dir: Some(&layout.scoop.buckets_root),
                dark_helper_path: None,
                innounp_helper_path: None,
            },
        )
        .await?;

        set_operation_stage(db, operation_id, LifecycleStage::PersistSyncing).await?;
        emit_stage(emit, LifecycleStage::PersistSyncing);
        sync_persist_entries(&current_root, &persist_root, &state.command_surface.persist).await?;

        set_operation_stage(db, operation_id, LifecycleStage::SurfaceRemoving).await?;
        emit_stage(emit, LifecycleStage::SurfaceRemoving);
        remove_surface(
            tool_root,
            &state.command_surface.bins,
            &state.command_surface.shortcuts,
            test_mode,
        )
        .await?;
    }

    let package_root = layout.scoop.package_app_root(package_name);
    if package_root.exists() {
        fs::remove_dir_all(&package_root).await.map_err(|err| {
            BackendError::Other(format!(
                "failed to remove {}: {err}",
                package_root.display()
            ))
        })?;
    }

    set_operation_stage(db, operation_id, LifecycleStage::StateRemoving).await?;
    emit_stage(emit, LifecycleStage::StateRemoving);
    remove_installed_state(db, package_name).await?;

    if let Some(state) = &state {
        let current_root = layout.scoop.package_current_root(package_name);
        let persist_root = layout.scoop.package_persist_root(package_name);
        set_operation_stage(db, operation_id, LifecycleStage::PostUninstallHooks).await?;
        emit_stage(emit, LifecycleStage::PostUninstallHooks);
        if let Err(err) = execute_hook_scripts(
            &state.uninstall.post_uninstall,
            HookPhase::PostUninstall,
            &HookExecutionContext {
                command_name: "uninstall",
                version: state.version(),
                install_root: &current_root,
                persist_root: &persist_root,
                archive_path: None,
                app: Some(package_name),
                bucket: Some(state.bucket()),
                buckets_dir: Some(&layout.scoop.buckets_root),
                dark_helper_path: None,
                innounp_helper_path: None,
            },
        )
        .await
        {
            tracing::warn!(
                package = %package_name,
                error = %err,
                "post_uninstall hook failed; continuing because it is warning-only"
            );
        }
    }

    Ok(vec![format!("Removed Scoop package '{}'.", package_name)])
}
