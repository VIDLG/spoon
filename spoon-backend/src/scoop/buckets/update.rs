use std::path::Path;

use tokio::fs;

use crate::layout::RuntimeLayout;
use crate::BackendContext;
use crate::{BackendError, BackendEvent, CommandStatus, Result, clone_repo, fsx};

use super::models::{BucketUpdateSummary, ScoopBucketOperationOutcome};
use super::registry::load_buckets_from_registry;

pub async fn update_buckets(
    tool_root: &Path,
    names: &[String],
    proxy: &str,
) -> Result<(Vec<String>, BucketUpdateSummary)> {
    update_buckets_streaming(tool_root, names, proxy, None).await
}

pub async fn update_buckets_with_context<P>(
    context: &BackendContext<P>,
    names: &[String],
) -> Result<(Vec<String>, BucketUpdateSummary)> {
    update_buckets_streaming_with_context(context, names, None).await
}

pub async fn update_buckets_outcome(
    tool_root: &Path,
    names: &[String],
    proxy: &str,
) -> Result<ScoopBucketOperationOutcome> {
    let (mut output, summary) = update_buckets(tool_root, names, proxy).await?;
    output.push(format!(
        "Bucket update summary: {} updated, {} skipped.",
        summary.updated, summary.skipped
    ));
    let buckets = load_buckets_from_registry(tool_root).await;
    Ok(ScoopBucketOperationOutcome {
        kind: "scoop_bucket_action",
        action: "update".to_string(),
        targets: names.to_vec(),
        status: CommandStatus::Success,
        title: "update Scoop buckets".to_string(),
        streamed: false,
        output,
        bucket_count: buckets.len(),
        buckets,
    })
}

pub async fn update_buckets_streaming(
    tool_root: &Path,
    names: &[String],
    proxy: &str,
    _emit: Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<(Vec<String>, BucketUpdateSummary)> {
    let all_buckets = load_buckets_from_registry(tool_root).await;
    let selected = if names.is_empty() {
        all_buckets
    } else {
        all_buckets
            .into_iter()
            .filter(|bucket| {
                names
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(&bucket.name))
            })
            .collect::<Vec<_>>()
    };

    let mut lines = Vec::new();
    let mut updated = 0_usize;
    let mut skipped = 0_usize;
    for bucket in selected {
        let source = bucket.source.clone();
        if source.trim().is_empty() {
            let line = format!(
                "Skipped bucket '{}': no registered source is available.",
                bucket.name
            );
            tracing::info!("{line}");
            lines.push(line);
            skipped += 1;
            continue;
        }
        let current_root = RuntimeLayout::from_root(tool_root)
            .scoop
            .bucket_root(&bucket.name);
        let branch = if bucket.branch.trim().is_empty() {
            "master"
        } else {
            &bucket.branch
        };
        let local_directory_source = Path::new(&source).is_dir();
        if local_directory_source {
            let line = format!(
                "Starting Scoop bucket sync from {} into {}",
                source,
                current_root.display()
            );
            tracing::info!("{line}");
            lines.push(line);
            if current_root.exists() {
                fs::remove_dir_all(&current_root).await.map_err(|err| {
                    BackendError::Other(format!(
                        "failed to remove {}: {err}",
                        current_root.display()
                    ))
                })?;
            }
            let mut clone_lines = fsx::copy_path_recursive(Path::new(&source), &current_root, None)
                .await
                .map_err(|err| err.context(format!("failed to update bucket '{}'", bucket.name)))
                .map(|_| {
                    vec![format!(
                        "> local bucket sync {} {} --branch {}",
                        Path::new(&source).display(),
                        current_root.display(),
                        branch
                    )]
                })?;
            clone_lines.push(format!(
                "Completed Scoop bucket sync into {}",
                current_root.display()
            ));
            clone_lines.push(format!("Updated bucket '{}'.", bucket.name));
            for line in clone_lines {
                tracing::info!("{line}");
                lines.push(line);
            }
            updated += 1;
            continue;
        }
        let temp_root = current_root.with_extension(format!("spoon-update-{}", std::process::id()));
        if temp_root.exists() {
            let _ = fs::remove_dir_all(&temp_root).await;
        }
        let line = format!("Updating bucket '{}' from {}...", bucket.name, source);
        tracing::info!("{line}");
        lines.push(line);
        let clone_lines =
            match clone_repo(&source, &temp_root, Some(branch), proxy, None, None).await {
                Ok(outcome) => {
                    let mut lines = vec![
                        format!(
                            "Starting Scoop bucket sync from {} into {}",
                            source,
                            current_root.display()
                        ),
                        format!("Fetching repository contents from {source}..."),
                        format!("Fetched repository contents from {source}"),
                    ];
                    if let Some(branch) = &outcome.head_branch {
                        lines.push(format!("Checked out branch: {branch}"));
                    }
                    if let Some(commit) = &outcome.head_commit {
                        lines.push(format!("HEAD at commit: {commit}"));
                    }
                    lines.push(format!(
                        "Completed Scoop bucket sync into {}",
                        current_root.display()
                    ));
                    lines
                }
                Err(err) => {
                    let _ = fs::remove_dir_all(&temp_root).await;
                    let line = format!("Failed to update bucket '{}': {err}", bucket.name);
                    tracing::info!("{line}");
                    lines.push(line);
                    continue;
                }
            };
        for line in clone_lines {
            tracing::info!("{line}");
            lines.push(line);
        }
        if current_root.exists() {
            fs::remove_dir_all(&current_root).await.map_err(|err| {
                BackendError::Other(format!(
                    "failed to remove {}: {err}",
                    current_root.display()
                ))
            })?;
        }
        fs::rename(&temp_root, &current_root).await.map_err(|err| {
            BackendError::Other(format!(
                "failed to replace {}: {err}",
                current_root.display()
            ))
        })?;
        let line = format!("Updated bucket '{}'.", bucket.name);
        tracing::info!("{line}");
        lines.push(line);
        updated += 1;
    }

    Ok((lines, BucketUpdateSummary { updated, skipped }))
}

pub async fn update_buckets_streaming_with_context<P>(
    context: &BackendContext<P>,
    names: &[String],
    emit: Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<(Vec<String>, BucketUpdateSummary)> {
    update_buckets_streaming(
        &context.root,
        names,
        context.proxy.as_deref().unwrap_or(""),
        emit,
    )
    .await
}

pub async fn update_buckets_streaming_outcome(
    tool_root: &Path,
    names: &[String],
    proxy: &str,
    emit: Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<ScoopBucketOperationOutcome> {
    let (mut output, summary) = update_buckets_streaming(tool_root, names, proxy, emit).await?;
    output.push(format!(
        "Bucket update summary: {} updated, {} skipped.",
        summary.updated, summary.skipped
    ));
    let buckets = load_buckets_from_registry(tool_root).await;
    Ok(ScoopBucketOperationOutcome {
        kind: "scoop_bucket_action",
        action: "update".to_string(),
        targets: names.to_vec(),
        status: CommandStatus::Success,
        title: "update Scoop buckets".to_string(),
        streamed: true,
        output,
        bucket_count: buckets.len(),
        buckets,
    })
}

pub async fn update_buckets_streaming_outcome_with_context<P>(
    context: &BackendContext<P>,
    names: &[String],
    emit: Option<&mut dyn FnMut(BackendEvent)>,
) -> Result<ScoopBucketOperationOutcome> {
    update_buckets_streaming_outcome(
        &context.root,
        names,
        context.proxy.as_deref().unwrap_or(""),
        emit,
    )
    .await
}
