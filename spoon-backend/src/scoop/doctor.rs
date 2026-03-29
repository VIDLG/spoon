use std::path::Path;

use serde::Serialize;
use tokio::fs;

use crate::layout::RuntimeLayout;
use crate::{BackendError, Result};

use super::buckets::load_buckets_from_registry;
use super::paths::{scoop_root, scoop_state_root, shims_root};
use super::runtime::{ScoopRuntimeHost, ensure_scoop_shims_activated_with_host};

#[derive(Debug, Serialize)]
pub struct ScoopRuntimeDetails {
    pub root: String,
    pub state_root: String,
    pub shims_root: String,
}

#[derive(Debug, Serialize)]
pub struct LegacyScoopStateIssue {
    pub kind: &'static str,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ScoopDoctorDetails {
    pub kind: &'static str,
    pub success: bool,
    pub runtime: ScoopRuntimeDetails,
    pub ensured_paths: Vec<String>,
    pub shim_activation_output: Vec<String>,
    pub registered_buckets: Vec<super::buckets::Bucket>,
    pub legacy_state_issues: Vec<LegacyScoopStateIssue>,
}

/// Detect old flat Scoop state files that are not part of the canonical state layout.
///
/// Scans `scoop/state/*.json` directly (the old layout) and reports any files found
/// that are NOT the bucket registry (`buckets.json`) and NOT inside the canonical
/// `packages/` subdirectory. These represent legacy state from the pre-canonical
/// model and should be reported explicitly rather than silently treated as supported.
pub async fn detect_legacy_flat_state_files(
    layout: &RuntimeLayout,
) -> Vec<LegacyScoopStateIssue> {
    let state_root = &layout.scoop.state_root;
    let mut issues = Vec::new();

    if !state_root.exists() {
        return issues;
    }

    let mut entries = match fs::read_dir(state_root).await {
        Ok(entries) => entries,
        Err(_) => return issues,
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();

        // Skip directories (including the canonical "packages/" subdirectory)
        if path.is_dir() {
            continue;
        }

        // Skip the bucket registry file which legitimately lives in state_root
        if path.file_name().and_then(|n| n.to_str()) == Some("buckets.json") {
            continue;
        }

        // Only consider .json files
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        issues.push(LegacyScoopStateIssue {
            kind: "legacy scoop state",
            path: path.display().to_string(),
            message: format!(
                "legacy scoop state file '{}' is not supported; remove it and rebuild state from canonical packages/",
                path.display()
            ),
        });
    }

    issues
}

pub async fn doctor_with_host(
    tool_root: &Path,
    proxy: &str,
    host: &dyn ScoopRuntimeHost,
) -> Result<ScoopDoctorDetails> {
    let scoop_root = scoop_root(tool_root);
    let ensured_paths = vec![
        scoop_root.clone(),
        scoop_root.join("apps"),
        scoop_root.join("buckets"),
        scoop_root.join("cache"),
        scoop_root.join("persist"),
        scoop_state_root(tool_root),
        shims_root(tool_root),
    ];

    for path in &ensured_paths {
        fs::create_dir_all(path)
            .await
            .map_err(|err| BackendError::fs("create", path, err))?;
    }

    super::buckets::ensure_main_bucket_ready(tool_root, proxy).await?;
    let shim_activation_output = ensure_scoop_shims_activated_with_host(tool_root, host).await?;

    let layout = RuntimeLayout::from_root(tool_root);
    let legacy_state_issues = detect_legacy_flat_state_files(&layout).await;

    Ok(ScoopDoctorDetails {
        kind: "scoop_doctor",
        success: legacy_state_issues.is_empty(),
        runtime: ScoopRuntimeDetails {
            root: scoop_root.display().to_string(),
            state_root: scoop_state_root(tool_root).display().to_string(),
            shims_root: shims_root(tool_root).display().to_string(),
        },
        ensured_paths: ensured_paths
            .into_iter()
            .map(|path| path.display().to_string())
            .collect(),
        shim_activation_output,
        registered_buckets: load_buckets_from_registry(tool_root).await,
        legacy_state_issues,
    })
}
