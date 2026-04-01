use std::fs;
use std::path::Path;

use serde::Serialize;
use walkdir::WalkDir;

use crate::BackendContext;
use crate::layout::RuntimeLayout;

use super::detect::detect_runtimes;
use super::official;
use super::paths;
use super::rules::read_installed_toolchain_target;
use super::{
    MsvcRuntimeKind, MsvcValidationStatus, manifest, manifest_dir, native_host_arch,
    read_canonical_state,
};

pub fn user_facing_toolchain_label(raw: &str) -> String {
    raw.replace("msvc-", "").replace("sdk-", "")
}

pub fn latest_toolchain_version_label(tool_root: Option<&Path>) -> Option<String> {
    let root = tool_root.map(Path::to_path_buf)?;
    let manifest_root = manifest_dir(&root);
    let target_arch = super::MsvcRequest::for_tool_root(&root).normalized_target_arch();
    manifest::latest_toolchain_target_from_cached_manifest(
        &manifest_root,
        native_host_arch(),
        &target_arch,
    )
    .map(|target| user_facing_toolchain_label(&target.label()))
}

pub fn latest_toolchain_version_label_with_context<P>(
    context: &BackendContext<P>,
) -> Option<String> {
    let request = super::MsvcRequest::from_context(context);
    let manifest_root = manifest_dir(&request.root);
    let target_arch = request.normalized_target_arch();
    manifest::latest_toolchain_target_from_cached_manifest(
        &manifest_root,
        native_host_arch(),
        &target_arch,
    )
    .map(|target| user_facing_toolchain_label(&target.label()))
}

pub fn installed_toolchain_version_label(tool_root: &Path) -> Option<String> {
    let target = read_installed_toolchain_target(&paths::msvc_root(tool_root))?;
    Some(user_facing_toolchain_label(&target.label()))
}

