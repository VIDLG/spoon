use std::path::Path;

use crate::{BackendContext, BackendError, BackendEvent, CancellationToken, Result, check_token_cancel};

use super::{
    MsvcOperationKind, MsvcOperationOutcome, MsvcRequest, MsvcRuntimeKind, ToolchainFlags,
    ensure_cached_companion_cabs, ensure_cached_payloads, ensure_extracted_archives,
    ensure_extracted_msis, ensure_install_image, ensure_materialized_toolchain,
    ensure_msi_media_metadata, ensure_staged_external_cabs, managed_toolchain_flags_with_request,
    manifest, manifest_dir, msvc_dir, native_host_arch, paths, remove_autoenv_dir,
    remove_managed_toolchain_wrappers, user_facing_toolchain_label, write_installed_state,
    write_managed_toolchain_wrappers, write_runtime_state, cleanup_post_install_cache,
    read_installed_toolchain_target, runtime_state_path, push_stream_line,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolchainAction {
    Install,
    Update,
}

impl ToolchainAction {
    fn title(self) -> &'static str {
        match self {
            Self::Install => "install MSVC Toolchain",
            Self::Update => "update MSVC Toolchain",
        }
    }

    const fn operation_kind(self) -> MsvcOperationKind {
        match self {
            Self::Install => MsvcOperationKind::Install,
            Self::Update => MsvcOperationKind::Update,
        }
    }
}

pub fn handle_manifest_refresh_failure(
    action: ToolchainAction,
    lines: &mut Vec<String>,
    emit: &mut Option<&mut dyn FnMut(BackendEvent)>,
    err: BackendError,
) -> Result<()> {
    if action == ToolchainAction::Update {
        return Err(BackendError::Other(format!(
            "failed to refresh latest managed MSVC manifest for update: {err}"
        )));
    }
    push_stream_line(
        lines,
        emit,
        format!("Warning: failed to refresh managed MSVC manifest cache: {err}"),
    );
    Ok(())
}

fn managed_toolchain_is_current(tool_root: &Path, latest: &manifest::ToolchainTarget) -> bool {
    paths::msvc_toolchain_root(tool_root).exists()
        && runtime_state_path(tool_root).exists()
        && read_installed_toolchain_target(&paths::msvc_root(tool_root))
            .is_some_and(|installed| installed.msvc == latest.msvc && installed.sdk == latest.sdk)
}

