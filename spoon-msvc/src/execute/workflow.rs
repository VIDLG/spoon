//! Workflow orchestration — install/update/uninstall lifecycle, state management.

use std::path::Path;

use fs_err as fs;
use walkdir::WalkDir;

use spoon_core::{CoreError, Result, format_bytes};

use crate::common::emit_notice;
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
    emit: Option<&spoon_core::EventSender>,
    err: spoon_core::CoreError,
) -> spoon_core::Result<()> {
    if action == ToolchainAction::Update {
        return Err(spoon_core::CoreError::Other(format!(
            "failed to refresh latest managed MSVC manifest for update: {err}"
        )));
    }
    emit_notice(
        emit,
        &format!("Warning: failed to refresh managed MSVC manifest cache: {err}"),
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

fn write_runtime_state(tool_root: &Path, emit: Option<&spoon_core::EventSender>) -> Result<()> {
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
    emit_notice(
        emit,
        &format!(
            "Wrote managed runtime state into {}.",
            runtime_state.display()
        ),
    );
    Ok(())
}

fn remove_autoenv_dir(tool_root: &Path, emit: Option<&spoon_core::EventSender>) -> Result<()> {
    let autoenv_root = pipeline::msvc_dir(tool_root).join("autoenv");
    if !autoenv_root.exists() {
        return Ok(());
    }
    fs::remove_dir_all(&autoenv_root)
        .map_err(|e| CoreError::fs("remove_dir_all", &autoenv_root, e))?;
    emit_notice(
        emit,
        &format!("Removed autoenv directory {}.", autoenv_root.display()),
    );
    Ok(())
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
    emit: Option<&spoon_core::EventSender>,
) -> Result<()> {
    let image_root = pipeline::install_image_cache_dir(tool_root);
    if !image_root.exists() {
        emit_notice(emit, "Install image not present yet; skipped toolchain materialization.");
        return Ok(());
    }

    let toolchain_root = pipeline::msvc_dir(tool_root);
    fs::create_dir_all(&toolchain_root)
        .map_err(|e| CoreError::fs("create_dir_all", &toolchain_root, e))?;

    let before = crate::status::count_files_recursively(&toolchain_root);
    let copied = pipeline::copy_tree_into(&image_root, &toolchain_root)?;
    let after = crate::status::count_files_recursively(&toolchain_root);
    let reused = usize::from(after == before);
    write_installed_state(tool_root, target)?;

    emit_notice(
        emit,
        &format!(
            "Materialized managed toolchain image into {} (copied {}, reused {}).",
            toolchain_root.display(),
            copied,
            reused
        ),
    );
    Ok(())
}

pub fn cleanup_post_install_cache(tool_root: &Path, emit: Option<&spoon_core::EventSender>) {
    let cache_root = paths::msvc_cache_root(tool_root);
    let cleanup_targets = [cache_root.join("image")];
    let mut removed = 0_usize;
    let mut freed_bytes = 0_u64;

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
            Err(err) => {
                emit_notice(
                    emit,
                    &format!(
                        "Warning: failed to remove transient MSVC cache dir {}: {err}",
                        dir.display()
                    ),
                );
            }
        }
    }

    emit_notice(
        emit,
        &format!(
            "Cleaned transient MSVC install-image cache after install (removed {}, freed {}).",
            removed,
            format_bytes(freed_bytes)
        ),
    );
    emit_notice(
        emit,
        &format!(
            "Retained MSI extraction cache under {} for reuse.",
            cache_root.join("expanded").display()
        ),
    );
    emit_notice(
        emit,
        &format!(
            "Retained MSI staging cache under {} for reuse.",
            cache_root.join("stage").display()
        ),
    );
}

// ---------------------------------------------------------------------------
// Top-level async workflow
// ---------------------------------------------------------------------------

