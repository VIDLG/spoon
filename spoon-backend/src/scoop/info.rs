use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::CommandStatus;
use crate::control_plane::sqlite::db_path_for_layout;
use crate::layout::RuntimeLayout;

use super::manifest;
use super::projection::{
    collect_bin_items, collect_shortcut_items, collect_urls_vec, directory_size,
    integration_display_key, json_value_or_display, license_display_value, manifest_value,
    manifest_value_owned, policy_config_kind, resolve_env_map, resolve_env_paths, string_items,
    string_map_items,
};
use super::state::read_installed_state;
use super::state::InstalledPackageState;

#[derive(Debug, Serialize)]
pub struct ScoopPackageActionOutcome {
    pub kind: &'static str,
    pub action: String,
    pub package: ScoopActionPackage,
    pub success: bool,
    pub title: String,
    pub streamed: bool,
    pub output: Vec<String>,
    pub state: ScoopPackageInstallState,
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageOperationOutcome {
    pub kind: &'static str,
    pub action: String,
    pub package: ScoopActionPackage,
    pub status: CommandStatus,
    pub title: String,
    pub streamed: bool,
    pub output: Vec<String>,
    pub state: ScoopPackageInstallState,
}

impl ScoopPackageOperationOutcome {
    pub const fn is_success(&self) -> bool {
        self.status.is_success()
    }
}

#[derive(Debug, Serialize)]
pub struct ScoopActionPackage {
    pub name: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageInstallState {
    pub installed: bool,
    pub installed_version: Option<String>,
    pub current: Option<String>,
}

pub async fn package_operation_outcome(
    tool_root: &Path,
    action: &str,
    package_name: &str,
    display_name: &str,
    status: CommandStatus,
    title: impl Into<String>,
    output: Vec<String>,
    streamed: bool,
) -> ScoopPackageOperationOutcome {
    let layout = RuntimeLayout::from_root(tool_root);
    let prefix = layout.scoop.package_current_root(package_name);
    let installed_version = read_installed_state(&layout, package_name)
        .await
        .map(|state| state.version.trim().to_string());
    let installed = installed_version.is_some() && prefix.exists();
    ScoopPackageOperationOutcome {
        kind: "scoop_package_operation",
        action: action.to_string(),
        package: ScoopActionPackage {
            name: package_name.to_string(),
            display_name: display_name.to_string(),
        },
        status,
        title: title.into(),
        streamed,
        output,
        state: ScoopPackageInstallState {
            installed,
            installed_version,
            current: installed.then(|| prefix.display().to_string()),
        },
    }
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageError {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ScoopPolicyAppliedValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageMetadata {
    pub name: String,
    pub bucket: String,
    pub latest_version: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub manifest: String,
    pub license: Option<String>,
    pub depends: Option<serde_json::Value>,
    pub suggest: Option<serde_json::Value>,
    pub extract_dir: Option<serde_json::Value>,
    pub extract_to: Option<serde_json::Value>,
    pub notes: Vec<String>,
    pub download_urls: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageInstall {
    pub installed: bool,
    pub installed_version: Option<String>,
    pub current: String,
    pub installed_size_bytes: Option<u64>,
    pub cache_size_bytes: Option<u64>,
    pub bins: Vec<String>,
    pub state: Option<String>,
    pub persist_root: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScoopCommandIntegration {
    pub shims: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ScoopEnvironmentIntegration {
    pub add_path: Vec<String>,
    pub set: Vec<String>,
    pub persist: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ScoopSystemIntegration {
    pub shortcuts: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ScoopPolicyIntegration<D> {
    pub desired: Vec<D>,
    pub applied_values: Vec<ScoopPolicyAppliedValue>,
    pub config_files: Vec<String>,
    pub config_directories: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageIntegration<D> {
    pub commands: ScoopCommandIntegration,
    pub environment: ScoopEnvironmentIntegration,
    pub system: ScoopSystemIntegration,
    pub policy: ScoopPolicyIntegration<D>,
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageDetails<D> {
    pub kind: &'static str,
    pub success: bool,
    pub package: ScoopPackageMetadata,
    pub install: ScoopPackageInstall,
    pub integration: ScoopPackageIntegration<D>,
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageDetailsError {
    pub kind: &'static str,
    pub success: bool,
    pub package: String,
    pub error: ScoopPackageError,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ScoopPackageDetailsOutcome<D> {
    Details(ScoopPackageDetails<D>),
    Error(ScoopPackageDetailsError),
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageManifestOutcome {
    pub kind: &'static str,
    pub package: String,
    pub status: CommandStatus,
    pub title: String,
    pub manifest_path: Option<String>,
    pub content: Option<String>,
    pub error: Option<ScoopPackageError>,
    pub streamed: bool,
}

impl ScoopPackageManifestOutcome {
    pub const fn is_success(&self) -> bool {
        self.status.is_success()
    }
}

pub async fn package_manifest(tool_root: &Path, package_name: &str) -> ScoopPackageManifestOutcome {
    let Some(resolved) = manifest::resolve_package_manifest(package_name, tool_root).await else {
        return ScoopPackageManifestOutcome {
            kind: "scoop_package_manifest",
            package: package_name.to_string(),
            title: format!("cat Scoop package {package_name}"),
            status: CommandStatus::Failed,
            manifest_path: None,
            content: None,
            error: Some(ScoopPackageError {
                message: format!(
                    "Scoop package '{package_name}' was not found in registered buckets."
                ),
            }),
            streamed: false,
        };
    };
    match fs::read_to_string(&resolved.manifest_path) {
        Ok(content) => ScoopPackageManifestOutcome {
            kind: "scoop_package_manifest",
            package: package_name.to_string(),
            title: format!("cat Scoop package {package_name}"),
            status: CommandStatus::Success,
            manifest_path: Some(resolved.manifest_path.display().to_string()),
            content: Some(content),
            error: None,
            streamed: false,
        },
        Err(err) => ScoopPackageManifestOutcome {
            kind: "scoop_package_manifest",
            package: package_name.to_string(),
            title: format!("cat Scoop package {package_name}"),
            status: CommandStatus::Failed,
            manifest_path: Some(resolved.manifest_path.display().to_string()),
            content: None,
            error: Some(ScoopPackageError {
                message: format!(
                    "Failed to read manifest {}: {err}",
                    resolved.manifest_path.display()
                ),
            }),
            streamed: false,
        },
    }
}

pub async fn package_info<D, F>(
    tool_root: &Path,
    package_name: &str,
    desired_policy: Vec<D>,
    desired_key: F,
) -> ScoopPackageDetailsOutcome<D>
where
    D: Serialize,
    F: Fn(&D) -> &str,
{
    let layout = RuntimeLayout::from_root(tool_root);
    let resolved = manifest::resolve_package_manifest(package_name, tool_root).await;
    let installed_state: Option<InstalledPackageState> =
        read_installed_state(&layout, package_name).await;
    let installed_version = installed_state
        .as_ref()
        .map(|state| state.version.trim().to_string());
    let current_root = layout.scoop.package_current_root(package_name);

    let Some(resolved) = resolved else {
        return ScoopPackageDetailsOutcome::Error(ScoopPackageDetailsError {
            kind: "package_info",
            success: false,
            package: package_name.to_string(),
            error: ScoopPackageError {
                message: format!(
                    "Scoop package '{package_name}' was not found in registered buckets."
                ),
            },
        });
    };

    let manifest = manifest::load_manifest(&resolved.manifest_path).await;
    let latest_version = manifest
        .as_ref()
        .and_then(|doc| doc.version.as_ref())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let description = manifest
        .as_ref()
        .and_then(|doc| doc.description.as_ref())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let homepage = manifest
        .as_ref()
        .and_then(|doc| doc.homepage.as_ref())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let installed = installed_version.is_some() && current_root.exists();
    let installed_size = if installed {
        Some(directory_size(&current_root))
    } else {
        None
    };
    let cache_size = installed_state.as_ref().and_then(|state| state.cache_size_bytes);
    let state_path = db_path_for_layout(&layout);

    let manifest_license = manifest
        .as_ref()
        .and_then(|doc| manifest_value_owned(doc, "license"))
        .as_ref()
        .and_then(license_display_value);
    let manifest_depends = manifest
        .as_ref()
        .and_then(|doc| manifest_value_owned(doc, "depends"))
        .as_ref()
        .and_then(json_value_or_display);
    let manifest_suggest = manifest
        .as_ref()
        .and_then(|doc| manifest_value_owned(doc, "suggest"))
        .as_ref()
        .and_then(json_value_or_display);
    let manifest_extract_dir = manifest
        .as_ref()
        .and_then(|doc| manifest_value_owned(doc, "extract_dir"))
        .as_ref()
        .and_then(json_value_or_display);
    let manifest_extract_to = manifest
        .as_ref()
        .and_then(|doc| manifest_value_owned(doc, "extract_to"))
        .as_ref()
        .and_then(json_value_or_display);
    let manifest_notes = manifest
        .as_ref()
        .and_then(|doc| manifest_value(doc, "notes"))
        .map(|value| string_items(Some(value)))
        .unwrap_or_default();
    let manifest_download_urls = manifest
        .as_ref()
        .and_then(|doc| manifest_value_owned(doc, "url"))
        .as_ref()
        .map(|value| collect_urls_vec(value, true))
        .unwrap_or_default();
    let manifest_bins = manifest
        .as_ref()
        .and_then(|doc| manifest_value_owned(doc, "bin"))
        .as_ref()
        .map(collect_bin_items)
        .unwrap_or_default();
    let manifest_shortcuts = manifest
        .as_ref()
        .and_then(|doc| manifest_value_owned(doc, "shortcuts"))
        .as_ref()
        .map(collect_shortcut_items)
        .unwrap_or_default();
    let manifest_persist = manifest
        .as_ref()
        .and_then(|doc| manifest_value_owned(doc, "persist"))
        .as_ref()
        .and_then(json_value_or_display);
    let manifest_env_add_paths = manifest
        .as_ref()
        .map(|doc| {
            let value = manifest_value_owned(doc, "env_add_path");
            string_items(value)
        })
        .unwrap_or_default();
    let manifest_env_set_items = manifest
        .as_ref()
        .map(|doc| string_map_items(manifest_value(doc, "env_set")))
        .unwrap_or_default();

    // Read runtime fields from typed canonical state
    let runtime_bins = installed_state
        .as_ref()
        .map(|state| state.bins.clone())
        .unwrap_or_default();
    let runtime_shortcuts: Vec<String> = installed_state
        .as_ref()
        .map(|state| {
            state
                .shortcuts
                .iter()
                .filter_map(|sc| {
                    let name = sc.name.trim();
                    let target = sc.target_path.trim();
                    let args = sc.args.as_deref().map(str::trim).filter(|v| !v.is_empty());
                    match (name, target, args) {
                        ("", _, _) | (_, "", _) => None,
                        (n, t, Some(a)) => Some(format!("{n} -> {t} {a}")),
                        (n, t, None) => Some(format!("{n} -> {t}")),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let runtime_persist: Option<serde_json::Value> = installed_state
        .as_ref()
        .filter(|state| !state.persist.is_empty())
        .and_then(|state| {
            let paths: Vec<&str> = state
                .persist
                .iter()
                .map(|entry| entry.relative_path.as_str())
                .collect();
            serde_json::to_value(&paths).ok()
        })
        .and_then(|value| json_value_or_display(&value));
    let runtime_integrations: Vec<(String, String)> = installed_state
        .as_ref()
        .map(|state| {
            state
                .integrations
                .iter()
                .filter_map(|(key, value)| {
                    let trimmed = value.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some((
                            integration_display_key(package_name, key),
                            trimmed.to_string(),
                        ))
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let runtime_env_add_paths = installed_state
        .as_ref()
        .map(|state| state.env_add_path.clone())
        .unwrap_or_default();
    let runtime_env_set_items: Vec<(String, String)> = installed_state
        .as_ref()
        .map(|state| {
            state
                .env_set
                .iter()
                .filter_map(|(k, v)| {
                    let v_trimmed = v.trim();
                    if v_trimmed.is_empty() {
                        None
                    } else {
                        Some((k.clone(), v_trimmed.to_string()))
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let persist_root = layout.scoop.package_persist_root(package_name);
    let resolved_env_add_paths = if runtime_env_add_paths.is_empty() {
        resolve_env_paths(manifest_env_add_paths, &current_root, &persist_root)
    } else {
        resolve_env_paths(runtime_env_add_paths, &current_root, &persist_root)
    };
    let resolved_env_set = if runtime_env_set_items.is_empty() {
        resolve_env_map(manifest_env_set_items, &current_root, &persist_root)
    } else {
        resolve_env_map(runtime_env_set_items, &current_root, &persist_root)
    };

    let desired_keys = desired_policy
        .iter()
        .map(|entry| desired_key(entry).to_string())
        .collect::<BTreeSet<_>>();
    let mut applied_values = Vec::new();
    let mut config_files = Vec::new();
    let mut config_dirs = Vec::new();
    for (key, value) in &runtime_integrations {
        if desired_keys.contains(key) {
            continue;
        }
        match policy_config_kind(key) {
            Some("config files") => config_files.push(value.clone()),
            Some("config directories") => config_dirs.push(value.clone()),
            _ => applied_values.push(ScoopPolicyAppliedValue {
                key: key.clone(),
                value: value.clone(),
            }),
        }
    }
    let effective_bins = if runtime_bins.is_empty() {
        manifest_bins.clone()
    } else {
        runtime_bins.clone()
    };

    ScoopPackageDetailsOutcome::Details(ScoopPackageDetails {
        kind: "package_info",
        success: true,
        package: ScoopPackageMetadata {
            name: package_name.to_string(),
            bucket: resolved.bucket.name.clone(),
            latest_version,
            description,
            homepage,
            manifest: resolved.manifest_path.display().to_string(),
            license: manifest_license,
            depends: manifest_depends,
            suggest: manifest_suggest,
            extract_dir: manifest_extract_dir,
            extract_to: manifest_extract_to,
            notes: manifest_notes,
            download_urls: manifest_download_urls,
        },
        install: ScoopPackageInstall {
            installed,
            installed_version,
            current: current_root.display().to_string(),
            installed_size_bytes: installed_size,
            cache_size_bytes: cache_size.filter(|value| *value > 0),
            bins: effective_bins,
            state: state_path
                .exists()
                .then(|| state_path.display().to_string()),
            persist_root: (installed
                && runtime_persist
                    .clone()
                    .or(manifest_persist.clone())
                    .is_some())
            .then(|| persist_root.display().to_string()),
        },
        integration: ScoopPackageIntegration {
            commands: ScoopCommandIntegration {
                shims: if runtime_bins.is_empty() {
                    None
                } else {
                    Some(runtime_bins)
                },
            },
            environment: ScoopEnvironmentIntegration {
                add_path: resolved_env_add_paths,
                set: resolved_env_set,
                persist: runtime_persist.or(manifest_persist),
            },
            system: ScoopSystemIntegration {
                shortcuts: if runtime_shortcuts.is_empty() {
                    manifest_shortcuts
                } else {
                    runtime_shortcuts
                },
            },
            policy: ScoopPolicyIntegration {
                desired: desired_policy,
                applied_values,
                config_files,
                config_directories: config_dirs,
            },
        },
    })
}
