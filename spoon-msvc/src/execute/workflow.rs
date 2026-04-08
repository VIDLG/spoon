//! Workflow orchestration — install/update/uninstall lifecycle, state management.

use std::path::Path;

use fs_err as fs;
use walkdir::WalkDir;

use spoon_core::{CoreError, Result, format_bytes};

use crate::facts::manifest;
use crate::paths;
use crate::rules::write_installed_toolchain_target;
use crate::wrappers;

use super::discover;
use super::pipeline;

// ---------------------------------------------------------------------------
// ToolchainAction
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum ToolchainAction {
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

    fn operation_kind(self) -> crate::types::MsvcOperationKind {
        match self {
            Self::Install => crate::types::MsvcOperationKind::Install,
            Self::Update => crate::types::MsvcOperationKind::Update,
        }
    }
}

// ---------------------------------------------------------------------------
// State helpers
// ---------------------------------------------------------------------------

fn handle_manifest_refresh_failure(
    action: ToolchainAction,
    lines: &mut Vec<String>,
    _emit: &mut Option<&mut dyn FnMut(spoon_core::SpoonEvent)>,
    err: spoon_core::CoreError,
) -> spoon_core::Result<()> {
    if action == ToolchainAction::Update {
        return Err(spoon_core::CoreError::Other(format!(
            "failed to refresh latest managed MSVC manifest for update: {err}"
        )));
    }
    crate::common::push_stream_line(
        lines,
        &mut None,
        format!("Warning: failed to refresh managed MSVC manifest cache: {err}"),
    );
    Ok(())
}

fn managed_toolchain_is_current(tool_root: &Path, latest: &crate::facts::manifest::ToolchainTarget) -> bool {
    paths::msvc_toolchain_root(tool_root).exists()
        && pipeline::runtime_state_path(tool_root).exists()
        && crate::rules::read_installed_toolchain_target(&paths::msvc_root(tool_root))
            .is_some_and(|installed| installed.msvc == latest.msvc && installed.sdk == latest.sdk)
}

fn write_managed_canonical_state(
    request: &crate::types::MsvcRequest,
    operation: crate::types::MsvcOperationKind,
    installed: bool,
    version: Option<String>,
    sdk_version: Option<String>,
    validation_status: Option<crate::types::MsvcValidationStatus>,
    validation_message: Option<String>,
) -> crate::state::Result<()> {
    let layout = spoon_core::RuntimeLayout::from_root(&request.root);
    let previous = crate::state::read_canonical_state(&layout);
    let state = crate::state::MsvcCanonicalState {
        runtime_kind: crate::types::MsvcRuntimeKind::Managed,
        installed,
        version,
        sdk_version,
        last_operation: Some(operation),
        last_stage: Some(crate::types::MsvcLifecycleStage::Completed),
        validation_status: validation_status.or_else(|| {
            previous.as_ref().and_then(|state| state.validation_status.clone())
        }),
        validation_message: validation_message
            .or_else(|| previous.as_ref().and_then(|state| state.validation_message.clone())),
        managed: crate::state::ManagedMsvcStateDetail {
            selected_target_arch: Some(request.normalized_target_arch()),
        },
        official: previous.map(|state| state.official).unwrap_or_default(),
    };
    crate::state::write_canonical_state(&layout, &state)
}

fn write_installed_state(
    tool_root: &Path,
    target: &manifest::ToolchainTarget,
) -> Result<()> {
    let managed_root = paths::msvc_root(tool_root);
    let rules_target = crate::facts::rules::ToolchainTarget {
        msvc: target.msvc.clone(),
        sdk: target.sdk.clone(),
    };
    write_installed_toolchain_target(&managed_root, &rules_target)
        .map_err(|e| CoreError::Other(format!(
            "failed to write installed MSVC state under {}: {e}",
            managed_root.display()
        )))?;
    Ok(())
}

fn write_runtime_state(tool_root: &Path) -> Result<Vec<String>> {
    let state_root = paths::msvc_state_root(tool_root);
    fs::create_dir_all(&state_root)
        .map_err(|e| CoreError::fs("create_dir_all", &state_root, e))?;
    let runtime_state = pipeline::runtime_state_path(tool_root);
    fs::write(
        &runtime_state,
        serde_json::to_string_pretty(&serde_json::json!({
            "toolchain_root": pipeline::msvc_dir(tool_root),
            "wrappers_root": paths::shims_root(tool_root),
            "runtime": "managed"
        }))
        .map_err(|e| CoreError::Other(e.to_string()))?
        .as_bytes(),
    )
    .map_err(|e| CoreError::fs("write", &runtime_state, e))?;
    Ok(vec![format!(
        "Wrote managed runtime state into {}.",
        runtime_state.display()
    )])
}

