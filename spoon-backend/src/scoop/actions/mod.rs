mod common;
mod install;
mod uninstall;

use std::path::Path;

use crate::Result;
use crate::{
    BackendContext, BackendError, BackendEvent, CancellationToken, CommandStatus, SystemPort,
};
use crate::control_plane::{acquire_lock, begin_operation, complete_operation, release_lock, set_operation_stage};
use crate::db::Db;
use crate::event::LifecycleStage;
use crate::layout::RuntimeLayout;

use self::common::{effective_runtime_plan, emit_stage};
use self::install::install_package;
pub(crate) use self::uninstall::uninstall_package;

use super::host::{
    NoopPorts, ScoopRuntimeHost, ensure_scoop_shims_activated_with_host,
    reapply_package_command_surface,
    reapply_package_integrations,
};
use super::ports::ScoopIntegrationPort;
use super::planner::{ScoopPackageAction, ScoopPackagePlan};
use super::{ScoopPackageOperationOutcome, package_operation_outcome};

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
            ensure_scoop_shims_activated_with_host(tool_root, host).await?;
            let layout = RuntimeLayout::from_root(tool_root);
            let db = Db::open(&layout.scoop.db_path()).await?;
            output.extend(
                install_package(
                    tool_root,
                    &layout,
                    &db,
                    false,
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
            let layout = RuntimeLayout::from_root(tool_root);
            let db = Db::open(&layout.scoop.db_path()).await?;
            uninstall_package(
                tool_root,
                &layout,
                &db,
                false,
                0,
                &effective_plan.package_name,
                host,
                emit,
            )
            .await
        }
        ScoopPackageAction::Reapply => {
            reapply_package_command_surface(
                tool_root,
                &effective_plan.package_name,
                host,
                emit,
            )
            .await?;
            reapply_package_integrations(
                tool_root,
                &effective_plan.package_name,
                host,
                emit,
            )
            .await?;
            Ok(Vec::new())
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
    let host = NoopPorts;
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
    let effective_plan =
        effective_runtime_plan(&context.root, context.proxy.as_deref().unwrap_or(""), plan).await?;
    match effective_plan.action {
        ScoopPackageAction::Install | ScoopPackageAction::Update => {
            let mut output = Vec::new();
            ensure_scoop_shims_activated_with_host(&context.root, context).await?;
            let layout = RuntimeLayout::from_root(&context.root);
            let db = Db::open(&layout.scoop.db_path()).await?;
            output.extend(
                install_package(
                    &context.root,
                    &layout,
                    &db,
                    context.test_mode,
                    0,
                    &effective_plan,
                    context.proxy.as_deref().unwrap_or(""),
                    cancel,
                    context,
                    emit,
                )
                .await?,
            );
            Ok(output)
        }
        ScoopPackageAction::Uninstall => {
            let layout = RuntimeLayout::from_root(&context.root);
            let db = Db::open(&layout.scoop.db_path()).await?;
            uninstall_package(
                &context.root,
                &layout,
                &db,
                context.test_mode,
                0,
                &effective_plan.package_name,
                context,
                emit,
            )
            .await
        }
        ScoopPackageAction::Reapply => {
            reapply_package_command_surface(
                &context.root,
                &effective_plan.package_name,
                context,
                emit,
            )
            .await?;
            reapply_package_integrations(
                &context.root,
                &effective_plan.package_name,
                context,
                emit,
            )
            .await?;
            Ok(Vec::new())
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
) -> Result<ScoopPackageOperationOutcome> {
    let mut output = Vec::new();
    let layout = RuntimeLayout::from_root(tool_root);
    let db = Db::open(&layout.scoop.db_path()).await?;
    let effective_plan = effective_runtime_plan(tool_root, proxy, plan).await?;
    let lock_key = format!(
        "scoop:{}:{}",
        effective_plan.action.as_str(),
        effective_plan.package_name
    );
    if !acquire_lock(&db, &lock_key, plan.action.as_str()).await? {
        return Err(BackendError::OperationLockHeld { lock_key });
    }
    let bucket_name = effective_plan
        .resolved_manifest
        .as_ref()
        .map(|resolved| resolved.bucket.name.as_str());
    let operation_id = begin_operation(
        &db,
        effective_plan.action.as_str(),
        Some(&effective_plan.package_name),
        bucket_name,
    )
    .await?;
    set_operation_stage(&db, operation_id, LifecycleStage::Planned).await?;
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
            set_operation_stage(&db, operation_id, LifecycleStage::Completed).await?;
            complete_operation(&db, operation_id, "completed", None).await?;
            Ok(package_operation_outcome(
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
            complete_operation(&db, operation_id, "failed", Some(&err.to_string())).await?;
            Err(err)
        }
    };
    release_lock(&db, &lock_key).await?;
    final_result
}

pub async fn execute_package_action_outcome_streaming(
    tool_root: &Path,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<ScoopPackageOperationOutcome> {
    let host = NoopPorts;
    execute_package_action_outcome_streaming_with_host(tool_root, plan, proxy, cancel, &host, emit)
        .await
}

pub async fn execute_package_action_outcome_streaming_with_context<P>(
    context: &BackendContext<P>,
    plan: &ScoopPackagePlan,
    cancel: Option<&CancellationToken>,
    mut emit: Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<ScoopPackageOperationOutcome>
where
    P: SystemPort + ScoopIntegrationPort,
{
    let mut output = Vec::new();
    let layout = RuntimeLayout::from_root(&context.root);
    let db = Db::open(&layout.scoop.db_path()).await?;
    let effective_plan =
        effective_runtime_plan(&context.root, context.proxy.as_deref().unwrap_or(""), plan).await?;
    let lock_key = format!(
        "scoop:{}:{}",
        effective_plan.action.as_str(),
        effective_plan.package_name
    );
    if !acquire_lock(&db, &lock_key, plan.action.as_str()).await? {
        return Err(BackendError::OperationLockHeld { lock_key });
    }
    let bucket_name = effective_plan
        .resolved_manifest
        .as_ref()
        .map(|resolved| resolved.bucket.name.as_str());
    let operation_id = begin_operation(
        &db,
        effective_plan.action.as_str(),
        Some(&effective_plan.package_name),
        bucket_name,
    )
    .await?;
    set_operation_stage(&db, operation_id, LifecycleStage::Planned).await?;
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
            set_operation_stage(&db, operation_id, LifecycleStage::Completed).await?;
            complete_operation(&db, operation_id, "completed", None).await?;
            Ok(package_operation_outcome(
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
            complete_operation(&db, operation_id, "failed", Some(&err.to_string())).await?;
            Err(err)
        }
    };
    release_lock(&db, &lock_key).await?;
    final_result
}
