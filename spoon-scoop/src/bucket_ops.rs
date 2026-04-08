//! Bucket write operations: add, remove, update.

use std::path::Path;

use spoon_core::{RuntimeLayout, clone_repo, copy_path_recursive};

use crate::bucket::{
    Bucket, BucketSpec,
    load_buckets_from_registry, remove_bucket_from_registry,
    upsert_bucket_to_registry,
};
use crate::response::ScoopBucketOperationOutcome;
use crate::error::Result;
use crate::ScoopError;

/// Add a bucket: clone from source (or copy local dir) and register.
pub async fn add_bucket(
    tool_root: &Path,
    spec: &BucketSpec,
    proxy: &str,
) -> Result<ScoopBucketOperationOutcome> {
    let name = &spec.name;
    if name.trim().is_empty() {
        return Err(ScoopError::Config("bucket name must not be empty".to_string()));
    }

    let layout = RuntimeLayout::from_root(tool_root);
    let bucket_dir = layout.scoop.bucket_root(name);

    if bucket_dir.exists() {
        return Err(ScoopError::Other(format!("bucket '{}' already exists", name)));
    }

    // Ensure parent exists
    if let Some(parent) = bucket_dir.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| ScoopError::fs("create", parent, e))?;
    }

    let is_local_dir = spec.source.as_ref().map(|s| Path::new(s).is_dir()).unwrap_or(false);

    if is_local_dir {
        let source = spec.source.as_ref().expect("local source must exist");
        copy_path_recursive(Path::new(source), &bucket_dir, None).await
            .map_err(|e| e.context(format!("failed to copy bucket '{}'", name)))?;
        upsert_bucket_to_registry(&layout.scoop.root, spec).await?;
        tracing::info!("Synced local bucket '{}' into {}", source, bucket_dir.display());
    } else {
        let source = spec.source.as_ref()
            .ok_or_else(|| ScoopError::Config("bucket source must not be empty".to_string()))?;
        let branch = spec.branch.as_deref().unwrap_or("master");
        let temp_dir = bucket_dir.with_extension(format!("spoon-add-{}", std::process::id()));

        if temp_dir.exists() {
            let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        }

        tracing::info!("Cloning bucket '{}' from {}...", name, source);
        match clone_repo(source, &temp_dir, Some(branch), proxy, None, None).await {
            Ok(outcome) => {
                if let Some(b) = &outcome.head_branch {
                    tracing::info!("Checked out branch: {b}");
                }
                if let Some(c) = &outcome.head_commit {
                    tracing::info!("HEAD at commit: {c}");
                }
            }
            Err(err) => {
                let _ = tokio::fs::remove_dir_all(&temp_dir).await;
                return Err(ScoopError::Other(format!(
                    "failed to clone bucket '{}': {}", name, err
                )));
            }
        };

        if !temp_dir.join(".git").exists() {
            let _ = tokio::fs::remove_dir_all(&temp_dir).await;
            return Err(ScoopError::Other(format!("failed to clone bucket '{}'", name)));
        }

        if bucket_dir.exists() {
            tokio::fs::remove_dir_all(&bucket_dir).await
                .map_err(|e| ScoopError::fs("clear", &bucket_dir, e))?;
        }
        tokio::fs::rename(&temp_dir, &bucket_dir).await
            .map_err(|e| ScoopError::Other(format!("failed to finalize {}: {e}", bucket_dir.display())))?;
        upsert_bucket_to_registry(&layout.scoop.root, spec).await?;
    }

    tracing::info!("Registered bucket '{}'", name);
    let buckets = load_buckets_from_registry(&layout.scoop.root).await;
    Ok(ScoopBucketOperationOutcome {
        kind: "scoop_bucket_action",
        action: "add".to_string(),
        targets: vec![name.clone()],
        status: spoon_core::CommandStatus::Success,
        title: format!("add Scoop bucket {}", name),
        streamed: false,
        output: vec![],
        bucket_count: buckets.len(),
        buckets,
    })
}

