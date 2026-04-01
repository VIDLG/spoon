mod cache;
mod common;
mod detect;
mod execute;
mod manifest;
mod msi_extract;
pub mod official;
mod package_rules;
pub mod paths;
mod plan;
mod query;
mod rules;
mod status;
mod validation;
mod wrappers;

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use fs_err as fs;
use reqwest::Client;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::{
    BackendError, BackendEvent, CancellationToken, ProgressEvent, Result, check_token_cancel,
    event::progress_kind,
};

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "K", "M", "G", "T"];

    if bytes < 1024 {
        return format!("{bytes}B");
    }

    let mut value = bytes as f64;
    let mut unit_index = 0usize;
    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    format!("{value:.1}{}", UNITS[unit_index])
}

pub use self::cache::{clear as clear_cache, prune as prune_cache};
pub use self::common::{
    find_all_named_files, find_first_named_file, http_client, http_client_with_redirect,
    join_windows_path, normalize_proxy_url, path_components_lowercase, push_stream_line,
    unique_existing_dirs,
};
pub use self::manifest::{
    Payload, SelectedPayload, ToolchainTarget as ManifestToolchainTarget,
    companion_cab_payloads_for_selected_msi_from_cached_manifest,
    latest_toolchain_target_from_cached_manifest, selected_payloads_from_cached_manifest,
    sync_release_manifest_cache_async,
};
pub use self::msi_extract::{extract_msi_with_staged_cabs, read_msi_cab_names};
pub use self::package_rules::{
    ArchiveKind, ManagedPackageKind, ManifestPackageId, PayloadKind, archive_kind,
    identify_manifest_package_id, identify_payload, manifest_package_matches_msvc_target,
    normalize_msvc_build_version, package_kind, sdk_payload_matches_target,
};
pub use self::query::{
    MsvcIntegration, MsvcStatus, installed_toolchain_version_label,
    latest_toolchain_version_label, latest_toolchain_version_label_with_context, status,
    status_with_context, user_facing_toolchain_label,
};
pub use self::paths::{
    msvc_cache_root, msvc_manifest_root, msvc_root, msvc_state_root, msvc_toolchain_root,
    native_msvc_arch, official_msvc_cache_root, official_msvc_root, official_msvc_state_root,
};
pub(crate) use self::plan::MsvcRequest;
pub use self::plan::{
    MsvcLifecycleStage, MsvcOperationKind, MsvcOperationOutcome, MsvcOperationRequest,
    MsvcRuntimeKind, MsvcRuntimePreference, ToolchainFlags,
};
pub use self::detect::{DetectedMsvcRuntime, MsvcRuntimeDetection, detect_runtimes, detect_runtimes_with_context};
pub use self::rules::{
    ToolchainTarget, installed_state_path, package_token_after_prefix,
    parse_toolchain_target_from_lines, pick_higher_version, read_installed_toolchain_target,
    select_latest_toolchain_from_packages, version_key, write_installed_toolchain_target,
};
pub use self::status::count_files_recursively;
pub use self::validation::{validate_toolchain, validate_toolchain_with_context};
pub(crate) use self::wrappers::managed_toolchain_flags_with_request;
pub use self::wrappers::{
    managed_toolchain_flags, reapply_managed_command_surface_streaming,
    remove_managed_toolchain_wrappers, write_managed_toolchain_wrappers,
};
pub use self::execute::{
    ToolchainAction, handle_manifest_refresh_failure, install_toolchain_async,
    install_toolchain_async_streaming, install_toolchain_async_with_context,
    install_toolchain_streaming, install_toolchain_streaming_with_context,
    managed_toolchain_flags_with_context, uninstall_toolchain, uninstall_toolchain_with_context,
    update_toolchain_async, update_toolchain_async_streaming, update_toolchain_async_with_context,
    update_toolchain_streaming, update_toolchain_streaming_with_context,
};

pub(super) fn external<T, E>(
    result: std::result::Result<T, E>,
    message: impl Into<String>,
) -> Result<T>
where
    E: std::error::Error + Send + Sync + 'static,
{
    result.map_err(|err| BackendError::external(message.into(), err))
}

pub(super) fn native_host_arch() -> &'static str {
    paths::native_msvc_arch()
}

