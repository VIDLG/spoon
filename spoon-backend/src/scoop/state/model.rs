use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::scoop::{PersistEntry, ShortcutEntry};

/// Canonical installed-package record -- the single persisted state contract
/// for Scoop packages in spoon-backend.
///
/// Contains only non-derivable facts. Absolute paths, `current` roots, and
/// other layout-derived values are reconstructed from [`crate::layout::RuntimeLayout`]
/// at read time, not stored here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackageState {
    pub package: String,
    pub version: String,
    pub bucket: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_size_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bins: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shortcuts: Vec<ShortcutEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env_add_path: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env_set: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub persist: Vec<PersistEntry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub integrations: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_uninstall: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uninstaller_script: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_uninstall: Vec<String>,
}
