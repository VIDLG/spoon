use std::path::{Path, PathBuf};

use rusqlite::params;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::control_plane::ControlPlaneDb;
use super::paths::{
    scoop_bucket_root, scoop_buckets_root,
};
use crate::layout::RuntimeLayout;
use crate::BackendContext;
use crate::{BackendError, BackendEvent, CommandStatus, Result, clone_repo, fsx};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bucket {
    pub name: String,
    pub source: String,
    pub branch: String,
}

#[derive(Debug, Clone)]
pub struct BucketSpec {
    pub name: String,
    pub source: Option<String>,
    pub branch: Option<String>,
}

impl BucketSpec {
    /// Resolve a partial spec to a complete bucket.
    /// Source is filled from known buckets if missing.
    /// Branch defaults to "master" if missing.
    pub fn resolve(&self) -> Result<Bucket> {
        let source = match &self.source {
            Some(s) => s.clone(),
            None => known_bucket_source(&self.name)
                .ok_or_else(|| BackendError::Config(format!("unknown bucket: {}", self.name)))?,
        };
        let branch = self.branch.clone().unwrap_or_else(|| "master".to_string());
        Ok(Bucket {
            name: self.name.clone(),
            source,
            branch,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedBucket {
    pub bucket: Bucket,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BucketUpdateSummary {
    pub updated: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoopBucketInventory {
    pub kind: &'static str,
    pub success: bool,
    pub bucket_count: usize,
    pub buckets: Vec<Bucket>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoopBucketOperationOutcome {
    pub kind: &'static str,
    pub action: String,
    pub targets: Vec<String>,
    pub status: CommandStatus,
    pub title: String,
    pub streamed: bool,
    pub output: Vec<String>,
    pub bucket_count: usize,
    pub buckets: Vec<Bucket>,
}

impl ScoopBucketOperationOutcome {
    pub const fn is_success(&self) -> bool {
        self.status.is_success()
    }
}

const KNOWN_BUCKETS: &[(&str, &str)] = &[
    ("main", "https://github.com/ScoopInstaller/Main"),
    ("extras", "https://github.com/ScoopInstaller/Extras"),
    ("versions", "https://github.com/ScoopInstaller/Versions"),
    ("nirsoft", "https://github.com/ScoopInstaller/Nirsoft"),
    (
        "sysinternals",
        "https://github.com/niheaven/scoop-sysinternals",
    ),
    ("php", "https://github.com/ScoopInstaller/PHP"),
    (
        "nerd-fonts",
        "https://github.com/matthewjberger/scoop-nerd-fonts",
    ),
    (
        "nonportable",
        "https://github.com/ScoopInstaller/Nonportable",
    ),
    ("java", "https://github.com/ScoopInstaller/Java"),
    ("games", "https://github.com/Calinou/scoop-games"),
];

pub fn known_bucket_source(name: &str) -> Option<String> {
    let env_var = format!(
        "SPOON_TEST_SCOOP_BUCKET_{}_SOURCE",
        name.to_ascii_uppercase().replace('-', "_")
    );
    std::env::var(&env_var).ok().or_else(|| {
        KNOWN_BUCKETS
            .iter()
            .find(|(known, _)| known.eq_ignore_ascii_case(name))
            .map(|(_, source)| (*source).to_string())
    })
}

pub async fn load_buckets_from_registry(tool_root: &Path) -> Vec<Bucket> {
    let layout = RuntimeLayout::from_root(tool_root);
    let Ok(db) = ControlPlaneDb::open_for_layout(&layout).await else {
        return Vec::new();
    };
    db.call(|conn| {
        let mut stmt = conn.prepare(
            "SELECT name, remote_url, branch FROM bucket_registry ORDER BY rowid",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Bucket {
                name: row.get(0)?,
                source: row.get(1)?,
                branch: row.get(2)?,
            })
        })?;
        Ok(rows.filter_map(|row| row.ok()).collect::<Vec<_>>())
    })
    .await
    .unwrap_or_default()
}

pub async fn upsert_bucket_to_registry(tool_root: &Path, spec: &BucketSpec) -> Result<Vec<Bucket>> {
    let bucket = spec.resolve()?;
    let layout = RuntimeLayout::from_root(tool_root);
    let db = ControlPlaneDb::open_for_layout(&layout).await?;
    let bucket_name = bucket.name.clone();
    let source = bucket.source.clone();
    let branch = bucket.branch.clone();
    db.call_write(move |conn| {
        conn.execute(
            "INSERT INTO bucket_registry (name, remote_url, branch)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(name) DO UPDATE SET
                remote_url = excluded.remote_url,
                branch = excluded.branch",
            params![bucket_name, source, branch],
        )?;
        Ok(())
    })
    .await?;
    Ok(load_buckets_from_registry(tool_root).await)
}

pub async fn remove_bucket_from_registry_record(
    tool_root: &Path,
    name: &str,
) -> Result<Vec<Bucket>> {
    let layout = RuntimeLayout::from_root(tool_root);
    let db = ControlPlaneDb::open_for_layout(&layout).await?;
    let name = name.to_string();
    db.call_write(move |conn| {
        conn.execute(
            "DELETE FROM bucket_registry WHERE lower(name) = lower(?1)",
            params![name],
        )?;
        Ok(())
    })
    .await?;
    Ok(load_buckets_from_registry(tool_root).await)
}

pub async fn sync_main_bucket_registry(tool_root: &Path) -> Result<()> {
    upsert_bucket_to_registry(
        tool_root,
        &BucketSpec {
            name: "main".to_string(),
            source: None,
            branch: None,
        },
    )
    .await?;
    Ok(())
}

pub async fn ensure_main_bucket_ready(tool_root: &Path, proxy: &str) -> Result<()> {
    let main_root = scoop_bucket_root(tool_root, "main");
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

pub async fn ensure_main_bucket_ready_with_context<P>(context: &BackendContext<P>) -> Result<()>
{
    ensure_main_bucket_ready(&context.root, context.proxy.as_deref().unwrap_or("")).await
}

pub async fn resolve_manifest(tool_root: &Path, package_name: &str) -> Option<ResolvedBucket> {
    let buckets = load_buckets_from_registry(tool_root).await;
    for bucket in buckets {
        let manifest_path = scoop_buckets_root(tool_root)
            .join(&bucket.name)
            .join("bucket")
            .join(format!("{package_name}.json"));
        if tokio::fs::metadata(&manifest_path).await.is_ok() {
            return Some(ResolvedBucket {
                bucket,
                manifest_path,
            });
        }
    }
    None
}

pub fn resolve_manifest_sync(tool_root: &Path, package_name: &str) -> Option<ResolvedBucket> {
    let layout = RuntimeLayout::from_root(tool_root);
    let conn = rusqlite::Connection::open(crate::control_plane::sqlite::db_path_for_layout(&layout))
        .ok()?;
    let mut stmt = conn
        .prepare("SELECT name, remote_url, branch FROM bucket_registry ORDER BY rowid")
        .ok()?;
    let buckets = stmt
        .query_map([], |row| {
            Ok(Bucket {
                name: row.get(0)?,
                source: row.get(1)?,
                branch: row.get(2)?,
            })
        })
        .ok()?
        .filter_map(|row| row.ok())
        .collect::<Vec<_>>();

    for bucket in buckets {
        let manifest_path = scoop_buckets_root(tool_root)
            .join(&bucket.name)
            .join("bucket")
            .join(format!("{package_name}.json"));
        if manifest_path.exists() {
            return Some(ResolvedBucket {
                bucket,
                manifest_path,
            });
        }
    }
    None
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
    let bucket_dir = scoop_buckets_root(tool_root).join(name);
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
        let source = spec.source.as_ref().unwrap();
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
    if !local_directory_source && !temp_dir.join(".git").exists() {
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
) -> Result<()>
{
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
    let bucket_dir = scoop_buckets_root(tool_root).join(name);
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
) -> Result<(Vec<String>, BucketUpdateSummary)>
{
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
        let current_root = scoop_bucket_root(tool_root, &bucket.name);
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
