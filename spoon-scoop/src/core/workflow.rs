//! Scoop package lifecycle workflows.
//!
//! Merged from the original step decomposition into a single module for clarity.
//! Each step is a separate function, callable from install/uninstall workflows.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use spoon_core::{EventSender, ScoopLayout};

use crate::core::*;
use crate::error::Result;
use crate::ScoopError;

// ── Package action ──

/// The kind of lifecycle action to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScoopPackageAction {
    Install,
    Update,
    Uninstall,
    Reapply,
    Other,
}

impl ScoopPackageAction {
    pub fn from_str(s: &str) -> Self {
        match s {
            "install" => Self::Install,
            "update" => Self::Update,
            "uninstall" => Self::Uninstall,
            "reapply" => Self::Reapply,
            _ => Self::Other,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Update => "update",
            Self::Uninstall => "uninstall",
            Self::Reapply => "reapply",
            Self::Other => "run",
        }
    }

    pub fn build_args(self, package_name: &str, fallback_action: &str) -> Vec<String> {
        match self {
            Self::Install => vec![
                "install".to_string(),
                package_name.to_string(),
                "--no-update-scoop".to_string(),
            ],
            Self::Update => vec!["update".to_string(), package_name.to_string()],
            Self::Uninstall => vec!["uninstall".to_string(), package_name.to_string()],
            Self::Reapply => vec!["reapply".to_string(), package_name.to_string()],
            Self::Other => vec![fallback_action.to_string(), package_name.to_string()],
        }
    }

    pub fn should_resolve_manifest(self) -> bool {
        matches!(self, Self::Install | Self::Update)
    }
}

// ── Package plan ──

/// A planned package lifecycle action.
#[derive(Debug, Clone)]
pub struct ScoopPackagePlan {
    pub action: ScoopPackageAction,
    pub display_name: String,
    pub package_name: String,
    pub args: Vec<String>,
    pub resolved_manifest: Option<ResolvedBucket>,
}

impl ScoopPackagePlan {
    pub fn title(&self) -> String {
        format!("{} {}", self.action.as_str(), self.display_name)
    }

    pub fn command_line(&self) -> String {
        format!(
            "Planned Spoon package action (Scoop): {}",
            self.args.join(" ")
        )
    }

    pub fn resolution_line(&self) -> Option<String> {
        let resolved = self.resolved_manifest.as_ref()?;
        let branch = if resolved.bucket.branch.trim().is_empty() {
            "master"
        } else {
            &resolved.bucket.branch
        };
        Some(format!(
            "Resolved Scoop package '{}' from bucket '{}' [{branch}] at {}.",
            self.package_name, resolved.bucket.name, resolved.bucket.source
        ))
    }
}

/// Create a package plan from action and arguments.
pub fn plan_package_action(action: ScoopPackageAction, args: &[String]) -> Result<ScoopPackagePlan> {
    let (package_name, display_name) = match args.first() {
        Some(p) => (p.clone(), p.clone()),
        None => return Err(ScoopError::Config("package name required".to_string())),
    };

    Ok(ScoopPackagePlan {
        action,
        display_name,
        package_name,
        args: args.to_vec(),
        resolved_manifest: None,
    })
}

/// Create a package plan from action string, display name, package name, and optional tool root.
/// This matches the backend's `plan_package_action` signature for migration compatibility.
pub fn plan_package_action_with_display(
    action: &str,
    display_name: &str,
    package_name: &str,
    tool_root: Option<&Path>,
) -> ScoopPackagePlan {
    let action_kind = ScoopPackageAction::from_str(action);
    let resolved_manifest = tool_root
        .filter(|_| action_kind.should_resolve_manifest())
        .and_then(|root| crate::core::manifest::resolve_manifest_sync(root, package_name));
    let args = action_kind.build_args(package_name, action);
    ScoopPackagePlan {
        action: action_kind,
        display_name: display_name.to_string(),
        package_name: package_name.to_string(),
        args,
        resolved_manifest,
    }
}

// ── State I/O ──

/// Read installed state from install.json in the current version directory.
pub async fn read_installed_state(layout: &ScoopLayout, package_name: &str) -> Result<Option<InstalledPackageState>> {
    let current_root = layout.package_current_root(package_name);
    let install_json = current_root.join("install.json");
    if !install_json.exists() {
        return Ok(None);
    }
    let content = tokio::fs::read_to_string(&install_json)
        .await
        .map_err(|e| ScoopError::fs("read", &install_json, e))?;
    let state: InstalledPackageState = serde_json::from_str(&content)
        .map_err(ScoopError::from)?;
    Ok(Some(state))
}