fn find_preferred_msvc_binary(
    root: &Path,
    target_arch: &str,
    candidates: &[&str],
) -> Option<PathBuf> {
    let host_arch = native_host_arch().to_ascii_lowercase();
    let target_arch = target_arch.to_ascii_lowercase();
    let mut matches = WalkDir::new(root)
        .into_iter()
        .flatten()
        .filter(|entry| {
            entry.file_type().is_file()
                && candidates.iter().any(|candidate| {
                    entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| name.eq_ignore_ascii_case(candidate))
                })
        })
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();
    matches.sort_by_key(|path| {
        let lowered = path.display().to_string().to_ascii_lowercase();
        let host_target = format!("host{}\\{}", host_arch, target_arch);
        let host_native = format!("host{}\\", host_arch);
        (
            !lowered.contains(&host_target),
            !lowered.contains(&host_native),
            lowered,
        )
    });
    matches.into_iter().next()
}

fn is_target_arch_dir(path: &Path, target_arch: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case(target_arch))
}

pub(super) fn msvc_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_toolchain_root(tool_root)
}

pub fn runtime_state_path(tool_root: &Path) -> PathBuf {
    paths::msvc_state_root(tool_root).join("runtime.json")
}

pub(super) fn manifest_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_manifest_root(tool_root)
}

fn extract_zip_archive(archive_path: &Path, destination: &Path) -> Result<()> {
    let file = external(
        fs::File::open(archive_path),
        format!("failed to open {}", archive_path.display()),
    )?;
    let mut archive = external(
        ZipArchive::new(file),
        format!("invalid zip {}", archive_path.display()),
    )?;

    for index in 0..archive.len() {
        let mut entry = external(archive.by_index(index), "failed to read zip entry")?;
        let Some(relative_path) = entry.enclosed_name() else {
            continue;
        };
        let output_path = destination.join(relative_path);
        if entry.is_dir() {
            external(
                fs::create_dir_all(&output_path),
                format!("failed to create {}", output_path.display()),
            )?;
            continue;
        }
        if let Some(parent) = output_path.parent() {
            external(
                fs::create_dir_all(parent),
                format!("failed to create {}", parent.display()),
            )?;
        }
        let mut output = external(
            fs::File::create(&output_path),
            format!("failed to create {}", output_path.display()),
        )?;
        external(
            io::copy(&mut entry, &mut output),
            format!("failed to extract {}", output_path.display()),
        )?;
    }

    Ok(())
}

fn payload_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root).join("archives")
}

fn extracted_payload_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root)
        .join("expanded")
        .join("archives")
}

fn extracted_msi_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root)
        .join("expanded")
        .join("msi")
}

fn install_image_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root).join("image")
}

fn msi_metadata_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root)
        .join("metadata")
        .join("msi")
}

fn msi_staging_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root).join("stage").join("msi")
}

fn payload_cache_entry_name(payload: &manifest::SelectedPayload) -> String {
    let leaf = Path::new(&payload.payload.file_name)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("payload.bin");
    format!("{}-{}", payload.payload.sha256.to_ascii_lowercase(), leaf)
}

fn payload_cache_entry_path(tool_root: &Path, payload: &manifest::SelectedPayload) -> PathBuf {
    payload_cache_dir(tool_root).join(payload_cache_entry_name(payload))
}

fn extracted_payload_entry_dir(tool_root: &Path, payload: &manifest::SelectedPayload) -> PathBuf {
    extracted_payload_cache_dir(tool_root).join(payload.payload.sha256.to_ascii_lowercase())
}

fn msi_metadata_entry_path(tool_root: &Path, payload: &manifest::SelectedPayload) -> PathBuf {
    msi_metadata_cache_dir(tool_root).join(format!(
        "{}.txt",
        payload.payload.sha256.to_ascii_lowercase()
    ))
}

fn msi_staging_entry_dir(tool_root: &Path, payload: &manifest::SelectedPayload) -> PathBuf {
    msi_staging_cache_dir(tool_root).join(payload.payload.sha256.to_ascii_lowercase())
}

fn extracted_msi_entry_dir(tool_root: &Path, payload: &manifest::SelectedPayload) -> PathBuf {
    extracted_msi_cache_dir(tool_root).join(payload.payload.sha256.to_ascii_lowercase())
}

fn read_cached_msi_cab_names(
    tool_root: &Path,
    payload: &manifest::SelectedPayload,
) -> Option<Vec<String>> {
    let path = msi_metadata_entry_path(tool_root, payload);
    let content = fs::read_to_string(path).ok()?;
    Some(
        content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToString::to_string)
            .collect(),
    )
}

fn archive_kind_for_payload(payload: &manifest::SelectedPayload) -> Option<ArchiveKind> {
    archive_kind(&payload.payload.file_name).or_else(|| archive_kind(&payload.payload.url))
}