async fn run_toolchain_action_async(
    request: &MsvcRequest,
    action: ToolchainAction,
    cancel: Option<&CancellationToken>,
    mut emit: Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<MsvcOperationOutcome> {
    check_token_cancel(cancel)?;
    let tool_root = request.root.as_path();
    let proxy = request.proxy.as_str();
    let command_profile = request.command_profile.as_str();
    let selected_target_arch = request.normalized_target_arch();
    let mut lines = Vec::new();
    let manifest_root = manifest_dir(tool_root);
    if !request.test_mode {
        match manifest::sync_release_manifest_cache_async(&manifest_root, proxy).await {
            Ok(sync_lines) => {
                for line in sync_lines {
                    push_stream_line(&mut lines, &mut emit, line);
                }
            }
            Err(err) => handle_manifest_refresh_failure(action, &mut lines, &mut emit, err)?,
        }
    }
    let Some(target_packages) = manifest::latest_toolchain_target_from_cached_manifest(
        &manifest_root,
        native_host_arch(),
        &selected_target_arch,
    ) else {
        return Err(BackendError::Other(
            "failed to determine latest MSVC toolchain target from cached manifest".to_string(),
        ));
    };
    if action == ToolchainAction::Update
        && managed_toolchain_is_current(tool_root, &target_packages)
    {
        push_stream_line(
            &mut lines,
            &mut emit,
            format!(
                "Managed MSVC toolchain is already up to date: {}",
                user_facing_toolchain_label(&target_packages.label())
            ),
        );
        return Ok(MsvcOperationOutcome {
            kind: "msvc_operation",
            runtime: MsvcRuntimeKind::Managed,
            operation: action.operation_kind(),
            title: action.title().to_string(),
            status: crate::CommandStatus::Success,
            output: lines,
            streamed: false,
        });
    }
    push_stream_line(
        &mut lines,
        &mut emit,
        format!(
            "Selected target from cached manifest: {}",
            target_packages.label()
        ),
    );
    let Some(payloads) = manifest::selected_payloads_from_cached_manifest(
        &manifest_root,
        &target_packages,
        native_host_arch(),
        &selected_target_arch,
    ) else {
        return Err(BackendError::Other(format!(
            "payload plan is not available yet for {}; refresh the cached manifest first",
            target_packages.label()
        )));
    };
    for line in ensure_cached_payloads(
        tool_root,
        &target_packages,
        &payloads,
        proxy,
        cancel,
        &mut emit,
    )
    .await?
    {
        push_stream_line(&mut lines, &mut emit, line);
    }
    for line in ensure_msi_media_metadata(tool_root, &payloads)? {
        check_token_cancel(cancel)?;
        push_stream_line(&mut lines, &mut emit, line);
    }
    for line in
        ensure_cached_companion_cabs(tool_root, &target_packages, &payloads, proxy, &mut emit)
            .await?
    {
        check_token_cancel(cancel)?;
        push_stream_line(&mut lines, &mut emit, line);
    }
    for line in ensure_staged_external_cabs(tool_root, &payloads)? {
        check_token_cancel(cancel)?;
        push_stream_line(&mut lines, &mut emit, line);
    }
    for line in ensure_extracted_msis(tool_root, &payloads, &mut emit)? {
        check_token_cancel(cancel)?;
        push_stream_line(&mut lines, &mut emit, line);
    }
    for line in ensure_extracted_archives(tool_root, &payloads)? {
        check_token_cancel(cancel)?;
        push_stream_line(&mut lines, &mut emit, line);
    }
    for line in ensure_install_image(tool_root, &payloads)? {
        check_token_cancel(cancel)?;
        push_stream_line(&mut lines, &mut emit, line);
    }
    for line in ensure_materialized_toolchain(tool_root, &target_packages)? {
        check_token_cancel(cancel)?;
        push_stream_line(&mut lines, &mut emit, line);
    }
    for line in cleanup_post_install_cache(tool_root) {
        check_token_cancel(cancel)?;
        push_stream_line(&mut lines, &mut emit, line);
    }
    for line in write_runtime_state(tool_root)? {
        check_token_cancel(cancel)?;
        push_stream_line(&mut lines, &mut emit, line);
    }
    for line in remove_autoenv_dir(tool_root)? {
        check_token_cancel(cancel)?;
        push_stream_line(&mut lines, &mut emit, line);
    }
    push_stream_line(
        &mut lines,
        &mut emit,
        format!(
            "Selected {} payloads from cached manifest for installation.",
            payloads.len()
        ),
    );
    push_stream_line(
        &mut lines,
        &mut emit,
        format!(
            "Installed latest MSVC toolchain target directly with spoon: {} + {}",
            target_packages.msvc, target_packages.sdk
        ),
    );
    write_installed_state(tool_root, &target_packages)?;
    match managed_toolchain_flags_with_request(request).await {
        Ok(wrapper_flags) => {
            for line in
                write_managed_toolchain_wrappers(tool_root, command_profile, &wrapper_flags)?
            {
                check_token_cancel(cancel)?;
                push_stream_line(&mut lines, &mut emit, line);
            }
        }
        Err(err) => {
            push_stream_line(
                &mut lines,
                &mut emit,
                format!("Skipped managed wrapper generation: {err}"),
            );
        }
    }
    push_stream_line(
        &mut lines,
        &mut emit,
        format!(
            "Managed wrappers are materialized under {}.",
            paths::shims_root(tool_root).display()
        ),
    );

    Ok(MsvcOperationOutcome {
        kind: "msvc_operation",
        runtime: MsvcRuntimeKind::Managed,
        operation: action.operation_kind(),
        title: action.title().to_string(),
        status: crate::CommandStatus::Success,
        output: lines,
        streamed: false,
    })
}

pub async fn install_toolchain_async(tool_root: &Path) -> Result<MsvcOperationOutcome> {
    let request = MsvcRequest::for_tool_root(tool_root);
    run_toolchain_action_async(&request, ToolchainAction::Install, None, None).await
}

pub async fn update_toolchain_async(tool_root: &Path) -> Result<MsvcOperationOutcome> {
    let request = MsvcRequest::for_tool_root(tool_root);
    run_toolchain_action_async(&request, ToolchainAction::Update, None, None).await
}

pub async fn install_toolchain_async_with_context<P>(
    context: &BackendContext<P>,
) -> Result<MsvcOperationOutcome> {
    let request = MsvcRequest::from_context(context);
    run_toolchain_action_async(&request, ToolchainAction::Install, None, None).await
}

pub async fn update_toolchain_async_with_context<P>(
    context: &BackendContext<P>,
) -> Result<MsvcOperationOutcome> {
    let request = MsvcRequest::from_context(context);
    run_toolchain_action_async(&request, ToolchainAction::Update, None, None).await
}

pub async fn managed_toolchain_flags_with_context<P>(
    context: &BackendContext<P>,
) -> Result<ToolchainFlags> {
    let request = MsvcRequest::from_context(context);
    managed_toolchain_flags_with_request(&request).await
}

pub async fn install_toolchain_async_streaming<F>(
    tool_root: &Path,
    emit: &mut F,
) -> Result<MsvcOperationOutcome>
where
    F: FnMut(BackendEvent),
{
    let request = MsvcRequest::for_tool_root(tool_root);
    let mut callback = emit as &mut dyn FnMut(BackendEvent);
    let mut result = run_toolchain_action_async(
        &request,
        ToolchainAction::Install,
        None,
        Some(&mut callback),
    )
    .await?;
    result.streamed = true;
    Ok(result)
}

pub async fn update_toolchain_async_streaming<F>(
    tool_root: &Path,
    emit: &mut F,
) -> Result<MsvcOperationOutcome>
where
    F: FnMut(BackendEvent),
{
    let request = MsvcRequest::for_tool_root(tool_root);
    let mut callback = emit as &mut dyn FnMut(BackendEvent);
    let mut result =
        run_toolchain_action_async(&request, ToolchainAction::Update, None, Some(&mut callback))
            .await?;
    result.streamed = true;
    Ok(result)
}

pub async fn install_toolchain_streaming<F>(
    tool_root: &Path,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<MsvcOperationOutcome>
where
    F: FnMut(BackendEvent),
{
    let request = MsvcRequest::for_tool_root(tool_root);
    let mut callback = emit as &mut dyn FnMut(BackendEvent);
    let mut result = run_toolchain_action_async(
        &request,
        ToolchainAction::Install,
        cancel,
        Some(&mut callback),
    )
    .await?;
    result.streamed = true;
    Ok(result)
}

pub async fn update_toolchain_streaming<F>(
    tool_root: &Path,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<MsvcOperationOutcome>
where
    F: FnMut(BackendEvent),
{
    let request = MsvcRequest::for_tool_root(tool_root);
    let mut callback = emit as &mut dyn FnMut(BackendEvent);
    let mut result = run_toolchain_action_async(
        &request,
        ToolchainAction::Update,
        cancel,
        Some(&mut callback),
    )
    .await?;
    result.streamed = true;
    Ok(result)
}

pub async fn install_toolchain_streaming_with_context<P, F>(
    context: &BackendContext<P>,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<MsvcOperationOutcome>
where
    F: FnMut(BackendEvent),
{
    let request = MsvcRequest::from_context(context);
    let mut callback = emit as &mut dyn FnMut(BackendEvent);
    let mut result = run_toolchain_action_async(
        &request,
        ToolchainAction::Install,
        cancel,
        Some(&mut callback),
    )
    .await?;
    result.streamed = true;
    Ok(result)
}

pub async fn update_toolchain_streaming_with_context<P, F>(
    context: &BackendContext<P>,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<MsvcOperationOutcome>
where
    F: FnMut(BackendEvent),
{
    let request = MsvcRequest::from_context(context);
    let mut callback = emit as &mut dyn FnMut(BackendEvent);
    let mut result = run_toolchain_action_async(
        &request,
        ToolchainAction::Update,
        cancel,
        Some(&mut callback),
    )
    .await?;
    result.streamed = true;
    Ok(result)
}

pub async fn uninstall_toolchain(tool_root: &Path) -> Result<MsvcOperationOutcome> {
    let target = msvc_dir(tool_root);
    let state_root = paths::msvc_state_root(tool_root);
    let mut lines = vec![format!("> remove MSVC toolchain at {}", target.display())];
    lines.extend(remove_managed_toolchain_wrappers(tool_root)?);

    if target.exists() {
        std::fs::remove_dir_all(&target)
            .map_err(|err| BackendError::fs("remove", &target, err))?;
        lines.push("Removed toolchain directory.".to_string());
    } else {
        lines.push("Toolchain directory not present; nothing to remove.".to_string());
    }
    if state_root.exists() {
        std::fs::remove_dir_all(&state_root)
            .map_err(|err| BackendError::fs("remove", &state_root, err))?;
        lines.push("Removed managed state directory.".to_string());
    }

    lines.push(format!(
        "Managed MSVC cache is retained at {}",
        paths::msvc_cache_root(tool_root).display()
    ));

    Ok(MsvcOperationOutcome {
        kind: "msvc_operation",
        runtime: MsvcRuntimeKind::Managed,
        operation: MsvcOperationKind::Uninstall,
        title: "uninstall MSVC Toolchain".to_string(),
        status: crate::CommandStatus::Success,
        output: lines,
        streamed: false,
    })
}

pub async fn uninstall_toolchain_with_context<P>(
    context: &BackendContext<P>,
) -> Result<MsvcOperationOutcome> {
    uninstall_toolchain(&context.root).await
}
