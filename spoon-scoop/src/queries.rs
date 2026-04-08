use std::path::Path;

use async_recursion::async_recursion;
use spoon_core::{RuntimeLayout, ScoopLayout};

use crate::*;

/// Load installed packages from filesystem (reads apps directory).
pub async fn installed_packages(layout: &ScoopLayout) -> Vec<InstalledPackageState> {
    let mut packages = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(&layout.apps_root).await else {
        return packages;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip non-directories and special entries
        let Ok(meta) = entry.metadata().await else { continue };
        if !meta.is_dir() { continue; }

        // Find current version (current symlink or junction)
        let current_root = layout.package_current_root(&name);
        let Ok(current_meta) = tokio::fs::symlink_metadata(&current_root).await else { continue };
        if !current_meta.file_type().is_symlink() { continue; }

        // Read the target version from symlink
        let Ok(target) = tokio::fs::read_link(&current_root).await else { continue };
        let version = target.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Try to load install.json if exists
        let install_json = current_root.join("install.json");
        let state = if let Ok(content) = tokio::fs::read_to_string(&install_json).await {
            serde_json::from_str::<InstalledPackageState>(&content).ok()
        } else {
            None
        };

        let state = state.unwrap_or_else(|| InstalledPackageState {
            identity: InstalledPackageIdentity {
                package: name.clone(),
                version: version.clone(),
                bucket: "unknown".to_string(),
                architecture: None,
                cache_size_bytes: None,
            },
            command_surface: InstalledPackageCommandSurface::default(),
            integrations: Vec::new(),
            uninstall: InstalledPackageUninstall::default(),
        });

        packages.push(state);
    }
    packages
}

/// Get package info from manifest + installed state.
pub async fn package_info<D: Default + Clone>(
    tool_root: &Path,
    package_name: &str,
) -> ScoopPackageDetailsOutcome<D> {
    let layout = RuntimeLayout::from_root(tool_root);

    // Find manifest in buckets
    let manifest = resolve_package_manifest(&layout.scoop, package_name).await;
    let installed_state = read_installed_state(&layout.scoop, package_name).await.ok().flatten();

    match (manifest, installed_state) {
        (Some(resolved), installed) => {
            let scoop_manifest = load_manifest(&resolved.manifest_path).await;
            let installed_version = installed.as_ref().map(|s| s.version().to_string());
            let current = layout.scoop.package_current_root(package_name).display().to_string();

            let installed_size_bytes = if installed.is_some() {
                Some(directory_size_recursive(&layout.scoop.package_current_root(package_name)).await)
            } else {
                None
            };

            let metadata = ScoopPackageMetadata {
                name: package_name.to_string(),
                bucket: resolved.bucket.name.clone(),
                latest_version: scoop_manifest.as_ref().and_then(|m| m.version.clone()),
                description: scoop_manifest.as_ref().and_then(|m| m.description.clone()),
                homepage: scoop_manifest.as_ref().and_then(|m| m.homepage.clone()),
                manifest: resolved.manifest_path.display().to_string(),
                license: scoop_manifest.as_ref().and_then(|m| m.license.as_ref().map(|l| l.identifier().to_string())),
                depends: scoop_manifest.as_ref().and_then(|m| serde_json::to_value(&m.depends).ok()),
                suggest: scoop_manifest.as_ref().and_then(|m| serde_json::to_value(&m.suggest).ok()),
                extract_dir: scoop_manifest.as_ref().and_then(|m| m.extract_dir.as_ref().and_then(|v| serde_json::to_value(v).ok())),
                extract_to: scoop_manifest.as_ref().and_then(|m| m.extract_to.as_ref().and_then(|v| serde_json::to_value(v).ok())),
                notes: scoop_manifest.as_ref().map(|m| m.notes.as_ref().map(|n| n.lines().iter().map(|s| s.to_string()).collect()).unwrap_or_default()).unwrap_or_default(),
                download_urls: scoop_manifest.as_ref().map(|m| m.url.as_ref().map(|u| u.to_vec()).unwrap_or_default()).unwrap_or_default(),
            };

            let install = ScoopPackageInstall {
                installed: installed.is_some(),
                installed_version,
                current,
                installed_size_bytes,
                cache_size_bytes: None,
                bins: installed.as_ref().map(|s| s.command_surface.bins.clone()).unwrap_or_default(),
                state: installed.as_ref().and_then(|s| s.identity.architecture.clone()),
                persist_root: Some(layout.scoop.package_persist_root(package_name).display().to_string()),
            };

            let integration = ScoopPackageIntegration {
                commands: ScoopCommandIntegration {
                    shims: if installed.is_some() {
                        Some(vec![layout.shims.display().to_string()])
                    } else {
                        None
                    },
                },
                environment: ScoopEnvironmentIntegration {
                    add_path: installed.as_ref().map(|s| s.command_surface.env_add_path.clone()).unwrap_or_default(),
                    set: installed.as_ref().map(|s| s.command_surface.env_set.iter().map(|(k, v)| format!("{k}={v}")).collect()).unwrap_or_default(),
                    persist: None,
                },
                system: ScoopSystemIntegration {
                    shortcuts: installed.as_ref().map(|s| s.command_surface.shortcuts.iter().map(|sc| sc.name.clone()).collect()).unwrap_or_default(),
                },
                policy: ScoopPolicyIntegration {
                    desired: Vec::new(),
                    applied_values: Vec::new(),
                    config_files: Vec::new(),
                    config_directories: Vec::new(),
                },
            };

            ScoopPackageDetailsOutcome::Details(ScoopPackageDetails {
                kind: "package_info",
                success: true,
                package: metadata,
                install,
                integration,
            })
        }
        (None, _) => {
            ScoopPackageDetailsOutcome::Error(ScoopPackageDetailsError {
                kind: "package_info",
                success: false,
                package: package_name.to_string(),
                error: ScoopPackageError {
                    message: format!("package '{}' not found in any bucket", package_name),
                },
            })
        }
    }
}