fn decode_hex_sha256(hex: &str) -> Result<[u8; 32]> {
    let trimmed = hex.trim();
    if trimmed.len() != 64 {
        return Err(BackendError::Other(format!(
            "expected 64 hex chars for sha256, got {}",
            trimmed.len()
        )));
    }
    let mut out = [0_u8; 32];
    for index in 0..32 {
        let start = index * 2;
        let byte = external(
            u8::from_str_radix(&trimmed[start..start + 2], 16),
            format!("invalid sha256 hex '{}'", trimmed),
        )?;
        out[index] = byte;
    }
    Ok(out)
}

fn file_sha256(path: &Path) -> Result<[u8; 32]> {
    let mut file = external(
        fs::File::open(path),
        format!("failed to open {}", path.display()),
    )?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 16 * 1024];
    loop {
        let read = external(
            file.read(&mut buf),
            format!("failed to read {}", path.display()),
        )?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(hasher.finalize().into())
}

fn payload_source_description(url: &str) -> String {
    if url.starts_with("file:///") {
        url.to_string()
    } else {
        Path::new(url)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| format!("remote payload {name}"))
            .unwrap_or_else(|| url.to_string())
    }
}

fn download_progress_target_label(file_name: &str) -> &str {
    Path::new(file_name)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(file_name)
}

async fn download_or_copy_payload(
    client: &Client,
    url: &str,
    destination: &Path,
    file_name: &str,
    cancel: Option<&CancellationToken>,
    emit: &mut Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<()> {
    check_token_cancel(cancel)?;
    if let Some(path) = url.strip_prefix("file:///") {
        external(
            fs::copy(path, destination),
            format!(
                "failed to copy local payload from {} to {}",
                path,
                destination.display()
            ),
        )?;
        return Ok(());
    }

    let path = Path::new(url);
    if path.exists() {
        external(
            fs::copy(path, destination),
            format!(
                "failed to copy local payload from {} to {}",
                path.display(),
                destination.display()
            ),
        )?;
        return Ok(());
    }

    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|err| BackendError::network(url, err))?
        .error_for_status()
        .map_err(|err| BackendError::network(url, err))?;
    let mut file = external(
        fs::File::create(destination),
        format!("failed to create {}", destination.display()),
    )?;
    let total_bytes = response.content_length();
    let mut downloaded_bytes = 0_u64;
    let mut last_emitted_percent = None;
    let mut last_emitted_mb_tenths = None;
    if let Some(total_bytes) = total_bytes {
        last_emitted_percent = Some(0);
        if let Some(callback) = emit.as_deref_mut() {
            callback(BackendEvent::Progress(ProgressEvent::bytes(
                progress_kind::DOWNLOAD,
                download_progress_target_label(file_name),
                0,
                Some(total_bytes),
            )));
        }
    } else if let Some(callback) = emit.as_deref_mut() {
        callback(BackendEvent::Progress(ProgressEvent::bytes(
            progress_kind::DOWNLOAD,
            download_progress_target_label(file_name),
            0,
            None,
        )));
    }
    loop {
        check_token_cancel(cancel)?;
        let next_chunk = response.chunk().await;
        let Some(chunk) = (match next_chunk {
            Ok(chunk) => chunk,
            Err(_err) if cancel.is_some_and(CancellationToken::is_cancelled) => {
                return Err(BackendError::Cancelled);
            }
            Err(err) => {
                return Err(BackendError::network(url, err)
                    .context(format!("failed to read response for '{url}'")));
            }
        }) else {
            break;
        };
        external(
            file.write_all(&chunk),
            format!("failed to write {}", destination.display()),
        )?;
        downloaded_bytes += chunk.len() as u64;
        if let Some(total_bytes) = total_bytes {
            let percent = if total_bytes == 0 {
                0
            } else {
                ((downloaded_bytes as f64 / total_bytes as f64) * 100.0)
                    .clamp(0.0, 100.0)
                    .round() as u64
            };
            if last_emitted_percent != Some(percent) {
                last_emitted_percent = Some(percent);
                if let Some(callback) = emit.as_deref_mut() {
                    callback(BackendEvent::Progress(ProgressEvent::bytes(
                        progress_kind::DOWNLOAD,
                        download_progress_target_label(file_name),
                        downloaded_bytes,
                        Some(total_bytes),
                    )));
                }
            }
        } else {
            let downloaded_mb_tenths = downloaded_bytes / (1024 * 1024 / 10);
            if last_emitted_mb_tenths != Some(downloaded_mb_tenths) {
                last_emitted_mb_tenths = Some(downloaded_mb_tenths);
                if let Some(callback) = emit.as_deref_mut() {
                    callback(BackendEvent::Progress(ProgressEvent::bytes(
                        progress_kind::DOWNLOAD,
                        download_progress_target_label(file_name),
                        downloaded_bytes,
                        None,
                    )));
                }
            }
        }
    }
    if let Some(total_bytes) = total_bytes {
        if let Some(callback) = emit.as_deref_mut() {
            callback(BackendEvent::Progress(ProgressEvent::bytes(
                progress_kind::DOWNLOAD,
                download_progress_target_label(file_name),
                downloaded_bytes,
                Some(total_bytes),
            )));
        }
    } else if let Some(callback) = emit.as_deref_mut() {
        callback(BackendEvent::Progress(ProgressEvent::bytes(
            progress_kind::DOWNLOAD,
            download_progress_target_label(file_name),
            downloaded_bytes,
            None,
        )));
    }
    Ok(())
}