/// Remove a bucket: delete directory and unregister.
pub async fn remove_bucket(
    tool_root: &Path,
    name: &str,
) -> Result<ScoopBucketOperationOutcome> {
    if name.eq_ignore_ascii_case("main") {
        return Err(ScoopError::Config("bucket 'main' cannot be removed".to_string()));
    }

    let layout = RuntimeLayout::from_root(tool_root);
    let bucket_dir = layout.scoop.bucket_root(name);

    if !bucket_dir.exists() {
        return Err(ScoopError::Other(format!("bucket '{}' is not installed", name)));
    }

    let mut output = Vec::new();
    tokio::fs::remove_dir_all(&bucket_dir).await
        .map_err(|e| ScoopError::fs("remove", &bucket_dir, e))?;
    output.push(format!("Removed bucket directory: {}", bucket_dir.display()));

    remove_bucket_from_registry(&layout.scoop.root, name).await?;
    output.push(format!("Removed bucket '{}'.", name));

    let buckets = load_buckets_from_registry(&layout.scoop.root).await;
    Ok(ScoopBucketOperationOutcome {
        kind: "scoop_bucket_action",
        action: "remove".to_string(),
        targets: vec![name.to_string()],
        status: spoon_core::CommandStatus::Success,
        title: format!("remove Scoop bucket {name}"),
        streamed: false,
        output,
        bucket_count: buckets.len(),
        buckets,
    })
}

/// Update buckets: git clone fresh and replace, or sync local directories.
pub async fn update_buckets(
    tool_root: &Path,
    names: &[String],
    proxy: &str,
) -> Result<ScoopBucketOperationOutcome> {
    let layout = RuntimeLayout::from_root(tool_root);
    let all_buckets = load_buckets_from_registry(&layout.scoop.root).await;

    let selected: Vec<Bucket> = if names.is_empty() {
        all_buckets
    } else {
        all_buckets.into_iter()
            .filter(|b| names.iter().any(|n| n.eq_ignore_ascii_case(&b.name)))
            .collect()
    };

    let mut output = Vec::new();
    let mut updated = 0_usize;
    let mut skipped = 0_usize;

    for bucket in selected {
        let source = bucket.source.clone();
        if source.trim().is_empty() {
            let line = format!("Skipped bucket '{}': no registered source.", bucket.name);
            tracing::info!("{line}");
            output.push(line);
            skipped += 1;
            continue;
        }

        let current_root = layout.scoop.bucket_root(&bucket.name);
        let branch = if bucket.branch.trim().is_empty() { "master" } else { &bucket.branch };
        let is_local_dir = Path::new(&source).is_dir();

        if is_local_dir {
            output.push(format!("Syncing local bucket '{}'...", bucket.name));
            if current_root.exists() {
                tokio::fs::remove_dir_all(&current_root).await
                    .map_err(|e| ScoopError::Other(format!("failed to remove {}: {e}", current_root.display())))?;
            }
            copy_path_recursive(Path::new(&source), &current_root, None).await
                .map_err(|e| e.context(format!("failed to update bucket '{}'", bucket.name)))?;
            output.push(format!("Updated bucket '{}' from local source.", bucket.name));
            updated += 1;
            continue;
        }

        let temp_root = current_root.with_extension(format!("spoon-update-{}", std::process::id()));
        if temp_root.exists() {
            let _ = tokio::fs::remove_dir_all(&temp_root).await;
        }

        output.push(format!("Updating bucket '{}' from {}...", bucket.name, source));
        match clone_repo(&source, &temp_root, Some(branch), proxy, None, None).await {
            Ok(outcome) => {
                if let Some(b) = &outcome.head_branch {
                    output.push(format!("Checked out branch: {b}"));
                }
                if let Some(c) = &outcome.head_commit {
                    output.push(format!("HEAD at commit: {c}"));
                }
            }
            Err(err) => {
                let _ = tokio::fs::remove_dir_all(&temp_root).await;
                output.push(format!("Failed to update bucket '{}': {err}", bucket.name));
                skipped += 1;
                continue;
            }
        };

        if current_root.exists() {
            tokio::fs::remove_dir_all(&current_root).await
                .map_err(|e| ScoopError::Other(format!("failed to remove {}: {e}", current_root.display())))?;
        }
        tokio::fs::rename(&temp_root, &current_root).await
            .map_err(|e| ScoopError::Other(format!("failed to replace {}: {e}", current_root.display())))?;

        output.push(format!("Updated bucket '{}'.", bucket.name));
        updated += 1;
    }

    output.push(format!("Bucket update summary: {} updated, {} skipped.", updated, skipped));

    let buckets = load_buckets_from_registry(&layout.scoop.root).await;
    Ok(ScoopBucketOperationOutcome {
        kind: "scoop_bucket_action",
        action: "update".to_string(),
        targets: names.to_vec(),
        status: spoon_core::CommandStatus::Success,
        title: "update Scoop buckets".to_string(),
        streamed: false,
        output,
        bucket_count: buckets.len(),
        buckets,
    })
}