fn remove_autoenv_dir(tool_root: &Path) -> Result<Vec<String>> {
    let autoenv_root = pipeline::msvc_dir(tool_root).join("autoenv");
    if !autoenv_root.exists() {
        return Ok(Vec::new());
    }
    fs::remove_dir_all(&autoenv_root)
        .map_err(|e| CoreError::fs("remove_dir_all", &autoenv_root, e))?;
    Ok(vec![format!(
        "Removed autoenv directory {}.",
        autoenv_root.display()
    )])
}

fn dir_size_bytes(root: &Path) -> Option<u64> {
    let mut total = 0_u64;
    for entry in WalkDir::new(root) {
        let entry = entry.ok()?;
        if !entry.file_type().is_file() {
            continue;
        }
        total = total.saturating_add(entry.metadata().ok()?.len());
    }
    Some(total)
}

fn user_facing_toolchain_label(raw: &str) -> String {
    raw.replace("msvc-", "").replace("sdk-", "")
}

// ---------------------------------------------------------------------------
// Materialization
// ---------------------------------------------------------------------------

pub fn ensure_materialized_toolchain(
    tool_root: &Path,
    target: &manifest::ToolchainTarget,
) -> Result<Vec<String>> {
    let image_root = pipeline::install_image_cache_dir(tool_root);
    if !image_root.exists() {
        return Ok(vec![
            "Install image not present yet; skipped toolchain materialization.".to_string(),
        ]);
    }

    let toolchain_root = pipeline::msvc_dir(tool_root);
    fs::create_dir_all(&toolchain_root)
        .map_err(|e| CoreError::fs("create_dir_all", &toolchain_root, e))?;

    let before = crate::status::count_files_recursively(&toolchain_root);
    let copied = pipeline::copy_tree_into(&image_root, &toolchain_root)?;
    let after = crate::status::count_files_recursively(&toolchain_root);
    let reused = usize::from(after == before);
    write_installed_state(tool_root, target)?;

    Ok(vec![format!(
        "Materialized managed toolchain image into {} (copied {}, reused {}).",
        toolchain_root.display(),
        copied,
        reused
    )])
}

pub fn cleanup_post_install_cache(tool_root: &Path) -> Vec<String> {
    let cache_root = paths::msvc_cache_root(tool_root);
    let cleanup_targets = [cache_root.join("image")];
    let mut removed = 0_usize;
    let mut freed_bytes = 0_u64;
    let mut warnings = Vec::new();

    for dir in cleanup_targets {
        if !dir.exists() {
            continue;
        }
        let bytes = dir_size_bytes(&dir).unwrap_or(0);
        match fs::remove_dir_all(&dir) {
            Ok(()) => {
                removed += 1;
                freed_bytes += bytes;
            }
            Err(err) => warnings.push(format!(
                "Warning: failed to remove transient MSVC cache dir {}: {err}",
                dir.display()
            )),
        }
    }

    let mut lines = vec![format!(
        "Cleaned transient MSVC install-image cache after install (removed {}, freed {}).",
        removed,
        format_bytes(freed_bytes)
    )];
    lines.push(format!(
        "Retained MSI extraction cache under {} for reuse.",
        cache_root.join("expanded").display()
    ));
    lines.push(format!(
        "Retained MSI staging cache under {} for reuse.",
        cache_root.join("stage").display()
    ));
    lines.extend(warnings);
    lines
}

// ---------------------------------------------------------------------------
// Top-level async workflow
// ---------------------------------------------------------------------------