#[derive(Debug, Serialize)]
pub struct MsvcCommandIntegration {
    pub wrappers: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MsvcEnvironmentIntegration {
    pub shims_root: String,
    pub user_path_entry: String,
}

#[derive(Debug, Serialize)]
pub struct ManagedMsvcIntegration {
    pub commands: MsvcCommandIntegration,
    pub environment: MsvcEnvironmentIntegration,
}

#[derive(Debug, Serialize)]
pub struct OfficialMsvcSystemIntegration {
    pub vswhere_discovery: String,
    pub shared_windows_sdk_root: String,
    pub registration: String,
}

#[derive(Debug, Serialize)]
pub struct OfficialMsvcIntegration {
    pub system: OfficialMsvcSystemIntegration,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum MsvcIntegration {
    ActiveManaged(ManagedMsvcIntegration),
    ActiveOfficial(OfficialMsvcIntegration),
    None { none: bool },
}

#[derive(Debug, Serialize)]
pub struct ManagedMsvcRuntimeStatus {
    pub status: String,
    pub installed_version: Option<String>,
    pub root: String,
    pub toolchain: String,
    pub state: String,
    pub cache: String,
    pub runtime_state_present: bool,
    pub archives: usize,
    pub staged_msi_payloads: usize,
    pub extracted_msi_payloads: usize,
    pub install_image_files: usize,
    pub integration: MsvcIntegration,
}

#[derive(Debug, Serialize)]
pub struct OfficialMsvcRuntimeStatus {
    pub status: String,
    pub installed_version: Option<String>,
    pub root: String,
    pub state: String,
    pub cache: String,
    pub runtime_state_present: bool,
    pub integration: MsvcIntegration,
}

#[derive(Debug, Serialize)]
pub struct MsvcStatus {
    pub kind: &'static str,
    pub success: bool,
    pub authoritative_runtime: Option<MsvcRuntimeKind>,
    pub validation_status: Option<MsvcValidationStatus>,
    pub validation_message: Option<String>,
    pub managed: ManagedMsvcRuntimeStatus,
    pub official: OfficialMsvcRuntimeStatus,
}

pub async fn status(tool_root: &Path) -> MsvcStatus {
    let request = super::MsvcRequest::for_tool_root(tool_root);
    status_with_request(&request).await
}

pub async fn status_with_context<P>(context: &BackendContext<P>) -> MsvcStatus {
    let request = super::MsvcRequest::from_context(context);
    status_with_request(&request).await
}

async fn status_with_request(request: &super::MsvcRequest) -> MsvcStatus {
    let tool_root = request.root.as_path();
    let detected = detect_runtimes(tool_root);
    let layout = RuntimeLayout::from_root(tool_root);
    let canonical = read_canonical_state(&layout).await;
    let managed_root = paths::msvc_root(tool_root);
    let official_root = paths::official_msvc_root(tool_root);
    let managed_wrapper_names = managed_wrapper_names(tool_root);
    let canonical_managed_version = canonical
        .as_ref()
        .filter(|state| state.runtime_kind == MsvcRuntimeKind::Managed)
        .and_then(|state| match (&state.version, &state.sdk_version) {
            (Some(version), Some(sdk)) => Some(format!("{version} + {sdk}")),
            (Some(version), None) => Some(version.clone()),
            (None, Some(sdk)) => Some(sdk.clone()),
            (None, None) => None,
        });
    let canonical_official_version = canonical
        .as_ref()
        .filter(|state| state.runtime_kind == MsvcRuntimeKind::Official)
        .and_then(|state| match (&state.version, &state.sdk_version) {
            (Some(version), Some(sdk)) => Some(format!("{version} + {sdk}")),
            (Some(version), None) => Some(version.clone()),
            (None, Some(sdk)) => Some(sdk.clone()),
            (None, None) => None,
        });
    let managed_installed_version = canonical_managed_version
        .clone()
        .or_else(|| detected.managed.installed_version.clone());
    let official_installed_version = canonical_official_version
        .clone()
        .or_else(|| detected.official.installed_version.clone());
    MsvcStatus {
        kind: "msvc_status",
        success: true,
        authoritative_runtime: canonical.as_ref().map(|state| state.runtime_kind),
        validation_status: canonical
            .as_ref()
            .and_then(|state| state.validation_status.clone()),
        validation_message: canonical
            .as_ref()
            .and_then(|state| state.validation_message.clone()),
        managed: ManagedMsvcRuntimeStatus {
            status: managed_installed_version
                .as_ref()
                .map(|version| format!("installed ({version})"))
                .unwrap_or_else(|| "not installed".to_string()),
            installed_version: managed_installed_version,
            root: managed_root.display().to_string(),
            toolchain: paths::msvc_toolchain_root(tool_root).display().to_string(),
            state: paths::msvc_state_root(tool_root).display().to_string(),
            cache: paths::msvc_cache_root(tool_root).display().to_string(),
            runtime_state_present: detected.managed.runtime_state_present,
            archives: cached_payload_archive_count(Some(tool_root)).unwrap_or(0),
            staged_msi_payloads: staged_msi_payload_count(Some(tool_root)).unwrap_or(0),
            extracted_msi_payloads: extracted_msi_payload_count(Some(tool_root)).unwrap_or(0),
            install_image_files: install_image_file_count(Some(tool_root)).unwrap_or(0),
            integration: if detected.managed.installed_version.is_some() {
                MsvcIntegration::ActiveManaged(ManagedMsvcIntegration {
                    commands: MsvcCommandIntegration {
                        wrappers: managed_wrapper_names
                            .into_iter()
                            .map(str::to_string)
                            .collect(),
                    },
                    environment: MsvcEnvironmentIntegration {
                        shims_root: paths::shims_root(tool_root).display().to_string(),
                        user_path_entry: "<root>/shims".to_string(),
                    },
                })
            } else {
                MsvcIntegration::None { none: true }
            },
        },
        official: OfficialMsvcRuntimeStatus {
            status: official_installed_version
                .as_ref()
                .map(|version| format!("installed ({version})"))
                .unwrap_or_else(|| "not installed".to_string()),
            installed_version: official_installed_version,
            root: official_root.display().to_string(),
            state: paths::official_msvc_state_root(tool_root)
                .display()
                .to_string(),
            cache: paths::official_msvc_cache_root(tool_root)
                .display()
                .to_string(),
            runtime_state_present: detected.official.runtime_state_present,
            integration: if detected.official.installed_version.is_some() {
                MsvcIntegration::ActiveOfficial(OfficialMsvcIntegration {
                    system: OfficialMsvcSystemIntegration {
                        vswhere_discovery: official::vswhere_path().display().to_string(),
                        shared_windows_sdk_root: official::windows_kits_root()
                            .display()
                            .to_string(),
                        registration: "Visual Studio Installer + Windows SDK discovery".to_string(),
                    },
                })
            } else {
                MsvcIntegration::None { none: true }
            },
        },
    }
}

pub(crate) fn cached_payload_archive_count(tool_root: Option<&Path>) -> Option<usize> {
    let root = tool_root.map(Path::to_path_buf)?;
    let dir = super::payload_cache_dir(&root);
    let entries = fs::read_dir(dir).ok()?;
    Some(
        entries
            .flatten()
            .filter(|entry| entry.path().is_file())
            .count(),
    )
}

pub(crate) fn staged_msi_payload_count(tool_root: Option<&Path>) -> Option<usize> {
    let root = tool_root.map(Path::to_path_buf)?;
    let dir = super::msi_staging_cache_dir(&root);
    let entries = fs::read_dir(dir).ok()?;
    Some(
        entries
            .flatten()
            .filter(|entry| entry.path().join(".complete").exists())
            .count(),
    )
}

pub(crate) fn extracted_msi_payload_count(tool_root: Option<&Path>) -> Option<usize> {
    let root = tool_root.map(Path::to_path_buf)?;
    let dir = super::extracted_msi_cache_dir(&root);
    let entries = fs::read_dir(dir).ok()?;
    Some(
        entries
            .flatten()
            .filter(|entry| entry.path().join(".complete").exists())
            .count(),
    )
}

pub(crate) fn install_image_file_count(tool_root: Option<&Path>) -> Option<usize> {
    let root = tool_root.map(Path::to_path_buf)?;
    let dir = super::install_image_cache_dir(&root);
    Some(count_files_recursively(&dir))
}

pub fn count_files_recursively(root: &Path) -> usize {
    WalkDir::new(root)
        .into_iter()
        .flatten()
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.file_name().to_string_lossy() != ".complete")
        .count()
}

fn managed_wrapper_names(tool_root: &Path) -> Vec<&'static str> {
    let shims_root = paths::shims_root(tool_root);
    [
        ("spoon-cl", shims_root.join("spoon-cl.cmd")),
        ("spoon-link", shims_root.join("spoon-link.cmd")),
        ("spoon-lib", shims_root.join("spoon-lib.cmd")),
        ("spoon-rc", shims_root.join("spoon-rc.cmd")),
        ("spoon-mt", shims_root.join("spoon-mt.cmd")),
        ("spoon-nmake", shims_root.join("spoon-nmake.cmd")),
        ("spoon-dumpbin", shims_root.join("spoon-dumpbin.cmd")),
    ]
    .into_iter()
    .filter_map(|(name, path): (&str, std::path::PathBuf)| path.exists().then_some(name))
    .collect()
}
