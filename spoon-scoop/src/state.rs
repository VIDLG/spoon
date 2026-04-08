use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Shim target derived from manifest `bin` entries.
#[derive(Debug, Clone)]
pub struct ShimTarget {
    pub relative_path: String,
    pub alias: String,
    pub args: Vec<String>,
}

/// Persist entry from manifest `persist` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistEntry {
    pub relative_path: String,
    pub store_name: String,
}

/// Shortcut entry from manifest `shortcuts` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutEntry {
    pub target_path: String,
    pub name: String,
    pub args: Option<String>,
    pub icon_path: Option<String>,
}

/// Integration applied to a package (e.g. PATH entries, env vars).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedIntegration {
    pub key: String,
    pub value: String,
}

/// Full installed package state, read from filesystem.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstalledPackageState {
    pub identity: InstalledPackageIdentity,
    pub command_surface: InstalledPackageCommandSurface,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub integrations: Vec<AppliedIntegration>,
    pub uninstall: InstalledPackageUninstall,
}

impl InstalledPackageState {
    pub fn package(&self) -> &str {
        &self.identity.package
    }

    pub fn version(&self) -> &str {
        &self.identity.version
    }

    pub fn bucket(&self) -> &str {
        &self.identity.bucket
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstalledPackageIdentity {
    pub package: String,
    pub version: String,
    pub bucket: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstalledPackageCommandSurface {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bins: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shortcuts: Vec<ShortcutEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env_add_path: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env_set: std::collections::BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub persist: Vec<PersistEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstalledPackageUninstall {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_uninstall: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uninstaller_script: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_uninstall: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct InstalledPackageSummary {
    pub name: String,
    pub version: String,
}

/// Supplemental shim specification provided by the host application.
#[derive(Debug, Clone)]
pub struct SupplementalShimSpec {
    pub alias: String,
    pub relative_path: String,
}