/// Write installed state to install.json.
pub async fn write_installed_state(layout: &ScoopLayout, state: &InstalledPackageState) -> Result<()> {
    let current_root = layout.package_current_root(state.package());
    let install_json = current_root.join("install.json");
    let content = serde_json::to_string_pretty(state)
        .map_err(ScoopError::from)?;
    tokio::fs::write(&install_json, content)
        .await
        .map_err(|e| ScoopError::fs("write", &install_json, e))?;
    Ok(())
}

/// Remove installed state (install.json).
pub async fn remove_installed_state(layout: &ScoopLayout, package_name: &str) -> Result<()> {
    let install_json = layout.package_current_root(package_name).join("install.json");
    if install_json.exists() {
        tokio::fs::remove_file(&install_json)
            .await
            .map_err(|e| ScoopError::fs("remove", &install_json, e))?;
    }
    Ok(())
}

// ── Step 1: Acquire (download) ──

/// Download package assets to cache.
pub async fn acquire_assets(
    layout: &ScoopLayout,
    client: &reqwest::Client,
    package_name: &str,
    version: &str,
    assets: &[PackageAsset],
    _proxy: &str,
    cancel: Option<&spoon_core::CancellationToken>,
    events: Option<&EventSender>,
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for asset in assets {
        let cache_file = layout.package_cache_file(
            package_name,
            version,
            asset.target_name.as_deref().unwrap_or("archive"),
        );
        if cache_file.exists() {
            paths.push(cache_file);
            continue;
        }

        if let Some(sender) = events {
            sender.send(spoon_core::BackendEvent::Stage(spoon_core::StageEvent::started(
                spoon_core::LifecycleStage::Acquiring,
            )));
            sender.send(spoon_core::BackendEvent::Progress(spoon_core::ProgressEvent::bytes(
                spoon_core::ProgressKind::Download,
                format!("Downloading {}", asset.target_name.as_deref().unwrap_or("asset")),
                0,
                None,
            )));
        }

        spoon_core::copy_or_download_to_file(
            client,
            &asset.url,
            &cache_file,
            "download",
            spoon_core::ProgressKind::Download,
            cancel,
            events,
        )
        .await?;

        paths.push(cache_file);
    }
    Ok(paths)
}

// ── Step 2: Materialize (extract) ──

/// Extract downloaded archives to the version directory.
pub async fn materialize_assets(
    _layout: &ScoopLayout,
    cache_paths: &[PathBuf],
    package_name: &str,
    version: &str,
    source: &ResolvedPackageSource,
    events: Option<&EventSender>,
) -> Result<PathBuf> {
    let staging_root = PathBuf::from(format!("C:/spoon-staging/{package_name}#{version}"));

    if let Some(sender) = events {
        sender.send(spoon_core::BackendEvent::Stage(spoon_core::StageEvent::started(
            spoon_core::LifecycleStage::Materializing,
        )));
    }

    // Extract first archive to staging
    let first = cache_paths.first().ok_or_else(|| ScoopError::Other("no archives to extract".to_string()))?;
    let extracted = extract_archive(first, &staging_root)?;

    // Apply extract_dir substitution if present
    let version_root = if source.extract_dir.is_empty() {
        extracted
    } else {
        let dir_name = &source.extract_dir[0];
        let candidate = staging_root.join(dir_name);
        if candidate.exists() {
            candidate
        } else {
            extracted
        }
    };

    Ok(version_root)
}

fn extract_archive(archive_path: &Path, dest: &Path) -> Result<PathBuf> {
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext.to_ascii_lowercase().as_str() {
        "zip" => {
            spoon_core::extract_zip_archive_sync(archive_path, dest)
                .map_err(|e| ScoopError::Other(format!("zip extraction failed: {e}")))?;
        }
        _ => {
            return Err(ScoopError::Other(format!(
                "unsupported archive type: {ext}"
            )));
        }
    }
    Ok(dest.to_path_buf())
}

// ── Step 3: Persist entries ──