pub(super) async fn ensure_cached_payloads(
    tool_root: &Path,
    target: &manifest::ToolchainTarget,
    payloads: &[manifest::SelectedPayload],
    proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: &mut Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<Vec<String>> {
    let cache_dir = payload_cache_dir(tool_root);
    external(
        fs::create_dir_all(&cache_dir),
        format!("failed to create {}", cache_dir.display()),
    )?;
    let client = http_client(proxy)?;
    let mut lines = vec![format!(
        "Caching {} MSVC payload archives under {}",
        payloads.len(),
        cache_dir.display()
    )];
    tracing::info!("{}", lines[0]);
    let mut downloaded = 0_usize;
    let mut reused = 0_usize;

    for (index, payload) in payloads.iter().enumerate() {
        check_token_cancel(cancel)?;
        if let Some(callback) = emit.as_deref_mut() {
            callback(BackendEvent::Progress(ProgressEvent::items(
                progress_kind::CACHE,
                format!(
                    "Caching payload {}/{}: {}",
                    index + 1,
                    payloads.len(),
                    payload.payload.file_name
                ),
                (index + 1) as u64,
                payloads.len() as u64,
            )));
        }
        let path = payload_cache_entry_path(tool_root, payload);
        let expected = decode_hex_sha256(&payload.payload.sha256).map_err(|err| {
            err.context(format!(
                "invalid payload sha256 for {}",
                payload.payload.file_name
            ))
        })?;
        if path.exists()
            && let Ok(actual) = file_sha256(&path)
            && actual == expected
        {
            reused += 1;
            tracing::info!(
                "Reused cached payload {}/{}: {}",
                index + 1,
                payloads.len(),
                payload.payload.file_name
            );
            continue;
        }

        if path.exists() {
            let _ = fs::remove_file(&path);
        }
        let download_result = download_or_copy_payload(
            &client,
            &payload.payload.url,
            &path,
            &payload.payload.file_name,
            cancel,
            emit,
        )
        .await;
        if let Err(err) = download_result {
            if err.to_string().eq_ignore_ascii_case("cancelled by user") {
                return Err(err);
            }
            return Err(err.context(format!(
                "failed to cache payload {}",
                payload_source_description(&payload.payload.url)
            )));
        }
        let actual = file_sha256(&path)
            .map_err(|err| err.context(format!("failed to verify {}", path.display())))?;
        if actual != expected {
            let _ = fs::remove_file(&path);
            return Err(BackendError::Other(format!(
                "sha256 mismatch for cached payload {}",
                payload.payload.file_name
            )));
        }
        downloaded += 1;
        tracing::info!(
            "Cached payload {}/{}: {}",
            index + 1,
            payloads.len(),
            payload.payload.file_name
        );
    }

    lines.push(format!(
        "Cached payload plan for {} (downloaded {}, reused {}).",
        target.label(),
        downloaded,
        reused
    ));
    Ok(lines)
}

pub(super) fn ensure_extracted_archives(
    tool_root: &Path,
    payloads: &[manifest::SelectedPayload],
) -> Result<Vec<String>> {
    let extract_root = extracted_payload_cache_dir(tool_root);
    external(
        fs::create_dir_all(&extract_root),
        format!("failed to create {}", extract_root.display()),
    )?;
    let mut extracted = 0_usize;
    let mut reused = 0_usize;
    let mut skipped = 0_usize;

    for payload in payloads {
        let Some(kind) = archive_kind_for_payload(payload) else {
            skipped += 1;
            continue;
        };
        if !matches!(kind, ArchiveKind::Zip | ArchiveKind::Vsix) {
            skipped += 1;
            continue;
        }

        let source = payload_cache_entry_path(tool_root, payload);
        if !source.exists() {
            skipped += 1;
            continue;
        }

        let destination = extracted_payload_entry_dir(tool_root, payload);
        let marker = destination.join(".complete");
        if marker.exists() {
            reused += 1;
            continue;
        }

        if destination.exists() {
            let _ = fs::remove_dir_all(&destination);
        }
        external(
            fs::create_dir_all(&destination),
            format!("failed to create {}", destination.display()),
        )?;
        extract_zip_archive(&source, &destination)?;
        external(
            fs::write(&marker, b"ok"),
            format!("failed to write {}", marker.display()),
        )?;
        extracted += 1;
    }

    Ok(vec![format!(
        "Prepared extracted archive payloads (extracted {}, reused {}, skipped {}).",
        extracted, reused, skipped
    )])
}

pub(super) fn ensure_msi_media_metadata(
    tool_root: &Path,
    payloads: &[manifest::SelectedPayload],
) -> Result<Vec<String>> {
    let metadata_root = msi_metadata_cache_dir(tool_root);
    external(
        fs::create_dir_all(&metadata_root),
        format!("failed to create {}", metadata_root.display()),
    )?;
    let mut inspected = 0_usize;
    let mut reused = 0_usize;
    let mut external_cabs = 0_usize;
    let mut unreadable = 0_usize;
    let mut warnings = Vec::new();

    for payload in payloads {
        if !matches!(archive_kind_for_payload(payload), Some(ArchiveKind::Msi)) {
            continue;
        }
        let source = payload_cache_entry_path(tool_root, payload);
        if !source.exists() {
            continue;
        }
        let metadata_path = msi_metadata_entry_path(tool_root, payload);
        if metadata_path.exists() {
            reused += 1;
            let existing = fs::read_to_string(&metadata_path).unwrap_or_default();
            external_cabs += existing
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                .count();
            continue;
        }

        let cab_names = match msi_extract::read_msi_cab_names(&source) {
            Ok(cab_names) => cab_names,
            Err(err) => {
                unreadable += 1;
                warnings.push(format!(
                    "Warning: failed to inspect MSI media table for {}: {err}",
                    source.display()
                ));
                continue;
            }
        };
        external_cabs += cab_names
            .iter()
            .filter(|name| !name.trim().is_empty() && !name.starts_with('#'))
            .count();
        external(
            fs::write(&metadata_path, cab_names.join("\n")),
            format!("failed to write {}", metadata_path.display()),
        )?;
        inspected += 1;
    }

    let mut lines = vec![format!(
        "Prepared MSI media metadata (inspected {}, reused {}, external cabs {}).",
        inspected, reused, external_cabs
    )];
    if unreadable > 0 {
        lines.push(format!(
            "Skipped MSI media inspection for {} unreadable payload(s).",
            unreadable
        ));
    }
    lines.extend(warnings);
    Ok(lines)
}

pub(super) async fn ensure_cached_companion_cabs(
    tool_root: &Path,
    target: &manifest::ToolchainTarget,
    payloads: &[manifest::SelectedPayload],
    proxy: &str,
    emit: &mut Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<Vec<String>> {
    let manifest_root = manifest_dir(tool_root);
    let mut companion_cabs = Vec::new();

    for payload in payloads {
        if !matches!(archive_kind_for_payload(payload), Some(ArchiveKind::Msi)) {
            continue;
        }
        let Some(cab_names) = read_cached_msi_cab_names(tool_root, payload) else {
            continue;
        };
        let external_cab_names = cab_names
            .into_iter()
            .filter(|name| !name.starts_with('#'))
            .collect::<Vec<_>>();
        if external_cab_names.is_empty() {
            continue;
        }
        if let Some(mut cabs) =
            manifest::companion_cab_payloads_for_selected_msi_from_cached_manifest(
                &manifest_root,
                payload,
                &external_cab_names,
            )
        {
            companion_cabs.append(&mut cabs);
        }
    }

    companion_cabs.sort_by(|left, right| {
        left.payload
            .file_name
            .cmp(&right.payload.file_name)
            .then(left.payload.url.cmp(&right.payload.url))
    });
    companion_cabs.dedup_by(|left, right| {
        left.payload.file_name == right.payload.file_name && left.payload.url == right.payload.url
    });

    if companion_cabs.is_empty() {
        return Ok(vec![format!(
            "Prepared external CAB payload cache plan for {} (selected 0).",
            target.label()
        )]);
    }

    let mut lines =
        ensure_cached_payloads(tool_root, target, &companion_cabs, proxy, None, emit).await?;
    lines[0] = format!(
        "Caching {} external CAB payload archives under {}",
        companion_cabs.len(),
        payload_cache_dir(tool_root).display()
    );
    lines[1] = format!(
        "Prepared external CAB payload cache plan for {} (selected {}).",
        target.label(),
        companion_cabs.len()
    );
    Ok(lines)
}

pub(super) fn ensure_staged_external_cabs(
    tool_root: &Path,
    payloads: &[manifest::SelectedPayload],
) -> Result<Vec<String>> {
    let staging_root = msi_staging_cache_dir(tool_root);
    external(
        fs::create_dir_all(&staging_root),
        format!("failed to create {}", staging_root.display()),
    )?;
    let manifest_root = manifest_dir(tool_root);
    let mut staged = 0_usize;
    let mut reused = 0_usize;
    let mut skipped = 0_usize;

    for payload in payloads {
        if !matches!(archive_kind_for_payload(payload), Some(ArchiveKind::Msi)) {
            skipped += 1;
            continue;
        }
        let Some(cab_names) = read_cached_msi_cab_names(tool_root, payload) else {
            skipped += 1;
            continue;
        };
        let external_cab_names = cab_names
            .into_iter()
            .filter(|name| !name.starts_with('#'))
            .collect::<Vec<_>>();
        if external_cab_names.is_empty() {
            skipped += 1;
            continue;
        }

        let Some(companion_cabs) =
            manifest::companion_cab_payloads_for_selected_msi_from_cached_manifest(
                &manifest_root,
                payload,
                &external_cab_names,
            )
        else {
            skipped += 1;
            continue;
        };
        if companion_cabs.is_empty() {
            skipped += 1;
            continue;
        }

        let staging_dir = msi_staging_entry_dir(tool_root, payload);
        let marker = staging_dir.join(".complete");
        if marker.exists() {
            reused += 1;
            continue;
        }

        if staging_dir.exists() {
            let _ = fs::remove_dir_all(&staging_dir);
        }
        external(
            fs::create_dir_all(&staging_dir),
            format!("failed to create {}", staging_dir.display()),
        )?;

        for cab_payload in companion_cabs {
            let source = payload_cache_entry_path(tool_root, &cab_payload);
            if !source.exists() {
                return Err(BackendError::Other(format!(
                    "external CAB payload {} is missing from cache",
                    source.display()
                )));
            }
            let file_name = Path::new(&cab_payload.payload.file_name)
                .file_name()
                .and_then(|name| name.to_str())
                .filter(|name| !name.is_empty())
                .unwrap_or("payload.cab");
            let destination = staging_dir.join(file_name);
            external(
                fs::copy(&source, &destination),
                format!(
                    "failed to stage external CAB {} to {}",
                    source.display(),
                    destination.display()
                ),
            )?;
        }
        external(
            fs::write(&marker, b"ok"),
            format!("failed to write {}", marker.display()),
        )?;
        staged += 1;
    }

    Ok(vec![format!(
        "Prepared MSI staging dirs for external CABs (staged {}, reused {}, skipped {}).",
        staged, reused, skipped
    )])
}

pub(super) fn ensure_extracted_msis(
    tool_root: &Path,
    payloads: &[manifest::SelectedPayload],
    emit: &mut Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<Vec<String>> {
    let extract_root = extracted_msi_cache_dir(tool_root);
    external(
        fs::create_dir_all(&extract_root),
        format!("failed to create {}", extract_root.display()),
    )?;
    let mut extracted = 0_usize;
    let mut reused = 0_usize;
    let mut skipped = 0_usize;
    let mut warnings = Vec::new();
    let actionable = payloads
        .iter()
        .filter(|payload| matches!(archive_kind_for_payload(payload), Some(ArchiveKind::Msi)))
        .filter(|payload| {
            let source = payload_cache_entry_path(tool_root, payload);
            let staging_marker = msi_staging_entry_dir(tool_root, payload).join(".complete");
            source.exists() && staging_marker.exists()
        })
        .count();
    if actionable > 0 {
        push_stream_line(
            &mut warnings,
            emit,
            format!(
                "Preparing extraction for {actionable} MSI payload(s) under {}",
                extract_root.display()
            ),
        );
    }
    let mut extracted_index = 0_usize;

    for payload in payloads {
        if !matches!(archive_kind_for_payload(payload), Some(ArchiveKind::Msi)) {
            skipped += 1;
            continue;
        }
        let source = payload_cache_entry_path(tool_root, payload);
        let staging_dir = msi_staging_entry_dir(tool_root, payload);
        let staging_marker = staging_dir.join(".complete");
        if !source.exists() || !staging_marker.exists() {
            skipped += 1;
            continue;
        }
        extracted_index += 1;
        let label = Path::new(&payload.payload.file_name)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&payload.payload.file_name);

        let destination = extracted_msi_entry_dir(tool_root, payload);
        let marker = destination.join(".complete");
        if marker.exists() {
            reused += 1;
            if let Some(callback) = emit.as_deref_mut() {
                callback(BackendEvent::Progress(ProgressEvent::items(
                    progress_kind::EXTRACT,
                    format!(
                        "Reusing extracted MSI payload {}/{}: {}",
                        extracted_index, actionable, label
                    ),
                    extracted_index as u64,
                    actionable as u64,
                )));
            }
            tracing::info!(
                "Reused extracted MSI payload {}/{}: {}",
                extracted_index,
                actionable,
                label
            );
            continue;
        }

        if destination.exists() {
            let _ = fs::remove_dir_all(&destination);
        }
        external(
            fs::create_dir_all(&destination),
            format!("failed to create {}", destination.display()),
        )?;
        if let Some(callback) = emit.as_deref_mut() {
            callback(BackendEvent::Progress(ProgressEvent::items(
                progress_kind::EXTRACT,
                format!(
                    "Extracting MSI payload {}/{}: {}",
                    extracted_index, actionable, label
                ),
                extracted_index as u64,
                actionable as u64,
            )));
        }

        let source_for_extract = source.clone();
        let destination_for_extract = destination.clone();
        let staging_for_extract = staging_dir.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = msi_extract::extract_msi_with_staged_cabs(
                &source_for_extract,
                &destination_for_extract,
                &staging_for_extract,
            )
            .map_err(|err| err.to_string());
            let _ = tx.send(result);
        });
        let started = Instant::now();
        let extract_result = loop {
            match rx.recv_timeout(Duration::from_secs(5)) {
                Ok(result) => break result,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(callback) = emit.as_deref_mut() {
                        callback(BackendEvent::Progress(ProgressEvent::activity(
                            progress_kind::EXTRACT,
                            format!(
                                "Extracting MSI payload {}/{}: {} ({:.0}s elapsed)",
                                extracted_index,
                                actionable,
                                label,
                                started.elapsed().as_secs_f64()
                            ),
                        )));
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break Err(format!(
                        "MSI extraction worker stopped unexpectedly for {}",
                        source.display()
                    ));
                }
            }
        };

        match extract_result {
            Ok(_) => {
                fs::write(&marker, b"ok").map_err(|err| BackendError::fs("write", &marker, err))?;
                extracted += 1;
                tracing::info!(
                    "Extracted MSI payload {}/{}: {}",
                    extracted_index,
                    actionable,
                    label
                );
            }
            Err(err) => {
                let _ = fs::remove_dir_all(&destination);
                tracing::warn!(
                    "Warning: failed to extract MSI payload {}/{}: {}",
                    extracted_index,
                    actionable,
                    label
                );
                warnings.push(format!(
                    "Warning: failed to extract MSI payload {}: {err}",
                    source.display()
                ));
            }
        }
    }

    let lines = vec![format!(
        "Prepared extracted MSI payloads (extracted {}, reused {}, skipped {}).",
        extracted, reused, skipped
    )];
    let mut lines = lines;
    lines.extend(warnings);
    Ok(lines)
}

