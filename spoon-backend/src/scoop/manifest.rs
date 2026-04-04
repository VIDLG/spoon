use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{BackendError, Result};

use super::buckets::resolve_manifest;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ScoopManifest {
    pub version: Option<String>,
    pub homepage: Option<String>,
    pub description: Option<String>,
    pub license: Option<License>,
    pub notes: Option<Notes>,
    pub depends: Option<StringOrArray>,
    pub suggest: Option<SuggestMap>,
    pub architecture: Option<ArchitectureMap>,
    pub url: Option<StringOrArray>,
    pub hash: Option<StringOrArray>,
    pub bin: Option<BinEntries>,
    pub shortcuts: Option<Vec<Shortcut>>,
    pub env_add_path: Option<StringOrArray>,
    pub env_set: Option<std::collections::HashMap<String, String>>,
    pub persist: Option<StringOrArray>,
    pub extract_dir: Option<String>,
    pub extract_to: Option<String>,
    pub checkver: Option<serde_json::Value>,
    pub autoupdate: Option<serde_json::Value>,
    pub installer: Option<Installer>,
    pub uninstaller: Option<Installer>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum License {
    Simple(String),
    Detailed {
        identifier: String,
        url: Option<String>,
    },
}

impl License {
    pub fn identifier(&self) -> &str {
        match self {
            License::Simple(s) => s,
            License::Detailed { identifier, .. } => identifier,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Notes {
    Single(String),
    Multiple(Vec<String>),
}

impl Notes {
    pub fn lines(&self) -> Vec<&str> {
        match self {
            Notes::Single(s) => vec![s.as_str()],
            Notes::Multiple(v) => v.iter().map(|s| s.as_str()).collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StringOrArray {
    Single(String),
    Multiple(Vec<String>),
}

impl StringOrArray {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            StringOrArray::Single(s) => vec![s.clone()],
            StringOrArray::Multiple(v) => v.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SuggestMap(pub std::collections::HashMap<String, StringOrArray>);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArchitectureMap {
    #[serde(rename = "64bit")]
    pub x64: Option<ArchConfig>,
    #[serde(rename = "32bit")]
    pub x86: Option<ArchConfig>,
    #[serde(rename = "arm64")]
    pub arm64: Option<ArchConfig>,
}

impl ArchitectureMap {
    pub fn for_arch(&self, arch: &str) -> Option<&ArchConfig> {
        match arch {
            "x64" | "amd64" | "64bit" => self.x64.as_ref(),
            "x86" | "32bit" => self.x86.as_ref(),
            "arm64" => self.arm64.as_ref(),
            _ => self.x64.as_ref().or(self.x86.as_ref()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArchConfig {
    pub url: Option<StringOrArray>,
    pub hash: Option<StringOrArray>,
    pub bin: Option<BinEntries>,
    pub extract_dir: Option<String>,
    pub env_add_path: Option<StringOrArray>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BinEntry {
    Path(String),
    WithAlias(Vec<String>),
}

impl BinEntry {
    pub fn path(&self) -> &str {
        match self {
            BinEntry::Path(p) => p,
            BinEntry::WithAlias(v) => v.first().map(|s| s.as_str()).unwrap_or(""),
        }
    }

    pub fn alias(&self) -> Option<&str> {
        match self {
            BinEntry::Path(_) => None,
            BinEntry::WithAlias(v) => v.get(1).map(|s| s.as_str()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BinEntries {
    Single(BinEntry),
    Multiple(Vec<BinEntry>),
}

impl BinEntries {
    pub fn to_vec(&self) -> Vec<&BinEntry> {
        match self {
            BinEntries::Single(b) => vec![b],
            BinEntries::Multiple(v) => v.iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Shortcut {
    Simple(String),
    Detailed {
        name: Option<String>,
        #[serde(rename = "target")]
        target_path: String,
        args: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Installer {
    pub file: Option<String>,
    pub script: Option<StringOrArray>,
    pub args: Option<StringOrArray>,
    pub keep: Option<bool>,
}

pub fn parse_manifest(json: &str) -> std::result::Result<ScoopManifest, serde_json::Error> {
    serde_json::from_str(json)
}

pub async fn load_manifest(path: &Path) -> Option<ScoopManifest> {
    let content = tokio::fs::read_to_string(path).await.ok()?;
    parse_manifest(&content).ok()
}

pub async fn load_manifest_value(path: &Path) -> Result<serde_json::Value> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|err| BackendError::fs("read", path, err))?;
    serde_json::from_str(&content)
        .map_err(BackendError::from)
        .map_err(|err| err.context(format!("invalid manifest {}", path.display())))
}

pub fn load_manifest_sync(path: &Path) -> Option<ScoopManifest> {
    let content = std::fs::read_to_string(path).ok()?;
    parse_manifest(&content).ok()
}

pub struct ScoopStringField;

impl ScoopStringField {
    pub fn as_json(value: &StringOrArray) -> serde_json::Value {
        match value {
            StringOrArray::Single(s) => serde_json::Value::String(s.clone()),
            StringOrArray::Multiple(v) => serde_json::Value::Array(
                v.iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            ),
        }
    }
}

pub struct ScoopBinField;

impl ScoopBinField {
    pub fn as_json(value: &BinEntries) -> serde_json::Value {
        match value {
            BinEntries::Single(b) => serde_json::Value::String(b.path().to_string()),
            BinEntries::Multiple(v) => serde_json::Value::Array(
                v.iter()
                    .map(|b| serde_json::Value::String(b.path().to_string()))
                    .collect(),
            ),
        }
    }
}

pub async fn resolve_package_manifest(
    package_name: &str,
    tool_root: &Path,
) -> Option<super::buckets::ResolvedBucket> {
    resolve_manifest(tool_root, package_name).await
}

pub async fn load_package_manifest(package_name: &str, tool_root: &Path) -> Option<ScoopManifest> {
    let resolved = resolve_manifest(tool_root, package_name).await?;
    load_manifest(&resolved.manifest_path).await
}

pub fn load_package_manifest_sync(package_name: &str, tool_root: &Path) -> Option<ScoopManifest> {
    let resolved =
        tokio::runtime::Handle::current().block_on(resolve_manifest(tool_root, package_name))?;
    load_manifest_sync(&resolved.manifest_path)
}

pub fn latest_version(tool_root: &Path, package_name: &str) -> Option<String> {
    let resolved =
        tokio::runtime::Handle::current().block_on(resolve_manifest(tool_root, package_name))?;
    let manifest = load_manifest_sync(&resolved.manifest_path)?;
    manifest.version
}

pub async fn search_manifests_async(
    _query: &str,
    _tool_root: &Path,
) -> Vec<(String, String, ScoopManifest)> {
    Vec::new()
}

pub async fn latest_version_async(package_name: &str, tool_root: &Path) -> Option<String> {
    latest_version(tool_root, package_name)
}