/// Restore persist entries from persist directory into the package directory.
pub async fn restore_persist_entries(
    version_root: &Path,
    persist_root: &Path,
    entries: &[PersistEntry],
) -> Result<()> {
    for entry in entries {
        let from = persist_root.join(&entry.store_name);
        let to = version_root.join(&entry.relative_path);
        if from.exists() {
            if from.is_dir() {
                spoon_core::copy_path_recursive(&from, &to, None).await?;
            } else {
                if let Some(parent) = to.parent() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(|e| ScoopError::fs("create", parent, e))?;
                }
                tokio::fs::copy(&from, &to)
                    .await
                    .map_err(|e| ScoopError::fs("copy", &to, e))?;
            }
        }
    }
    Ok(())
}

/// Sync persist entries from the package directory back to persist directory.
pub async fn sync_persist_entries(
    version_root: &Path,
    persist_root: &Path,
    entries: &[PersistEntry],
) -> Result<()> {
    for entry in entries {
        let from = version_root.join(&entry.relative_path);
        let to = persist_root.join(&entry.store_name);
        if from.exists() {
            if from.is_dir() {
                spoon_core::copy_path_recursive(&from, &to, None).await?;
            } else {
                if let Some(parent) = to.parent() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(|e| ScoopError::fs("create", parent, e))?;
                }
                tokio::fs::copy(&from, &to)
                    .await
                    .map_err(|e| ScoopError::fs("copy", &to, e))?;
            }
        }
    }
    Ok(())
}

// ── Step 4: Surface (shims, shortcuts, PATH) ──

/// Apply shim files and shortcuts for a package.
pub async fn apply_install_surface(
    _layout: &ScoopLayout,
    _package_name: &str,
    version_root: &Path,
    source: &ResolvedPackageSource,
    test_mode: bool,
    events: Option<&EventSender>,
) -> Result<Vec<String>> {
    if let Some(sender) = events {
        sender.send(spoon_core::BackendEvent::Stage(spoon_core::StageEvent::started(
            spoon_core::LifecycleStage::SurfaceApplying,
        )));
    }

    // Create shims directory
    let shims_dir = version_root.join("shims");
    tokio::fs::create_dir_all(&shims_dir)
        .await
        .map_err(|e| ScoopError::fs("create", &shims_dir, e))?;

    let mut written_bins = Vec::new();

    // Write shim files for each bin entry
    for shim_target in &source.bins {
        let shim_path = shims_dir.join(format!("{}.cmd", shim_target.alias));

        let shim_content = format!(
            "@echo off\nset __do_not_use__=1\nexe=\"%~dp0{}\"\nif exist \"%SCOOP_PERSIST_DIR%\\{}\" (\n  set \"SCOOP_PERSIST_DIR=%~dp0..\\..\\persist\\{}\"\n)\n\"%~dp0..\\{}\" %*\n",
            shim_target.relative_path.replace('/', "\\"),
            shim_target.relative_path.replace('/', "\\"),
            shim_target.relative_path.replace('/', "\\"),
            shim_target.relative_path.replace('/', "\\")
        );

        tokio::fs::write(&shim_path, shim_content)
            .await
            .map_err(|e| ScoopError::fs("write", &shim_path, e))?;

        written_bins.push(shim_target.alias.clone());
    }

    // Add shims dir to PATH (if not test mode)
    if !test_mode {
        // This will be handled by the caller via ScoopPorts
    }

    Ok(written_bins)
}

/// Remove shim files and shortcuts for a package.
pub async fn remove_surface(
    layout: &ScoopLayout,
    package_name: &str,
    bins: &[String],
) -> Result<()> {
    let shims_dir = layout.package_current_root(package_name).join("shims");
    for bin in bins {
        let shim_path = shims_dir.join(format!("{}.cmd", bin));
        if shim_path.exists() {
            tokio::fs::remove_file(&shim_path)
                .await
                .map_err(|e| ScoopError::fs("remove", &shim_path, e))?;
        }
    }
    Ok(())
}

// ── Step 5: Integrate (run integration hooks) ──

/// Run integration scripts for a package.
/// Returns applied integrations.
pub async fn run_integrations(
    _package_name: &str,
    _version_root: &Path,
    _persist_root: &Path,
    _pre_scripts: &[String],
    _post_scripts: &[String],
    _events: Option<&EventSender>,
) -> Result<Vec<AppliedIntegration>> {
    // Integration scripts are host-specific. For now, return empty.
    // The binary implements ScoopPorts::apply_integrations for actual hook execution.
    Ok(Vec::new())
}

// ── Full install workflow ──