fn copy_tree_into(src: &Path, dest: &Path) -> Result<usize> {
    let mut copied = 0_usize;
    for entry in WalkDir::new(src) {
        let entry = external(entry, format!("failed to walk {}", src.display()))?;
        let path = entry.path();
        if path == src {
            continue;
        }
        let relative = path.strip_prefix(src).map_err(|err| {
            BackendError::Other(format!(
                "failed to strip {} from {}: {err}",
                src.display(),
                path.display()
            ))
        })?;
        if relative
            .file_name()
            .map(|name| name == ".complete")
            .unwrap_or(false)
        {
            continue;
        }
        let destination = dest.join(relative);
        if entry.file_type().is_dir() {
            external(
                fs::create_dir_all(&destination),
                format!("failed to create {}", destination.display()),
            )?;
            continue;
        }
        if let Some(parent) = destination.parent() {
            external(
                fs::create_dir_all(parent),
                format!("failed to create {}", parent.display()),
            )?;
        }
        if destination.exists() {
            continue;
        }
        external(
            fs::copy(path, &destination),
            format!(
                "failed to copy {} into {}",
                path.display(),
                destination.display()
            ),
        )?;
        copied += 1;
    }
    Ok(copied)
}

pub(super) fn ensure_install_image(
    tool_root: &Path,
    payloads: &[manifest::SelectedPayload],
) -> Result<Vec<String>> {
    let image_root = install_image_cache_dir(tool_root);
    if image_root.exists() {
        external(
            fs::remove_dir_all(&image_root),
            format!("failed to reset {}", image_root.display()),
        )?;
    }
    external(
        fs::create_dir_all(&image_root),
        format!("failed to create {}", image_root.display()),
    )?;
    let mut copied = 0_usize;
    let mut skipped = 0_usize;

    for payload in payloads {
        let source_root = match archive_kind_for_payload(payload) {
            Some(ArchiveKind::Zip) => Some(extracted_payload_entry_dir(tool_root, payload)),
            Some(ArchiveKind::Vsix) => {
                let base = extracted_payload_entry_dir(tool_root, payload);
                let contents = base.join("Contents");
                Some(if contents.exists() { contents } else { base })
            }
            Some(ArchiveKind::Msi) => Some(extracted_msi_entry_dir(tool_root, payload)),
            _ => None,
        };
        let Some(source_root) = source_root else {
            skipped += 1;
            continue;
        };
        let marker = source_root.join(".complete");
        if !source_root.exists()
            || (!marker.exists()
                && matches!(archive_kind_for_payload(payload), Some(ArchiveKind::Msi)))
        {
            skipped += 1;
            continue;
        }

        copied += copy_tree_into(&source_root, &image_root)?;
    }

    Ok(vec![format!(
        "Prepared install image from extracted payloads (copied {}, skipped {}).",
        copied, skipped
    )])
}