/// Resolve package manifest across all buckets.
pub async fn resolve_package_manifest(layout: &ScoopLayout, package_name: &str) -> Option<ResolvedBucket> {
    let buckets = load_buckets_from_filesystem(layout).await;

    for bucket in buckets {
        let manifest_path = layout.bucket_root(&bucket.name).join("bucket").join(format!("{}.json", package_name));
        if manifest_path.exists() {
            return Some(ResolvedBucket { bucket, manifest_path });
        }
    }
    None
}

/// Load buckets from filesystem (buckets directory).
pub async fn load_buckets_from_filesystem(layout: &crate::bucket::ScoopLayout) -> Vec<Bucket> {
    let mut buckets = Vec::new();
    let Ok(mut entries) = tokio::fs::read_dir(&layout.buckets_root).await else {
        return buckets;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        let Ok(meta) = entry.metadata().await else { continue };
        if !meta.is_dir() { continue; }

        // Try to read .git/config for source
        let git_config = layout.bucket_root(&name).join(".git").join("config");
        let (source, branch) = if let Ok(content) = tokio::fs::read_to_string(&git_config).await {
            parse_git_config(&content)
        } else {
            ("unknown".to_string(), "master".to_string())
        };

        buckets.push(Bucket { name, source, branch });
    }
    buckets
}

fn parse_git_config(content: &str) -> (String, String) {
    let mut source = "unknown".to_string();
    let mut branch = "master".to_string();

    let mut in_remote_origin = false;
    for line in content.lines() {
        let line = line.trim();
        if line == "[remote \"origin\"]" {
            in_remote_origin = true;
        } else if line.starts_with('[') {
            in_remote_origin = false;
        } else if in_remote_origin && line.starts_with("url = ") {
            source = line.strip_prefix("url = ").unwrap_or("").to_string();
        }

        if line.starts_with("[branch") {
            if let Some(b) = line.strip_prefix("[branch \"").and_then(|s| s.strip_suffix("\"]")) {
                branch = b.to_string();
            }
        }
    }
    (source, branch)
}

#[async_recursion(?Send)]
async fn directory_size_recursive(path: &Path) -> u64 {
    let Ok(meta) = tokio::fs::symlink_metadata(path).await else { return 0 };
    if meta.file_type().is_symlink() { return 0; }
    if meta.is_file() { return meta.len(); }

    let mut total = 0u64;
    let Ok(mut entries) = tokio::fs::read_dir(path).await else { return 0 };
    while let Ok(Some(entry)) = entries.next_entry().await {
        total += directory_size_recursive(&entry.path()).await;
    }
    total
}

/// Get runtime status.
pub async fn runtime_status(tool_root: &Path) -> ScoopStatus {
    let layout = RuntimeLayout::from_root(tool_root);
    let buckets = load_buckets_from_filesystem(&layout.scoop).await;
    let packages = installed_packages(&layout.scoop).await;

    ScoopStatus {
        kind: "scoop_status",
        success: true,
        runtime: ScoopRuntimeStatus {
            root: tool_root.display().to_string(),
            shims: layout.shims.display().to_string(),
        },
        buckets,
        installed_packages: packages.iter().map(|p| InstalledPackageSummary {
            name: p.identity.package.clone(),
            version: p.identity.version.clone(),
        }).collect(),
        paths: ScoopPaths {
            apps: layout.scoop.apps_root.display().to_string(),
            cache: layout.scoop.cache_root.display().to_string(),
            persist: layout.scoop.persist_root.display().to_string(),
            state: layout.scoop.root.join("state").display().to_string(),
        },
    }
}