/// Install a package.
pub async fn install_package<P: ScoopPorts>(
    layout: &ScoopLayout,
    client: &reqwest::Client,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&spoon_core::CancellationToken>,
    _ports: &P,
    events: Option<&EventSender>,
) -> Result<()> {
    let package_name = &plan.package_name;

    // Resolve manifest
    let resolved = crate::runtime::resolve_package_manifest(layout, package_name)
        .await
        .ok_or_else(|| ScoopError::ManifestUnavailable)?;

    // Load manifest and resolve source
    let manifest_value = crate::core::load_manifest_value(&resolved.manifest_path)
        .await
        .map_err(|e| ScoopError::Other(format!("failed to load manifest: {e}")))?;
    let source = crate::core::resolve_package_source(&manifest_value)?;

    let version = &source.version;
    let version_root = layout.package_version_root(package_name, version);

    // Emit stage: Acquire
    if let Some(sender) = events {
        sender.send(spoon_core::BackendEvent::Stage(spoon_core::StageEvent::started(
            spoon_core::LifecycleStage::Acquiring,
        )));
    }

    // Step 1: Download assets to cache
    let cache_paths = acquire_assets(
        layout,
        client,
        package_name,
        version,
        &source.assets,
        proxy,
        cancel,
        events,
    )
    .await?;

    // Step 2: Materialize (extract)
    let extracted_root = materialize_assets(layout, &cache_paths, package_name, version, &source, events).await?;

    // Step 3: Restore persist entries
    if !source.persist.is_empty() {
        if let Some(sender) = events {
            sender.send(spoon_core::BackendEvent::Stage(spoon_core::StageEvent::started(
                spoon_core::LifecycleStage::PersistRestoring,
            )));
        }
        restore_persist_entries(&extracted_root, &layout.package_persist_root(package_name), &source.persist).await?;
    }

    // Step 4: Apply surface (shims, shortcuts)
    apply_install_surface(layout, package_name, &extracted_root, &source, false, events).await?;

    // Step 5: Integrations
    run_integrations(
        package_name,
        &extracted_root,
        &layout.package_persist_root(package_name),
        &source.pre_install,
        &source.post_install,
        events,
    )
    .await?;

    // Update current symlink
    let current = layout.package_current_root(package_name);
    if current.exists() {
        tokio::fs::remove_file(&current)
            .await
            .map_err(|e| ScoopError::fs("remove", &current, e))?;
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(&version_root, &current)
            .map_err(|e| ScoopError::Other(format!("failed to create current symlink: {e}")))?;
    }

    // Build and write installed state
    let state = InstalledPackageState {
        identity: InstalledPackageIdentity {
            package: package_name.clone(),
            version: version.clone(),
            bucket: resolved.bucket.name.clone(),
            architecture: None,
            cache_size_bytes: None,
        },
        command_surface: InstalledPackageCommandSurface {
            bins: source.bins.iter().map(|b| b.alias.clone()).collect(),
            shortcuts: source.shortcuts.clone(),
            env_add_path: source.env_add_path.clone(),
            env_set: source.env_set.clone(),
            persist: source.persist.clone(),
        },
        integrations: Vec::new(),
        uninstall: InstalledPackageUninstall {
            pre_uninstall: source.pre_uninstall.clone(),
            uninstaller_script: source.uninstaller_script.clone(),
            post_uninstall: source.post_uninstall.clone(),
        },
    };

    write_installed_state(layout, &state).await?;

    if let Some(sender) = events {
        sender.send(spoon_core::BackendEvent::Finished(spoon_core::FinishEvent::success(Some(
            format!("{package_name}@{version} installed successfully")
        ))));
    }

    Ok(())
}

// ── Full uninstall workflow ──

/// Uninstall a package.
pub async fn uninstall_package<P: ScoopPorts>(
    layout: &ScoopLayout,
    plan: &ScoopPackagePlan,
    _ports: &P,
    events: Option<&EventSender>,
) -> Result<()> {
    let package_name = &plan.package_name;

    // Read installed state
    let state = read_installed_state(layout, package_name)
        .await?
        .ok_or_else(|| ScoopError::Other(format!("package {} not installed", package_name)))?;

    if let Some(sender) = events {
        sender.send(spoon_core::BackendEvent::Stage(spoon_core::StageEvent::started(
            spoon_core::LifecycleStage::PreUninstallHooks,
        )));
    }

    // Emit stage: Uninstalling
    if let Some(sender) = events {
        sender.send(spoon_core::BackendEvent::Stage(spoon_core::StageEvent::started(
            spoon_core::LifecycleStage::Uninstalling,
        )));
    }

    // Sync persist entries back
    if !state.command_surface.persist.is_empty() {
        if let Some(sender) = events {
            sender.send(spoon_core::BackendEvent::Stage(spoon_core::StageEvent::started(
                spoon_core::LifecycleStage::PersistSyncing,
            )));
        }
        sync_persist_entries(
            &layout.package_current_root(package_name),
            &layout.package_persist_root(package_name),
            &state.command_surface.persist,
        )
        .await?;
    }

    // Remove surface (shims)
    remove_surface(layout, package_name, &state.command_surface.bins).await?;

    // Remove package directory
    let package_root = layout.package_app_root(package_name);
    if package_root.exists() {
        tokio::fs::remove_dir_all(&package_root)
            .await
            .map_err(|e| ScoopError::fs("remove", &package_root, e))?;
    }

    // Remove state
    remove_installed_state(layout, package_name).await?;

    if let Some(sender) = events {
        sender.send(spoon_core::BackendEvent::Finished(spoon_core::FinishEvent::success(Some(
            format!("{package_name} uninstalled successfully")
        ))));
    }

    Ok(())
}