pub(super) fn write_installed_state(
    tool_root: &Path,
    target: &manifest::ToolchainTarget,
) -> Result<()> {
    let managed_root = paths::msvc_root(tool_root);
    // Convert manifest::ToolchainTarget to rules::ToolchainTarget
    let rules_target = rules::ToolchainTarget {
        msvc: target.msvc.clone(),
        sdk: target.sdk.clone(),
    };
    write_installed_toolchain_target(&managed_root, &rules_target).map_err(|err| {
        err.context(format!(
            "failed to write installed MSVC state under {}",
            managed_root.display()
        ))
    })?;
    Ok(())
}

pub(super) fn write_runtime_state(tool_root: &Path) -> Result<Vec<String>> {
    let state_root = paths::msvc_state_root(tool_root);
    external(
        fs::create_dir_all(&state_root),
        format!("failed to create {}", state_root.display()),
    )?;
    let runtime_state = runtime_state_path(tool_root);
    fs::write(
        &runtime_state,
        serde_json::to_string_pretty(&serde_json::json!({
            "toolchain_root": msvc_dir(tool_root),
            "wrappers_root": paths::shims_root(tool_root),
            "runtime": "managed"
        }))?,
    )
    .map_err(|err| {
        BackendError::Other(format!(
            "failed to write {}: {err}",
            runtime_state.display()
        ))
    })?;
    Ok(vec![format!(
        "Wrote managed runtime state into {}.",
        runtime_state.display()
    )])
}