/// Search for packages in local buckets.
pub async fn search_results(tool_root: &Path, query: Option<&str>) -> ScoopSearchResults {
    let layout = RuntimeLayout::from_root(tool_root);
    let buckets = load_buckets_from_filesystem(&layout.scoop).await;
    let query_lower = query.map(|q| q.to_lowercase());

    let mut matches = Vec::new();

    for bucket in buckets {
        let bucket_path = layout.scoop.bucket_root(&bucket.name).join("bucket");
        let Ok(mut entries) = tokio::fs::read_dir(&bucket_path).await else { continue };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".json") { continue; }
            let package_name = name.trim_end_matches(".json").to_string();

            // Filter by query if provided
            if let Some(ref q) = query_lower {
                if !package_name.to_lowercase().contains(q) { continue; }
            }

            // Load manifest for details
            let manifest = load_manifest(&entry.path()).await;
            matches.push(ScoopSearchMatch {
                package_name,
                bucket: bucket.name.clone(),
                version: manifest.as_ref().and_then(|m| m.version.clone()),
                description: manifest.as_ref().and_then(|m| m.description.clone()),
                homepage: manifest.as_ref().and_then(|m| m.homepage.clone()),
            });
        }
    }

    ScoopSearchResults {
        kind: "search_results",
        success: true,
        query: query.map(|s| s.to_string()),
        matches,
    }
}

/// Run doctor diagnostics: ensure directories, ensure main bucket, check runtime state.
pub async fn doctor(tool_root: &Path) -> ScoopDoctorDetails {
    let layout = RuntimeLayout::from_root(tool_root);

    // Ensure all required directories exist
    let required_dirs = vec![
        layout.scoop.root.clone(),
        layout.scoop.apps_root.clone(),
        layout.scoop.buckets_root.clone(),
        layout.scoop.cache_root.clone(),
        layout.scoop.persist_root.clone(),
        layout.scoop.state_root.clone(),
        layout.shims.clone(),
    ];

    let mut ensured_paths = Vec::new();
    let mut issues = Vec::new();

    for dir in &required_dirs {
        if let Err(err) = tokio::fs::create_dir_all(dir).await {
            issues.push(format!("Failed to create directory {}: {err}", dir.display()));
        }
        ensured_paths.push(dir.display().to_string());
    }

    // Ensure main bucket is registered
    if let Err(err) = ensure_main_bucket_ready(&layout.scoop.root).await {
        issues.push(format!("Failed to ensure main bucket: {err}"));
    }

    let registered_buckets = load_buckets_from_filesystem(&layout.scoop).await;

    if registered_buckets.is_empty() {
        issues.push("No Scoop buckets are registered.".to_string());
    }

    ScoopDoctorDetails {
        kind: "scoop_doctor",
        success: issues.is_empty(),
        runtime: ScoopRuntimeDetails {
            root: layout.scoop.root.display().to_string(),
            state_root: layout.scoop.state_root.display().to_string(),
            shims_root: layout.shims.display().to_string(),
        },
        ensured_paths,
        registered_buckets,
        issues,
    }
}

/// Load installed packages (tool_root variant).
pub async fn installed_package_states(tool_root: &Path) -> Vec<InstalledPackageState> {
    let layout = RuntimeLayout::from_root(tool_root);
    installed_packages(&layout.scoop).await
}

/// Load installed packages with optional filter.
pub async fn installed_package_states_filtered<F>(
    tool_root: &Path,
    filter: Option<F>,
) -> Vec<InstalledPackageState>
where
    F: FnMut(&InstalledPackageState) -> bool,
{
    let mut packages = installed_package_states(tool_root).await;
    if let Some(f) = filter {
        packages.retain(f);
    }
    packages
}

/// Get package manifest content.
pub async fn package_manifest(tool_root: &Path, package_name: &str) -> ScoopPackageManifestOutcome {
    let layout = RuntimeLayout::from_root(tool_root);
    let resolved = resolve_package_manifest(&layout.scoop, package_name).await;

    match resolved {
        Some(resolved) => {
            let content = tokio::fs::read_to_string(&resolved.manifest_path).await;
            match content {
                Ok(content) => ScoopPackageManifestOutcome {
                    kind: "package_manifest",
                    package: package_name.to_string(),
                    status: spoon_core::CommandStatus::Success,
                    title: format!("Manifest for {}", package_name),
                    manifest_path: Some(resolved.manifest_path.display().to_string()),
                    content: Some(content),
                    error: None,
                    streamed: false,
                },
                Err(e) => ScoopPackageManifestOutcome {
                    kind: "package_manifest",
                    package: package_name.to_string(),
                    status: spoon_core::CommandStatus::Failed,
                    title: format!("Failed to read manifest for {}", package_name),
                    manifest_path: Some(resolved.manifest_path.display().to_string()),
                    content: None,
                    error: Some(ScoopPackageError {
                        message: e.to_string(),
                    }),
                    streamed: false,
                },
            }
        }
        None => ScoopPackageManifestOutcome {
            kind: "package_manifest",
            package: package_name.to_string(),
            status: spoon_core::CommandStatus::Failed,
            title: format!("Package {} not found", package_name),
            manifest_path: None,
            content: None,
            error: Some(ScoopPackageError {
                message: format!("package '{}' not found in any bucket", package_name),
            }),
            streamed: false,
        },
    }
}