async fn run_toolchain_action_async(
    request: &crate::types::MsvcRequest,
    action: ToolchainAction,
    cancel: Option<&spoon_core::CancellationToken>,
    emit: Option<&spoon_core::EventSender>,
) -> spoon_core::Result<crate::types::MsvcOperationOutcome> {
    let action_kind = action.operation_kind();
    let action_title = action.title().to_string();
    spoon_core::check_token_cancel(cancel)?;
    let tool_root = request.root.as_path();
    let proxy = request.proxy.as_str();
    let selected_target_arch = request.normalized_target_arch();
    let manifest_root = pipeline::manifest_dir(tool_root);
    if !request.test_mode {
        match crate::facts::manifest::sync_release_manifest_cache_async(&manifest_root, proxy).await {
            Ok(sync_lines) => {
                for line in sync_lines {
                    emit_notice(emit, &line);
                }
            }
            Err(err) => handle_manifest_refresh_failure(action, emit, err)?,
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
        emit_notice(
            emit,
            &format!(
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
        });
    }
    emit_notice(
        emit,
        &format!(
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
    pipeline::ensure_cached_payloads(
        tool_root,
        &target_packages,
        &payloads,
        proxy,
        cancel,
        emit,
    )
    .await?;
    pipeline::ensure_msi_media_metadata(tool_root, &payloads, emit)?;
    spoon_core::check_token_cancel(cancel)?;
    pipeline::ensure_cached_companion_cabs(tool_root, &target_packages, &payloads, proxy, emit).await?;
    spoon_core::check_token_cancel(cancel)?;
    pipeline::ensure_staged_external_cabs(tool_root, &payloads, emit)?;
    spoon_core::check_token_cancel(cancel)?;
    pipeline::ensure_extracted_msis(tool_root, &payloads, emit)?;
    spoon_core::check_token_cancel(cancel)?;
    pipeline::ensure_extracted_archives(tool_root, &payloads, emit)?;
    spoon_core::check_token_cancel(cancel)?;
    pipeline::ensure_install_image(tool_root, &payloads, emit)?;
    spoon_core::check_token_cancel(cancel)?;
    ensure_materialized_toolchain(tool_root, &target_packages, emit)?;
    spoon_core::check_token_cancel(cancel)?;
    cleanup_post_install_cache(tool_root, emit);
    spoon_core::check_token_cancel(cancel)?;
    write_runtime_state(tool_root, emit)?;
    spoon_core::check_token_cancel(cancel)?;
    remove_autoenv_dir(tool_root, emit)?;
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
                emit_notice(emit, &line);
            }
        }
        Err(err) => {
            emit_notice(
                emit,
                &format!("Skipped managed wrapper generation: {err}"),
            );
        }
    }
    emit_notice(
        emit,
        &format!(
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
    })
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

pub async fn install_toolchain(
    request: &crate::types::MsvcRequest,
    cancel: Option<&spoon_core::CancellationToken>,
    emit: Option<&spoon_core::EventSender>,
) -> spoon_core::Result<crate::types::MsvcOperationOutcome> {
    run_toolchain_action_async(request, ToolchainAction::Install, cancel, emit).await
}

pub async fn update_toolchain(
    request: &crate::types::MsvcRequest,
    cancel: Option<&spoon_core::CancellationToken>,
    emit: Option<&spoon_core::EventSender>,
) -> spoon_core::Result<crate::types::MsvcOperationOutcome> {
    run_toolchain_action_async(request, ToolchainAction::Update, cancel, emit).await
}

pub async fn uninstall_toolchain(
    request: &crate::types::MsvcRequest,
    _cancel: Option<&spoon_core::CancellationToken>,
    emit: Option<&spoon_core::EventSender>,
) -> spoon_core::Result<crate::types::MsvcOperationOutcome> {
    let tool_root = request.root.as_path();
    let target = pipeline::msvc_dir(tool_root);
    let state_root = paths::msvc_state_root(tool_root);
    emit_notice(emit, &format!("> remove MSVC toolchain at {}", target.display()));
    for line in wrappers::remove_managed_toolchain_wrappers(tool_root)? {
        emit_notice(emit, &line);
    }

    if target.exists() {
        std::fs::remove_dir_all(&target)
            .map_err(|err| spoon_core::CoreError::fs("remove", &target, err))?;
        emit_notice(emit, "Removed toolchain directory.");
    } else {
        emit_notice(emit, "Toolchain directory not present; nothing to remove.");
    }
    if state_root.exists() {
        std::fs::remove_dir_all(&state_root)
            .map_err(|err| spoon_core::CoreError::fs("remove", &state_root, err))?;
        emit_notice(emit, "Removed managed state directory.");
    }

    emit_notice(
        emit,
        &format!(
            "Managed MSVC cache is retained at {}",
            paths::msvc_cache_root(tool_root).display()
        ),
    );
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
    })
}
