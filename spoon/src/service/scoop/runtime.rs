use std::path::Path;

use anyhow::Result as AnyResult;
use spoon_scoop::{ScoopPackageAction, ScoopPackageOperationOutcome};

use crate::service::{
    APP_PORTS, CancellationToken, StreamChunk, stream_chunk_from_event,
};

use super::ScoopPackagePlan;

pub(crate) async fn doctor_details(
    tool_root: &Path,
) -> AnyResult<spoon_scoop::ScoopDoctorDetails> {
    Ok(spoon_scoop::doctor(tool_root).await)
}

pub(crate) fn resolved_pip_mirror_url_for_display(policy_value: &str) -> String {
    crate::service::resolved_pip_mirror_url_for_display(policy_value)
}

pub(crate) async fn reapply_package_integrations(
    tool_root: &Path,
    package_name: &str,
) -> AnyResult<Vec<String>> {
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    spoon_scoop::reapply_integrations(&layout.scoop, package_name, &APP_PORTS, None)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(vec![format!(
        "Reapplied integrations for '{}'.",
        package_name
    )])
}

pub(crate) async fn reapply_package_integrations_with_emit<F>(
    tool_root: &Path,
    package_name: &str,
    mut emit: F,
) -> AnyResult<Vec<String>>
where
    F: FnMut(StreamChunk),
{
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    let (sender, mut receiver) = spoon_core::event_bus(64);

    spoon_scoop::reapply_integrations(&layout.scoop, package_name, &APP_PORTS, Some(&sender))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    while let Ok(Some(event)) = receiver.try_recv() {
        if let Some(chunk) = stream_chunk_from_event(event) {
            emit(chunk);
        }
    }

    Ok(vec![format!(
        "Reapplied integrations for '{}'.",
        package_name
    )])
}

pub(crate) async fn reapply_package_command_surface(
    tool_root: &Path,
    package_name: &str,
) -> AnyResult<Vec<String>> {
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    spoon_scoop::reapply_command_surface(
        &layout.scoop,
        &layout.shims,
        package_name,
        &APP_PORTS,
        None,
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(vec![format!(
        "Reapplied command surface for '{}'.",
        package_name
    )])
}

pub(crate) async fn reapply_package_command_surface_with_emit<F>(
    tool_root: &Path,
    package_name: &str,
    mut emit: F,
) -> AnyResult<Vec<String>>
where
    F: FnMut(StreamChunk),
{
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    let (sender, mut receiver) = spoon_core::event_bus(64);

    spoon_scoop::reapply_command_surface(
        &layout.scoop,
        &layout.shims,
        package_name,
        &APP_PORTS,
        Some(&sender),
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    while let Ok(Some(event)) = receiver.try_recv() {
        if let Some(chunk) = stream_chunk_from_event(event) {
            emit(chunk);
        }
    }

    Ok(vec![format!(
        "Reapplied command surface for '{}'.",
        package_name
    )])
}

/// Execute a package install/update/uninstall action using spoon-scoop.
///
/// Events are forwarded to the caller's emit closure after the operation completes.
pub(crate) async fn execute_package_action_outcome_streaming(
    tool_root: &Path,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: Option<&mut dyn FnMut(StreamChunk)>,
) -> AnyResult<ScoopPackageOperationOutcome> {
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    let scoop_layout = &layout.scoop;

    // Create event bus for collecting operation events
    let (sender, mut receiver) = spoon_core::event_bus(64);

    // Build HTTP client with proxy support
    let client = spoon_core::ReqwestClientBuilder::new()
        .proxy(proxy)
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .build()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Execute the action via spoon-scoop
    let result: spoon_scoop::Result<()> = match plan.action {
        ScoopPackageAction::Install | ScoopPackageAction::Update => {
            spoon_scoop::install_package(
                scoop_layout,
                &client,
                plan,
                proxy,
                cancel,
                &APP_PORTS,
                Some(&sender),
            )
            .await
        }
        ScoopPackageAction::Uninstall => {
            spoon_scoop::uninstall_package(scoop_layout, plan, &APP_PORTS, Some(&sender)).await
        }
        ScoopPackageAction::Reapply => {
            spoon_scoop::uninstall_package(scoop_layout, plan, &APP_PORTS, Some(&sender)).await
        }
        _ => Err(spoon_scoop::ScoopError::Other(format!(
            "unsupported action: {:?}",
            plan.action
        ))),
    };

    // Forward collected events to the caller
    if let Some(emit) = emit {
        while let Ok(Some(event)) = receiver.try_recv() {
            if let Some(chunk) = stream_chunk_from_event(event) {
                emit(chunk);
            }
        }
    }

    // Build outcome
    let status = if result.is_ok() {
        spoon_core::CommandStatus::Success
    } else {
        spoon_core::CommandStatus::Failed
    };

    Ok(ScoopPackageOperationOutcome {
        kind: "package_operation",
        action: plan.action.as_str().to_string(),
        package: spoon_scoop::ScoopActionPackage {
            name: plan.package_name.clone(),
            display_name: plan.display_name.clone(),
        },
        status,
        title: plan.title(),
        state: Default::default(),
    })
}
