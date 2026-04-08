//! Pipeline steps — path helpers, `ensure_*` preparation functions, and `copy_tree_into`.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use fs_err as fs;
use walkdir::WalkDir;

use spoon_core::{
    CancellationToken, CoreError, ProgressEvent, Result, SpoonEvent, check_token_cancel, progress_kind,
    extract_zip_archive_sync,
};

use crate::common::{http_client, push_stream_line};
use crate::facts::package_rules::ArchiveKind;
use crate::facts::manifest;
use crate::msi_extract;
use crate::paths;

use super::integrity;

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

pub fn native_host_arch() -> &'static str {
    paths::native_msvc_arch()
}

pub fn msvc_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_toolchain_root(tool_root)
}

pub fn runtime_state_path(tool_root: &Path) -> PathBuf {
    paths::msvc_state_root(tool_root).join("runtime.json")
}

pub fn manifest_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_manifest_root(tool_root)
}

pub(crate) fn payload_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root).join("archives")
}

pub(crate) fn extracted_payload_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root)
        .join("expanded")
        .join("archives")
}

pub(crate) fn extracted_msi_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root)
        .join("expanded")
        .join("msi")
}

pub(crate) fn install_image_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root).join("image")
}

pub(crate) fn msi_metadata_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root)
        .join("metadata")
        .join("msi")
}

pub(crate) fn msi_staging_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root).join("stage").join("msi")
}

pub(crate) fn payload_cache_entry_name(payload: &manifest::SelectedPayload) -> String {
    let leaf = Path::new(&payload.payload.file_name)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("payload.bin");
    format!("{}-{}", payload.payload.sha256.to_ascii_lowercase(), leaf)
}

pub(crate) fn payload_cache_entry_path(tool_root: &Path, payload: &manifest::SelectedPayload) -> PathBuf {
    payload_cache_dir(tool_root).join(payload_cache_entry_name(payload))
}

pub(crate) fn extracted_payload_entry_dir(tool_root: &Path, payload: &manifest::SelectedPayload) -> PathBuf {
    extracted_payload_cache_dir(tool_root).join(payload.payload.sha256.to_ascii_lowercase())
}

pub(crate) fn msi_metadata_entry_path(tool_root: &Path, payload: &manifest::SelectedPayload) -> PathBuf {
    msi_metadata_cache_dir(tool_root).join(format!(
        "{}.txt",
        payload.payload.sha256.to_ascii_lowercase()
    ))
}

pub(crate) fn msi_staging_entry_dir(tool_root: &Path, payload: &manifest::SelectedPayload) -> PathBuf {
    msi_staging_cache_dir(tool_root).join(payload.payload.sha256.to_ascii_lowercase())
}

pub(crate) fn extracted_msi_entry_dir(tool_root: &Path, payload: &manifest::SelectedPayload) -> PathBuf {
    extracted_msi_cache_dir(tool_root).join(payload.payload.sha256.to_ascii_lowercase())
}

// ---------------------------------------------------------------------------
// MSI metadata helpers (depend on path helpers above)
// ---------------------------------------------------------------------------

pub(crate) fn read_cached_msi_cab_names(
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

// ---------------------------------------------------------------------------
// Download / copy helper
// ---------------------------------------------------------------------------

async fn download_or_copy_payload(
    client: &reqwest::Client,
    url: &str,
    destination: &Path,
    file_name: &str,
    cancel: Option<&CancellationToken>,
    _emit: &mut Option<&mut dyn FnMut(SpoonEvent)>,
) -> Result<()> {
    let label = integrity::download_progress_target_label(file_name).to_string();
    spoon_core::copy_or_download_to_file(
        client,
        url,
        destination,
        &label,
        progress_kind::DOWNLOAD,
        cancel,
        None,
    )
    .await
}

// ---------------------------------------------------------------------------
// copy_tree_into
// ---------------------------------------------------------------------------

pub(crate) fn copy_tree_into(src: &Path, dest: &Path) -> Result<usize> {
    let mut copied = 0_usize;
    for entry in WalkDir::new(src) {
        let entry = entry.map_err(|e| CoreError::Other(format!("walk {}: {e}", src.display())))?;
        let path = entry.path();
        if path == src {
            continue;
        }
        let relative = path.strip_prefix(src)
            .map_err(|e| CoreError::Other(format!("strip {} from {}: {e}", src.display(), path.display())))?;
        if relative
            .file_name()
            .map(|name| name == ".complete")
            .unwrap_or(false)
        {
            continue;
        }
        let destination = dest.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&destination)
                .map_err(|e| CoreError::fs("create_dir_all", &destination, e))?;
            continue;
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CoreError::fs("create_dir_all", parent, e))?;
        }
        if destination.exists() {
            continue;
        }
        fs::copy(path, &destination)
            .map_err(|e| {
                CoreError::Other(format!(
                    "failed to copy {} into {}: {e}",
                    path.display(),
                    destination.display()
                ))
            })?;
        copied += 1;
    }
    Ok(copied)
}

