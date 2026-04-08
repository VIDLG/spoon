use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::facts::package_rules::{
    ArchiveKind, ManifestPackageId, PayloadKind, archive_kind, identify_manifest_package_id,
    identify_payload, manifest_package_matches_msvc_target, normalize_msvc_build_version,
    sdk_payload_matches_target,
};
use spoon_core::CoreError;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Payload {
    #[serde(default)]
    pub url: String,
    #[serde(rename = "fileName", default)]
    pub file_name: String,
    #[serde(default)]
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedPayload {
    pub package_id: String,
    pub package_version: String,
    pub payload: Payload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainTarget {
    pub msvc: String,
    pub sdk: String,
}

impl ToolchainTarget {
    pub fn label(&self) -> String {
        format!("{} + {}", self.msvc, self.sdk)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ReleaseManifest {
    #[serde(default)]
    packages: Vec<ManifestPackage>,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestPackage {
    #[serde(default)]
    id: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    payloads: Vec<Payload>,
}

fn latest_manifest_path(manifest_root: &Path) -> std::path::PathBuf {
    let nested = manifest_root.join("vs").join("latest.json");
    if nested.exists() {
        nested
    } else {
        manifest_root.join("latest.json")
    }
}

fn load_release_manifest(manifest_root: &Path) -> Option<ReleaseManifest> {
    let path = latest_manifest_path(manifest_root);
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<ReleaseManifest>(&content).ok()
}

fn numeric_version_key(version: &str) -> Vec<u32> {
    normalize_msvc_build_version(version)
        .split('.')
        .filter_map(|part| part.parse::<u32>().ok())
        .collect()
}

fn better_version(candidate: &str, current: Option<&str>) -> bool {
    match current {
        Some(existing) => numeric_version_key(candidate) > numeric_version_key(existing),
        None => true,
    }
}

fn target_version<'a>(token: &'a str, prefix: &str) -> &'a str {
    token.strip_prefix(prefix).unwrap_or(token)
}

fn payload_archive_kind(payload: &Payload) -> Option<ArchiveKind> {
    archive_kind(&payload.url).or_else(|| archive_kind(&payload.file_name))
}

pub fn latest_toolchain_target_from_cached_manifest(
    manifest_root: &Path,
    host_arch: &str,
    target_arch: &str,
) -> Option<ToolchainTarget> {
    let manifest = load_release_manifest(manifest_root)?;
    let host_arch = host_arch.to_ascii_lowercase();
    let target_arch = target_arch.to_ascii_lowercase();
    let mut latest_msvc: Option<String> = None;
    let mut latest_sdk: Option<String> = None;

    for package in &manifest.packages {
        match identify_manifest_package_id(&package.id) {
            ManifestPackageId::MsvcHostTarget {
                build_version,
                host_arch: package_host_arch,
                target_arch: package_target_arch,
            } if package_host_arch == host_arch && package_target_arch == target_arch => {
                let candidate = format!("msvc-{build_version}");
                if better_version(
                    &build_version,
                    latest_msvc.as_deref().map(|v| target_version(v, "msvc-")),
                ) {
                    latest_msvc = Some(candidate);
                }
            }
            _ => {}
        }

        let has_matching_sdk_payload = package.payloads.iter().any(|payload| {
            identify_payload(&payload.file_name) == PayloadKind::Sdk
                && sdk_payload_matches_target(&payload.file_name, &target_arch)
        });
        if has_matching_sdk_payload
            && better_version(
                &package.version,
                latest_sdk.as_deref().map(|v| target_version(v, "sdk-")),
            )
        {
            latest_sdk = Some(format!("sdk-{}", package.version));
        }
    }

    Some(ToolchainTarget {
        msvc: latest_msvc?,
        sdk: latest_sdk?,
    })
}

pub async fn sync_release_manifest_cache_async(
    manifest_root: &Path,
    _proxy: &str,
) -> Result<Vec<String>, CoreError> {
    let path = latest_manifest_path(manifest_root);
    if path.exists() {
        return Ok(vec![format!(
            "Using cached MSVC release manifest at {}",
            path.display()
        )]);
    }
    Ok(vec![format!(
        "MSVC release manifest cache is missing at {}",
        path.display()
    )])
}

pub fn selected_payloads_from_cached_manifest(
    manifest_root: &Path,
    target: &ToolchainTarget,
    host_arch: &str,
    target_arch: &str,
) -> Option<Vec<SelectedPayload>> {
    let manifest = load_release_manifest(manifest_root)?;
    let host_arch = host_arch.to_ascii_lowercase();
    let target_arch = target_arch.to_ascii_lowercase();
    let target_msvc = target_version(&target.msvc, "msvc-");
    let target_sdk = target_version(&target.sdk, "sdk-");
    let mut selected = Vec::new();

    for package in &manifest.packages {
        let include_msvc_payloads = manifest_package_matches_msvc_target(
            &package.id,
            target_msvc,
            &host_arch,
            &target_arch,
        );
        let include_sdk_payloads = package.version == target_sdk
            && package.payloads.iter().any(|payload| {
                identify_payload(&payload.file_name) == PayloadKind::Sdk
                    && sdk_payload_matches_target(&payload.file_name, &target_arch)
            });

        if !include_msvc_payloads && !include_sdk_payloads {
            continue;
        }

        for payload in &package.payloads {
            let kind = payload_archive_kind(payload);
            let is_cab = matches!(kind, Some(ArchiveKind::Cab));
            let include = if include_msvc_payloads {
                matches!(
                    kind,
                    Some(ArchiveKind::Vsix | ArchiveKind::Msi | ArchiveKind::Zip)
                )
            } else {
                identify_payload(&payload.file_name) == PayloadKind::Sdk
                    && sdk_payload_matches_target(&payload.file_name, &target_arch)
                    && !is_cab
            };
            if !include {
                continue;
            }
            selected.push(SelectedPayload {
                package_id: package.id.clone(),
                package_version: package.version.clone(),
                payload: payload.clone(),
            });
        }
    }

    if selected.is_empty() {
        None
    } else {
        Some(selected)
    }
}

pub fn companion_cab_payloads_for_selected_msi_from_cached_manifest(
    manifest_root: &Path,
    payload: &SelectedPayload,
    external_cab_names: &[String],
) -> Option<Vec<SelectedPayload>> {
    if external_cab_names.is_empty() {
        return Some(Vec::new());
    }
    let manifest = load_release_manifest(manifest_root)?;
    let wanted = external_cab_names
        .iter()
        .map(|name| {
            std::path::Path::new(name)
                .file_name()
                .and_then(|part| part.to_str())
                .unwrap_or(name.as_str())
                .to_ascii_lowercase()
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut matches = Vec::new();

    for package in &manifest.packages {
        if package.version != payload.package_version {
            continue;
        }
        for candidate in &package.payloads {
            if !matches!(payload_archive_kind(candidate), Some(ArchiveKind::Cab)) {
                continue;
            }
            let Some(file_name) = std::path::Path::new(&candidate.file_name)
                .file_name()
                .and_then(|part| part.to_str())
            else {
                continue;
            };
            if !wanted.contains(&file_name.to_ascii_lowercase()) {
                continue;
            }
            matches.push(SelectedPayload {
                package_id: package.id.clone(),
                package_version: package.version.clone(),
                payload: candidate.clone(),
            });
        }
    }

    Some(matches)
}