async fn run_toolchain_action_async(
    request: &crate::types::MsvcRequest,
    action: ToolchainAction,
    cancel: Option<&spoon_core::CancellationToken>,
    mut emit: Option<&mut dyn FnMut(spoon_core::SpoonEvent)>,
) -> spoon_core::Result<crate::types::MsvcOperationOutcome> {
    let action_kind = action.operation_kind();
    let action_title = action.title().to_string();
    spoon_core::check_token_cancel(cancel)?;
    let tool_root = request.root.as_path();
    let proxy = request.proxy.as_str();
    let selected_target_arch = request.normalized_target_arch();
    let mut lines = Vec::new();
    let manifest_root = pipeline::manifest_dir(tool_root);
    if !request.test_mode {
        match crate::facts::manifest::sync_release_manifest_cache_async(&manifest_root, proxy).await {
            Ok(sync_lines) => {
                for line in sync_lines {
                    crate::common::push_stream_line(&mut lines, &mut emit, line);
                }
            }
            Err(err) => handle_manifest_refresh_failure(action, &mut lines, &mut emit, err)?,
        }
    }
    let Some(target_packages) = crate::facts::manifest::latest_toolchain_target_from_cached_manifest(
        &manifest_root,
        pipeline::native_host_arch(),
        &selected_target_arch,
    ) else {
        return Err(spoon_core::CoreError::Other(
            "failed to determine latest MSVC toolchain target from cached manifest".to_string(),
        ));
    };
    if action == ToolchainAction::Update
        && managed_toolchain_is_current(tool_root, &target_packages)
    {
        write_managed_canonical_state(
            request,
            action_kind,
            true,
            Some(user_facing_toolchain_label(&target_packages.msvc)),
            Some(user_facing_toolchain_label(&target_packages.sdk)),
            None,
            None,
        )
        .map_err(|e| spoon_core::CoreError::Other(e.to_string()))?;
        crate::common::push_stream_line(
            &mut lines,
            &mut emit,
            format!(
                "Managed MSVC toolchain is already up to date: {}",
                user_facing_toolchain_label(&target_packages.label())
            ),
        );
        return Ok(crate::types::MsvcOperationOutcome {
            kind: "msvc_operation",
            runtime: crate::types::MsvcRuntimeKind::Managed,
            operation: action_kind,
            title: action_title,
            status: true,
            output: lines,
            streamed: false,
        });
    }
    crate::common::push_stream_line(
        &mut lines,
        &mut emit,
        format!(
            "Selected target from cached manifest: {}",
            target_packages.label()
        ),
    );
    let Some(payloads) = crate::facts::manifest::selected_payloads_from_cached_manifest(
        &manifest_root,
        &target_packages,
        pipeline::native_host_arch(),
        &selected_target_arch,
    ) else {
        return Err(spoon_core::CoreError::Other(format!(
            "payload plan is not available yet for {}; refresh the cached manifest first",
            target_packages.label()
        )));
    };
    for line in pipeline::ensure_cached_payloads(
        tool_root,
        &target_packages,
        &payloads,
        proxy,
        cancel,
        &mut emit,
    )
    .await?
    {
        crate::common::push_stream_line(&mut lines, &mut emit, line);
    }
    for line in pipeline::ensure_msi_media_metadata(tool_root, &payloads)? {
        spoon_core::check_token_cancel(cancel)?;
        crate::common::push_stream_line(&mut lines, &mut emit, line);
    }
    for line in
        pipeline::ensure_cached_companion_cabs(tool_root, &target_packages, &payloads, proxy, &mut emit)
            .await?
    {
        spoon_core::check_token_cancel(cancel)?;
        crate::common::push_stream_line(&mut lines, &mut emit, line);
    }
    for line in pipeline::ensure_staged_external_cabs(tool_root, &payloads)? {
        spoon_core::check_token_cancel(cancel)?;
        crate::common::push_stream_line(&mut lines, &mut emit, line);
    }
    for line in pipeline::ensure_extracted_msis(tool_root, &payloads, &mut emit)? {
        spoon_core::check_token_cancel(cancel)?;
        crate::common::push_stream_line(&mut lines, &mut emit, line);
    }
    for line in pipeline::ensure_extracted_archives(tool_root, &payloads)? {
        spoon_core::check_token_cancel(cancel)?;
        crate::common::push_stream_line(&mut lines, &mut emit, line);
    }
    for line in pipeline::ensure_install_image(tool_root, &payloads)? {
        spoon_core::check_token_cancel(cancel)?;
        crate::common::push_stream_line(&mut lines, &mut emit, line);
    }
    for line in ensure_materialized_toolchain(tool_root, &target_packages)? {
        spoon_core::check_token_cancel(cancel)?;
        crate::common::push_stream_line(&mut lines, &mut emit, line);
    }
    for line in cleanup_post_install_cache(tool_root) {
        spoon_core::check_token_cancel(cancel)?;
        crate::common::push_stream_line(&mut lines, &mut emit, line);
    }
    for line in write_runtime_state(tool_root)? {
        spoon_core::check_token_cancel(cancel)?;
        crate::common::push_stream_line(&mut lines, &mut emit, line);
    }
    for line in remove_autoenv_dir(tool_root)? {
        spoon_core::check_token_cancel(cancel)?;
        crate::common::push_stream_line(&mut lines, &mut emit, line);
    }
    write_installed_state(tool_root, &target_packages)?;
    write_managed_canonical_state(
        request,
        action_kind,
        true,
        Some(user_facing_toolchain_label(&target_packages.msvc)),
        Some(user_facing_toolchain_label(&target_packages.sdk)),
        None,
        None,
    )
    .map_err(|e| spoon_core::CoreError::Other(e.to_string()))?;
    match discover::managed_toolchain_flags_with_request(request).await {
        Ok(wrapper_flags) => {
            for line in
                wrappers::write_managed_toolchain_wrappers(tool_root, &request.command_profile, &wrapper_flags)?
            {
                spoon_core::check_token_cancel(cancel)?;
                crate::common::push_stream_line(&mut lines, &mut emit, line);
            }
        }
        Err(err) => {
            crate::common::push_stream_line(
                &mut lines,
                &mut emit,
                format!("Skipped managed wrapper generation: {err}"),
            );
        }
    }
    crate::common::push_stream_line(
        &mut lines,
        &mut emit,
        format!(
            "Managed wrappers are materialized under {}.",
            paths::shims_root(tool_root).display()
        ),
    );

    Ok(crate::types::MsvcOperationOutcome {
        kind: "msvc_operation",
        runtime: crate::types::MsvcRuntimeKind::Managed,
        operation: action_kind,
        title: action_title,
        status: true,
        output: lines,
        streamed: false,
    })
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

pub async fn install_toolchain_async(
    request: &crate::types::MsvcRequest,
) -> spoon_core::Result<crate::types::MsvcOperationOutcome> {
    run_toolchain_action_async(request, ToolchainAction::Install, None, None).await
}

pub async fn update_toolchain_async(
    request: &crate::types::MsvcRequest,
) -> spoon_core::Result<crate::types::MsvcOperationOutcome> {
    run_toolchain_action_async(request, ToolchainAction::Update, None, None).await
}

pub async fn install_toolchain_streaming<F>(
    request: &crate::types::MsvcRequest,
    cancel: Option<&spoon_core::CancellationToken>,
    emit: &mut F,
) -> spoon_core::Result<crate::types::MsvcOperationOutcome>
where
    F: FnMut(spoon_core::SpoonEvent),
{
    let mut callback = emit as &mut dyn FnMut(spoon_core::SpoonEvent);
    let mut result = run_toolchain_action_async(
        request,
        ToolchainAction::Install,
        cancel,
        Some(&mut callback),
    )
    .await?;
    result.streamed = true;
    Ok(result)
}

pub async fn update_toolchain_streaming<F>(
    request: &crate::types::MsvcRequest,
    cancel: Option<&spoon_core::CancellationToken>,
    emit: &mut F,
) -> spoon_core::Result<crate::types::MsvcOperationOutcome>
where
    F: FnMut(spoon_core::SpoonEvent),
{
    let mut callback = emit as &mut dyn FnMut(spoon_core::SpoonEvent);
    let mut result = run_toolchain_action_async(
        request,
        ToolchainAction::Update,
        cancel,
        Some(&mut callback),
    )
    .await?;
    result.streamed = true;
    Ok(result)
}

pub async fn uninstall_toolchain_async(
    request: &crate::types::MsvcRequest,
) -> spoon_core::Result<crate::types::MsvcOperationOutcome> {
    let tool_root = request.root.as_path();
    let target = pipeline::msvc_dir(tool_root);
    let state_root = paths::msvc_state_root(tool_root);
    let mut lines = vec![format!("> remove MSVC toolchain at {}", target.display())];
    lines.extend(wrappers::remove_managed_toolchain_wrappers(tool_root)?);

    if target.exists() {
        std::fs::remove_dir_all(&target)
            .map_err(|err| spoon_core::CoreError::fs("remove", &target, err))?;
        lines.push("Removed toolchain directory.".to_string());
    } else {
        lines.push("Toolchain directory not present; nothing to remove.".to_string());
    }
    if state_root.exists() {
        std::fs::remove_dir_all(&state_root)
            .map_err(|err| spoon_core::CoreError::fs("remove", &state_root, err))?;
        lines.push("Removed managed state directory.".to_string());
    }

    lines.push(format!(
        "Managed MSVC cache is retained at {}",
        paths::msvc_cache_root(tool_root).display()
    ));
    write_managed_canonical_state(
        request,
        crate::types::MsvcOperationKind::Uninstall,
        false,
        None,
        None,
        None,
        None,
    )
    .map_err(|e| spoon_core::CoreError::Other(e.to_string()))?;

    Ok(crate::types::MsvcOperationOutcome {
        kind: "msvc_operation",
        runtime: crate::types::MsvcRuntimeKind::Managed,
        operation: crate::types::MsvcOperationKind::Uninstall,
        title: "uninstall MSVC Toolchain".to_string(),
        status: true,
        output: lines,
        streamed: false,
    })
}
