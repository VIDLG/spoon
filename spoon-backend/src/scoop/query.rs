use std::path::Path;

use super::buckets::{self, Bucket};
use super::manifest::{self, ScoopManifest};
use super::state::{InstalledPackageState, InstalledPackageSummary};
use crate::layout::RuntimeLayout;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ScoopRuntimeStatus {
    pub root: String,
    pub shims: String,
}

#[derive(Debug, Serialize)]
pub struct ScoopPaths {
    pub apps: String,
    pub cache: String,
    pub persist: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct ScoopStatus {
    pub kind: &'static str,
    pub success: bool,
    pub runtime: ScoopRuntimeStatus,
    pub buckets: Vec<Bucket>,
    pub installed_packages: Vec<InstalledPackageSummary>,
    pub paths: ScoopPaths,
}

#[derive(Debug, Serialize)]
pub struct ScoopSearchMatch {
    pub package_name: String,
    pub bucket: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScoopSearchResults {
    pub kind: &'static str,
    pub success: bool,
    pub query: Option<String>,
    pub matches: Vec<ScoopSearchMatch>,
}

/// Enumerate all canonical installed states through the store, sorted by
/// package name.
pub async fn installed_package_states(tool_root: &Path) -> Vec<InstalledPackageState> {
    let layout = RuntimeLayout::from_root(tool_root);
    super::state::list_all_installed_states(&layout).await
}

/// Enumerate all canonical installed states through the store, applying an
/// optional filter, sorted by package name.
pub async fn installed_package_states_filtered<F>(
    tool_root: &Path,
    filter: Option<F>,
) -> Vec<InstalledPackageState>
where
    F: FnMut(&InstalledPackageState) -> bool,
{
    let layout = RuntimeLayout::from_root(tool_root);
    super::state::list_installed_states_filtered(&layout, filter).await
}

async fn search_manifests_local_async(
    tool_root: &Path,
    query: &str,
) -> Vec<(String, String, ScoopManifest)> {
    let query = query.trim().to_ascii_lowercase();
    let mut matches = Vec::new();
    let layout = RuntimeLayout::from_root(tool_root);
    for bucket in buckets::load_buckets_from_registry(tool_root).await {
        let bucket_root = layout.scoop.bucket_root(&bucket.name).join("bucket");
        let Ok(mut entries) = tokio::fs::read_dir(&bucket_root).await else {
            continue;
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let Some(package_name) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            let Some(doc) = manifest::load_manifest(&path).await else {
                continue;
            };
            let matches_query = query.is_empty()
                || package_name.to_ascii_lowercase().contains(&query)
                || doc
                    .description
                    .as_deref()
                    .map(|value| value.to_ascii_lowercase().contains(&query))
                    .unwrap_or(false);
            if matches_query {
                matches.push((package_name.to_string(), bucket.name.clone(), doc));
            }
        }
    }
    matches.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    matches
}

pub async fn runtime_status(tool_root: &Path) -> ScoopStatus {
    let layout = RuntimeLayout::from_root(tool_root);
    let buckets = buckets::load_buckets_from_registry(tool_root).await;
    let summaries = super::state::list_installed_summaries(&layout).await;
    ScoopStatus {
        kind: "scoop_status",
        success: true,
        runtime: ScoopRuntimeStatus {
            root: layout.scoop.root.display().to_string(),
            shims: layout.shims.display().to_string(),
        },
        buckets,
        installed_packages: summaries,
        paths: ScoopPaths {
            apps: layout.scoop.apps_root.display().to_string(),
            cache: layout.scoop.cache_root.display().to_string(),
            persist: layout.scoop.persist_root.display().to_string(),
            state: layout.scoop.state_root.display().to_string(),
        },
    }
}

pub async fn search_results(tool_root: &Path, query: Option<&str>) -> ScoopSearchResults {
    let search_query = query.unwrap_or("");
    let matches = {
        let matches = manifest::search_manifests_async(search_query, tool_root).await;
        if matches.is_empty() {
            search_manifests_local_async(tool_root, search_query).await
        } else {
            matches
        }
    };
    ScoopSearchResults {
        kind: "scoop_search",
        success: true,
        query: query
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string),
        matches: matches
            .into_iter()
            .map(|item| ScoopSearchMatch {
                package_name: item.0,
                bucket: item.1,
                version: item.2.version,
                description: item.2.description,
                homepage: item.2.homepage,
            })
            .collect(),
    }
}