pub(super) fn remove_autoenv_dir(tool_root: &Path) -> Result<Vec<String>> {
    let autoenv_root = msvc_dir(tool_root).join("autoenv");
    if !autoenv_root.exists() {
        return Ok(Vec::new());
    }
    external(
        fs::remove_dir_all(&autoenv_root),
        format!("failed to remove {}", autoenv_root.display()),
    )?;
    Ok(vec![format!(
        "Removed autoenv directory {}.",
        autoenv_root.display()
    )])
}

pub(super) fn ensure_materialized_toolchain(
    tool_root: &Path,
    target: &manifest::ToolchainTarget,
) -> Result<Vec<String>> {
    let image_root = install_image_cache_dir(tool_root);
    if !image_root.exists() {
        return Ok(vec![
            "Install image not present yet; skipped toolchain materialization.".to_string(),
        ]);
    }

    let toolchain_root = msvc_dir(tool_root);
    external(
        fs::create_dir_all(&toolchain_root),
        format!("failed to create {}", toolchain_root.display()),
    )?;

    let before = count_files_recursively(&toolchain_root);
    let copied = copy_tree_into(&image_root, &toolchain_root)?;
    let after = count_files_recursively(&toolchain_root);
    let reused = usize::from(after == before);
    write_installed_state(tool_root, target)?;

    Ok(vec![format!(
        "Materialized managed toolchain image into {} (copied {}, reused {}).",
        toolchain_root.display(),
        copied,
        reused
    )])
}

pub(super) fn cleanup_post_install_cache(tool_root: &Path) -> Vec<String> {
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

#[cfg(test)]
mod tests;