// ── Update workflow ──

/// Update a package (uninstall + install).
pub async fn update_package<P: ScoopPorts>(
    layout: &ScoopLayout,
    client: &reqwest::Client,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&spoon_core::CancellationToken>,
    ports: &P,
    events: Option<&EventSender>,
) -> Result<()> {
    // First uninstall the current version
    uninstall_package(layout, plan, ports, events).await?;

    // Then install the new version (reuse install plan with Install action)
    let mut install_plan = plan.clone();
    install_plan.action = ScoopPackageAction::Install;

    install_package(layout, client, &install_plan, proxy, cancel, ports, events).await?;

    Ok(())
}

// ── Helper functions ──

/// Infer tool root from explicit override, config string, or environment.
pub fn infer_tool_root_with_overrides(
    explicit_root: Option<&Path>,
    config_root: Option<&str>,
) -> Option<PathBuf> {
    explicit_root.map(Path::to_path_buf).or_else(|| {
        let trimmed = config_root?.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(PathBuf::from(trimmed))
        }
    }).or_else(infer_tool_root)
}

/// Infer tool root from environment or config.
pub fn infer_tool_root() -> Option<PathBuf> {
    // Check SPOON_ROOT env var
    if let Ok(root) = std::env::var("SPOON_ROOT") {
        if !root.is_empty() {
            return Some(PathBuf::from(root));
        }
    }
    // Check SCOOP env var
    if let Ok(root) = std::env::var("SCOOP") {
        if !root.is_empty() {
            return Some(PathBuf::from(root));
        }
    }
    // Default to user home via home crate
    home::home_dir().map(|h| h.join("scoop"))
}

/// Execute a package action with streaming events.
pub async fn execute_package_action_streaming<P: ScoopPorts>(
    tool_root: &Path,
    plan: &ScoopPackagePlan,
    proxy: &str,
    cancel: Option<&spoon_core::CancellationToken>,
    ports: &P,
    mut emit: impl FnMut(spoon_core::BackendEvent) + Send + 'static,
) -> Result<ScoopPackageOperationOutcome> {
    let layout = spoon_core::RuntimeLayout::from_root(tool_root).scoop;
    let client = reqwest::Client::new();
    let (sender, mut receiver) = spoon_core::event_bus(16);

    // Spawn a task to forward events to the callback
    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            emit(event);
        }
    });

    let result = match plan.action {
        ScoopPackageAction::Install | ScoopPackageAction::Update => {
            install_package(&layout, &client, plan, proxy, cancel, ports, Some(&sender)).await
        }
        ScoopPackageAction::Uninstall => {
            uninstall_package(&layout, plan, ports, Some(&sender)).await
        }
        ScoopPackageAction::Reapply => {
            // For reapply, just return success
            Ok(())
        }
        ScoopPackageAction::Other => {
            Err(ScoopError::Config(format!("unknown action: {:?}", plan.action)))
        }
    };

    Ok(ScoopPackageOperationOutcome {
        kind: "package_operation",
        action: plan.action.as_str().to_string(),
        package: ScoopActionPackage {
            name: plan.package_name.clone(),
            display_name: plan.display_name.clone(),
        },
        status: if result.is_ok() {
            spoon_core::CommandStatus::Success
        } else {
            spoon_core::CommandStatus::Failed
        },
        title: plan.title(),
        streamed: true,
        output: result.err().map(|e| vec![e.to_string()]).unwrap_or_default(),
        state: ScoopPackageInstallState::default(),
    })
}
