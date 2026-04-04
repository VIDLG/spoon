use std::path::Path;

use tokio::fs;

use crate::layout::RuntimeLayout;
use crate::BackendContext;
use crate::{BackendError, CommandStatus, Result, clone_repo, fsx};

use super::models::{BucketSpec, ScoopBucketOperationOutcome, known_bucket_source};
use super::registry::{
    load_buckets_from_registry, remove_bucket_from_registry_record, sync_main_bucket_registry,
    upsert_bucket_to_registry,
};

pub async fn ensure_main_bucket_ready(tool_root: &Path, proxy: &str) -> Result<()> {
    let layout = RuntimeLayout::from_root(tool_root);
    let main_root = layout.scoop.bucket_root("main");
    let registry_has_main = load_buckets_from_registry(tool_root)
        .await
        .into_iter()
        .any(|bucket| bucket.name.eq_ignore_ascii_case("main"));
    if main_root.exists() {
        if !registry_has_main {
            sync_main_bucket_registry(tool_root).await?;
            tracing::info!("Registered default 'main' bucket");
        }
        return Ok(());
    }
    tracing::info!("Bootstrapping default 'main' bucket");
    let source = known_bucket_source("main").expect("main bucket must exist in known_buckets");
    add_bucket_to_registry(
        tool_root,
        &BucketSpec {
            name: "main".to_string(),
            source: Some(source),
            branch: Some("master".to_string()),
        },
        proxy,
    )
    .await?;
    Ok(())
}

pub async fn ensure_main_bucket_ready_with_context<P>(context: &BackendContext<P>) -> Result<()> {
    ensure_main_bucket_ready(&context.root, context.proxy.as_deref().unwrap_or("")).await
}

pub async fn add_bucket_to_registry(
    tool_root: &Path,
    spec: &BucketSpec,
    proxy: &str,
) -> Result<()> {
    let name = &spec.name;
    if name.trim().is_empty() {
        return Err(BackendError::Config(
            "bucket name must not be empty".to_string(),
        ));
    }
    let bucket_dir = RuntimeLayout::from_root(tool_root).scoop.bucket_root(name);
    if bucket_dir.exists() {
        return Err(BackendError::Other(format!(
            "bucket '{}' already exists",
            name
        )));
    }
    let local_directory_source = spec
        .source
        .as_ref()
        .map(|s| Path::new(s).is_dir())
        .unwrap_or(false);
    if let Some(parent) = bucket_dir.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|err| BackendError::fs("create", parent, err))?;
    }
    if local_directory_source {
        let source = spec.source.as_ref().expect("local source must exist");
        fsx::copy_path_recursive(Path::new(source), &bucket_dir, None)
            .await
            .map_err(|err| err.context(format!("failed to clone bucket '{}'", name)))?;
        upsert_bucket_to_registry(tool_root, spec).await?;
        tracing::info!(
            "Synced local bucket '{}' into {}",
            source,
            bucket_dir.display()
        );
        tracing::info!("Registered bucket '{}'", name);
        return Ok(());
    }

    let temp_dir = bucket_dir.with_extension(format!("spoon-add-{}", std::process::id()));
    if temp_dir.exists() {
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    let source = spec
        .source
        .as_ref()
        .ok_or_else(|| BackendError::Config("bucket source must not be empty".to_string()))?;
    let branch = spec.branch.as_deref().unwrap_or("master");

    match clone_repo(source, &temp_dir, Some(branch), proxy, None, None).await {
        Ok(outcome) => {
            tracing::info!("Fetching repository contents from {source}...");
            if let Some(branch) = &outcome.head_branch {
                tracing::info!("Checked out branch: {branch}");
            }
            if let Some(commit) = &outcome.head_commit {
                tracing::info!("HEAD at commit: {commit}");
            }
        }
        Err(err) => {
            let _ = fs::remove_dir_all(&temp_dir).await;
            return Err(BackendError::Other(format!(
                "failed to clone bucket '{}': {}",
                name, err
            )));
        }
    };
    if !temp_dir.join(".git").exists() {
        let _ = fs::remove_dir_all(&temp_dir).await;
        return Err(BackendError::Other(format!(
            "failed to clone bucket '{}'",
            name
        )));
    }
    if bucket_dir.exists() {
        fs::remove_dir_all(&bucket_dir)
            .await
            .map_err(|err| BackendError::fs("clear", &bucket_dir, err))?;
    }
    fs::rename(&temp_dir, &bucket_dir).await.map_err(|err| {
        BackendError::Other(format!(
            "failed to finalize {}: {err}",
            bucket_dir.display()
        ))
    })?;
    upsert_bucket_to_registry(tool_root, spec).await?;
    tracing::info!("Registered bucket '{}'", name);
    Ok(())
}

pub async fn add_bucket_to_registry_with_context<P>(
    context: &BackendContext<P>,
    spec: &BucketSpec,
) -> Result<()> {
    add_bucket_to_registry(&context.root, spec, context.proxy.as_deref().unwrap_or("")).await
}

pub async fn add_bucket_to_registry_outcome(
    tool_root: &Path,
    spec: &BucketSpec,
    proxy: &str,
) -> Result<ScoopBucketOperationOutcome> {
    add_bucket_to_registry(tool_root, spec, proxy).await?;
    let buckets = load_buckets_from_registry(tool_root).await;
    Ok(ScoopBucketOperationOutcome {
        kind: "scoop_bucket_action",
        action: "add".to_string(),
        targets: vec![spec.name.clone()],
        status: CommandStatus::Success,
        title: format!("add Scoop bucket {}", spec.name),
        streamed: false,
        output: vec![],
        bucket_count: buckets.len(),
        buckets,
    })
}

pub async fn remove_bucket_from_registry(tool_root: &Path, name: &str) -> Result<Vec<String>> {
    if name.eq_ignore_ascii_case("main") {
        return Err(BackendError::Config(
            "bucket 'main' cannot be removed".to_string(),
        ));
    }
    let bucket_dir = RuntimeLayout::from_root(tool_root).scoop.bucket_root(name);
    if !bucket_dir.exists() {
        return Err(BackendError::Other(format!(
            "bucket '{}' is not installed",
            name
        )));
    }
    fs::remove_dir_all(&bucket_dir)
        .await
        .map_err(|err| BackendError::fs("remove", &bucket_dir, err))?;
    remove_bucket_from_registry_record(tool_root, name).await?;
    Ok(vec![
        format!("Removed bucket directory: {}", bucket_dir.display()),
        format!("Removed bucket '{}'.", name),
    ])
}

pub async fn remove_bucket_from_registry_outcome(
    tool_root: &Path,
    name: &str,
) -> Result<ScoopBucketOperationOutcome> {
    let output = remove_bucket_from_registry(tool_root, name).await?;
    let buckets = load_buckets_from_registry(tool_root).await;
    Ok(ScoopBucketOperationOutcome {
        kind: "scoop_bucket_action",
        action: "remove".to_string(),
        targets: vec![name.to_string()],
        status: CommandStatus::Success,
        title: format!("remove Scoop bucket {name}"),
        streamed: false,
        output,
        bucket_count: buckets.len(),
        buckets,
    })
}