// ---------------------------------------------------------------------------
// ensure_* pipeline functions
// ---------------------------------------------------------------------------

pub async fn ensure_cached_payloads(
    tool_root: &Path,
    target: &manifest::ToolchainTarget,
    payloads: &[manifest::SelectedPayload],
    proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: &mut Option<&mut dyn FnMut(SpoonEvent)>,
) -> Result<Vec<String>> {
    let cache_dir = payload_cache_dir(tool_root);
    fs::create_dir_all(&cache_dir)
        .map_err(|e| CoreError::fs("create_dir_all", &cache_dir, e))?;
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
            callback(SpoonEvent::Progress(ProgressEvent::items(
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
        let expected = integrity::decode_hex_sha256(&payload.payload.sha256).map_err(|err| {
            CoreError::Other(format!(
                "invalid payload sha256 for {}: {err}",
                payload.payload.file_name
            ))
        })?;
        if path.exists()
            && let Ok(actual) = integrity::file_sha256(&path)
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
                integrity::payload_source_description(&payload.payload.url)
            )));
        }
        let actual = integrity::file_sha256(&path)
            .map_err(|e| CoreError::Other(format!("failed to verify {}: {e}", path.display())))?;
        if actual != expected {
            let _ = fs::remove_file(&path);
            return Err(CoreError::Other(format!(
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

pub fn ensure_extracted_archives(
    tool_root: &Path,
    payloads: &[manifest::SelectedPayload],
) -> Result<Vec<String>> {
    let extract_root = extracted_payload_cache_dir(tool_root);
    fs::create_dir_all(&extract_root)
        .map_err(|e| CoreError::fs("create_dir_all", &extract_root, e))?;
    let mut extracted = 0_usize;
    let mut reused = 0_usize;
    let mut skipped = 0_usize;

    for payload in payloads {
        let Some(kind) = integrity::archive_kind_for_payload(payload) else {
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
        fs::create_dir_all(&destination)
            .map_err(|e| CoreError::fs("create_dir_all", &destination, e))?;
        extract_zip_archive_sync(&source, &destination)
            .map_err(|e| CoreError::Other(format!("failed to extract {}: {e}", source.display())))?;
        fs::write(&marker, b"ok")
            .map_err(|e| CoreError::fs("write", &marker, e))?;
        extracted += 1;
    }

    Ok(vec![format!(
        "Prepared extracted archive payloads (extracted {}, reused {}, skipped {}).",
        extracted, reused, skipped
    )])
}

pub fn ensure_msi_media_metadata(
    tool_root: &Path,
    payloads: &[manifest::SelectedPayload],
) -> Result<Vec<String>> {
    let metadata_root = msi_metadata_cache_dir(tool_root);
    fs::create_dir_all(&metadata_root)
        .map_err(|e| CoreError::fs("create_dir_all", &metadata_root, e))?;
    let mut inspected = 0_usize;
    let mut reused = 0_usize;
    let mut external_cabs = 0_usize;
    let mut unreadable = 0_usize;
    let mut warnings = Vec::new();

    for payload in payloads {
        if !matches!(integrity::archive_kind_for_payload(payload), Some(ArchiveKind::Msi)) {
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
        fs::write(&metadata_path, cab_names.join("\n"))
            .map_err(|e| CoreError::fs("write", &metadata_path, e))?;
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

pub async fn ensure_cached_companion_cabs(
    tool_root: &Path,
    target: &manifest::ToolchainTarget,
    payloads: &[manifest::SelectedPayload],
    proxy: &str,
    emit: &mut Option<&mut dyn FnMut(SpoonEvent)>,
) -> Result<Vec<String>> {
    let mut companion_cabs = Vec::new();

    for payload in payloads {
        if !matches!(integrity::archive_kind_for_payload(payload), Some(ArchiveKind::Msi)) {
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
                &manifest_dir(tool_root),
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

pub fn ensure_staged_external_cabs(
    tool_root: &Path,
    payloads: &[manifest::SelectedPayload],
) -> Result<Vec<String>> {
    let staging_root = msi_staging_cache_dir(tool_root);
    fs::create_dir_all(&staging_root)
        .map_err(|e| CoreError::fs("create_dir_all", &staging_root, e))?;
    let mut staged = 0_usize;
    let mut reused = 0_usize;
    let mut skipped = 0_usize;

    for payload in payloads {
        if !matches!(integrity::archive_kind_for_payload(payload), Some(ArchiveKind::Msi)) {
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
                &manifest_dir(tool_root),
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
        fs::create_dir_all(&staging_dir)
            .map_err(|e| CoreError::fs("create_dir_all", &staging_dir, e))?;

        for cab_payload in companion_cabs {
            let source = payload_cache_entry_path(tool_root, &cab_payload);
            if !source.exists() {
                return Err(CoreError::Other(format!(
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
            fs::copy(&source, &destination)
                .map_err(|e| {
                    CoreError::Other(format!(
                        "failed to stage external CAB {} to {}: {e}",
                        source.display(),
                        destination.display()
                    ))
                })?;
        }
        fs::write(&marker, b"ok")
            .map_err(|e| CoreError::fs("write", &marker, e))?;
        staged += 1;
    }

    Ok(vec![format!(
        "Prepared MSI staging dirs for external CABs (staged {}, reused {}, skipped {}).",
        staged, reused, skipped
    )])
}

pub fn ensure_extracted_msis(
    tool_root: &Path,
    payloads: &[manifest::SelectedPayload],
    emit: &mut Option<&mut dyn FnMut(SpoonEvent)>,
) -> Result<Vec<String>> {
    let extract_root = extracted_msi_cache_dir(tool_root);
    fs::create_dir_all(&extract_root)
        .map_err(|e| CoreError::fs("create_dir_all", &extract_root, e))?;
    let mut extracted = 0_usize;
    let mut reused = 0_usize;
    let mut skipped = 0_usize;
    let mut warnings = Vec::new();
    let actionable = payloads
        .iter()
        .filter(|payload| matches!(integrity::archive_kind_for_payload(payload), Some(ArchiveKind::Msi)))
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
        if !matches!(integrity::archive_kind_for_payload(payload), Some(ArchiveKind::Msi)) {
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
                callback(SpoonEvent::Progress(ProgressEvent::items(
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
        fs::create_dir_all(&destination)
            .map_err(|e| CoreError::fs("create_dir_all", &destination, e))?;
        if let Some(callback) = emit.as_deref_mut() {
            callback(SpoonEvent::Progress(ProgressEvent::items(
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
            .map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
        let started = Instant::now();
        let extract_result = loop {
            match rx.recv_timeout(Duration::from_secs(5)) {
                Ok(result) => break result,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(callback) = emit.as_deref_mut() {
                        callback(SpoonEvent::Progress(ProgressEvent::activity(
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
                fs::write(&marker, b"ok")
                    .map_err(|e| CoreError::fs("write", &marker, e))?;
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

    let mut lines = vec![format!(
        "Prepared extracted MSI payloads (extracted {}, reused {}, skipped {}).",
        extracted, reused, skipped
    )];
    lines.extend(warnings);
    Ok(lines)
}

pub fn ensure_install_image(
    tool_root: &Path,
    payloads: &[manifest::SelectedPayload],
) -> Result<Vec<String>> {
    let image_root = install_image_cache_dir(tool_root);
    if image_root.exists() {
        fs::remove_dir_all(&image_root)
            .map_err(|e| CoreError::fs("remove_dir_all", &image_root, e))?;
    }
    fs::create_dir_all(&image_root)
        .map_err(|e| CoreError::fs("create_dir_all", &image_root, e))?;
    let mut copied = 0_usize;
    let mut skipped = 0_usize;

    for payload in payloads {
        let source_root = match integrity::archive_kind_for_payload(payload) {
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
                && matches!(integrity::archive_kind_for_payload(payload), Some(ArchiveKind::Msi)))
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
