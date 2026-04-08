use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::state::{PersistEntry, ShimTarget, ShortcutEntry};
use crate::{ScoopError, error::Result};

/// Fully resolved package source from a manifest.
#[derive(Debug, Clone)]
pub struct ResolvedPackageSource {
    pub version: String,
    pub assets: Vec<PackageAsset>,
    pub depends: Vec<String>,
    pub extract_dir: Vec<String>,
    pub extract_to: Vec<String>,
    pub installer_script: Vec<String>,
    pub bins: Vec<ShimTarget>,
    pub shortcuts: Vec<ShortcutEntry>,
    pub env_add_path: Vec<String>,
    pub env_set: BTreeMap<String, String>,
    pub persist: Vec<PersistEntry>,
    pub pre_install: Vec<String>,
    pub post_install: Vec<String>,
    pub pre_uninstall: Vec<String>,
    pub post_uninstall: Vec<String>,
    pub uninstaller_script: Vec<String>,
}

/// A single downloadable asset from a package manifest.
#[derive(Debug, Clone)]
pub struct PackageAsset {
    pub url: String,
    pub hash: String,
    pub target_name: Option<String>,
}

// ── Internal raw types for serde deserialization ──

#[derive(Debug, Clone, Default, Deserialize)]
struct RawManifest {
    #[serde(flatten)]
    base: RawPackageFields,
    #[serde(default)]
    architecture: BTreeMap<String, RawPackageFields>,
}

impl RawManifest {
    fn resolve(&self, arch_key: &str) -> RawPackageFields {
        let mut resolved = self.base.clone();
        if let Some(arch) = self.architecture.get(arch_key) {
            resolved.overlay(arch);
        }
        resolved
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
struct RawPackageFields {
    version: Option<String>,
    url: Option<RawStringList>,
    hash: Option<RawStringList>,
    depends: Option<RawStringList>,
    extract_dir: Option<RawStringList>,
    extract_to: Option<RawStringList>,
    installer: Option<RawScriptOwner>,
    bin: Option<RawBinField>,
    shortcuts: Option<Vec<Vec<String>>>,
    env_add_path: Option<RawStringList>,
    env_set: Option<BTreeMap<String, String>>,
    persist: Option<RawPersistField>,
    pre_install: Option<RawStringList>,
    post_install: Option<RawStringList>,
    pre_uninstall: Option<RawStringList>,
    post_uninstall: Option<RawStringList>,
    uninstaller: Option<RawScriptOwner>,
    innosetup: Option<bool>,
}

impl RawPackageFields {
    fn overlay(&mut self, other: &Self) {
        if other.version.is_some() { self.version = other.version.clone(); }
        if other.url.is_some() { self.url = other.url.clone(); }
        if other.hash.is_some() { self.hash = other.hash.clone(); }
        if other.depends.is_some() { self.depends = other.depends.clone(); }
        if other.extract_dir.is_some() { self.extract_dir = other.extract_dir.clone(); }
        if other.extract_to.is_some() { self.extract_to = other.extract_to.clone(); }
        if other.installer.is_some() { self.installer = other.installer.clone(); }
        if other.bin.is_some() { self.bin = other.bin.clone(); }
        if other.shortcuts.is_some() { self.shortcuts = other.shortcuts.clone(); }
        if other.env_add_path.is_some() { self.env_add_path = other.env_add_path.clone(); }
        if other.env_set.is_some() { self.env_set = other.env_set.clone(); }
        if other.persist.is_some() { self.persist = other.persist.clone(); }
        if other.pre_install.is_some() { self.pre_install = other.pre_install.clone(); }
        if other.post_install.is_some() { self.post_install = other.post_install.clone(); }
        if other.pre_uninstall.is_some() { self.pre_uninstall = other.pre_uninstall.clone(); }
        if other.post_uninstall.is_some() { self.post_uninstall = other.post_uninstall.clone(); }
        if other.uninstaller.is_some() { self.uninstaller = other.uninstaller.clone(); }
        if other.innosetup.is_some() { self.innosetup = other.innosetup; }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawStringList {
    Single(String),
    Many(Vec<String>),
}

impl RawStringList {
    fn into_trimmed_vec(self) -> Vec<String> {
        match self {
            Self::Single(item) => normalize_string_items([item]),
            Self::Many(items) => normalize_string_items(items),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawScriptOwner { script: Option<RawStringList> }

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawBinField { Single(String), Many(Vec<RawBinEntry>) }

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawBinEntry { Path(String), Tuple(Vec<String>) }

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawPersistField { Single(String), Many(Vec<RawPersistEntry>) }

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawPersistEntry { Path(String), Tuple(Vec<String>) }

// ── Public helpers ──

pub fn current_architecture_key() -> &'static str {
    if cfg!(target_arch = "x86_64") { "64bit" }
    else if cfg!(target_arch = "aarch64") { "arm64" }
    else { "32bit" }
}

pub fn dependency_lookup_key(spec: &str) -> String {
    let trimmed = spec.trim();
    if trimmed.contains("://") {
        return Path::new(trimmed)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or(trimmed)
            .to_string();
    }
    trimmed
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(trimmed)
        .to_string()
}

pub fn resolve_package_source(manifest: &Value) -> Result<ResolvedPackageSource> {
    let raw: RawManifest = serde_json::from_value(manifest.clone())
        .map_err(|err| manifest_error(format!("failed to parse Scoop manifest: {err}")))?;
    let fields = raw.resolve(current_architecture_key());
    Ok(ResolvedPackageSource {
        version: required_trimmed_string(fields.version.clone(), "version")?,
        assets: parse_assets(&fields)?,
        depends: parse_string_list(fields.depends),
        extract_dir: parse_string_list(fields.extract_dir),
        extract_to: parse_string_list(fields.extract_to),
        installer_script: parse_script(fields.installer),
        bins: parse_bins(fields.bin)?,
        shortcuts: parse_shortcuts(fields.shortcuts)?,
        env_add_path: parse_string_list(fields.env_add_path),
        env_set: fields.env_set.unwrap_or_default(),
        persist: parse_persist_entries(fields.persist)?,
        pre_install: parse_string_list(fields.pre_install),
        post_install: parse_string_list(fields.post_install),
        pre_uninstall: parse_string_list(fields.pre_uninstall),
        post_uninstall: parse_string_list(fields.post_uninstall),
        uninstaller_script: parse_script(fields.uninstaller),
    })
}

// ── Internal helpers ──

fn manifest_error(message: impl Into<String>) -> ScoopError {
    ScoopError::ManifestValidation(message.into())
}

fn normalize_string_items<I>(items: I) -> Vec<String>
where I: IntoIterator<Item = String> {
    items.into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn required_trimmed_string(value: Option<String>, key: &str) -> Result<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .ok_or_else(|| manifest_error(format!("manifest is missing `{key}`")))
}

fn parse_string_list(value: Option<RawStringList>) -> Vec<String> {
    value.map(RawStringList::into_trimmed_vec).unwrap_or_default()
}

fn parse_assets(fields: &RawPackageFields) -> Result<Vec<PackageAsset>> {
    let urls = parse_string_list(fields.url.clone());
    let hashes = parse_string_list(fields.hash.clone());
    if urls.is_empty() {
        return Err(manifest_error("manifest is missing supported `url`"));
    }
    if hashes.is_empty() {
        return Err(manifest_error("manifest is missing supported `hash`"));
    }
    if urls.len() != hashes.len() {
        return Err(manifest_error("`url` and `hash` entry counts must match"));
    }
    Ok(urls.into_iter().zip(hashes)
        .map(|(url, hash)| PackageAsset {
            target_name: asset_target_name(&url),
            url, hash,
        })
        .collect())
}

fn asset_target_name(url: &str) -> Option<String> {
    Path::new(url).file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.to_string())
}

fn parse_script(owner: Option<RawScriptOwner>) -> Vec<String> {
    owner.and_then(|owner| owner.script)
        .map_or_else(Vec::new, |script| parse_string_list(Some(script)))
}

fn parse_bins(value: Option<RawBinField>) -> Result<Vec<ShimTarget>> {
    let mut bins = Vec::new();
    match value {
        None => {}
        Some(RawBinField::Single(path)) => {
            let path = path.trim().to_string();
            if !path.is_empty() {
                let alias = Path::new(&path)
                    .file_stem().and_then(|v| v.to_str()).unwrap_or(&path).to_string();
                bins.push(ShimTarget { relative_path: path, alias, args: Vec::new() });
            }
        }
        Some(RawBinField::Many(entries)) => {
            for entry in entries {
                match entry {
                    RawBinEntry::Path(path) => {
                        let path = path.trim().to_string();
                        if path.is_empty() { continue; }
                        let alias = Path::new(&path)
                            .file_stem().and_then(|v| v.to_str()).unwrap_or(&path).to_string();
                        bins.push(ShimTarget { relative_path: path, alias, args: Vec::new() });
                    }
                    RawBinEntry::Tuple(parts) if !parts.is_empty() => {
                        let relative = parts[0].trim().to_string();
                        if relative.is_empty() { continue; }
                        let alias = parts.get(1)
                            .map(|v| v.trim().to_string())
                            .filter(|v| !v.is_empty())
                            .unwrap_or_else(|| Path::new(&relative)
                                .file_stem().and_then(|v| v.to_str()).unwrap_or(&relative).to_string());
                        let args = parts.iter().skip(2)
                            .map(|v| v.trim().to_string())
                            .filter(|v| !v.is_empty())
                            .collect();
                        bins.push(ShimTarget { relative_path: relative, alias, args });
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(bins)
}

fn parse_shortcuts(entries: Option<Vec<Vec<String>>>) -> Result<Vec<ShortcutEntry>> {
    let mut shortcuts = Vec::new();
    for entry in entries.unwrap_or_default() {
        if entry.is_empty() { continue; }
        let target_path = entry[0].trim().to_string();
        if target_path.is_empty() { continue; }
        let name = entry.get(1)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| Path::new(&target_path)
                .file_stem().and_then(|v| v.to_str()).unwrap_or(&target_path).to_string());
        let args = entry.get(2).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let icon_path = entry.get(3).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        shortcuts.push(ShortcutEntry { target_path, name, args, icon_path });
    }
    Ok(shortcuts)
}

fn parse_persist_entries(value: Option<RawPersistField>) -> Result<Vec<PersistEntry>> {
    let mut entries = Vec::new();
    match value {
        None => {}
        Some(RawPersistField::Single(path)) => {
            let path = path.trim().to_string();
            if !path.is_empty() {
                entries.push(PersistEntry { relative_path: path.clone(), store_name: path });
            }
        }
        Some(RawPersistField::Many(items)) => {
            for entry in items {
                match entry {
                    RawPersistEntry::Path(path) => {
                        let path = path.trim().to_string();
                        if path.is_empty() { continue; }
                        entries.push(PersistEntry { relative_path: path.clone(), store_name: path });
                    }
                    RawPersistEntry::Tuple(parts) if !parts.is_empty() => {
                        let relative = parts[0].trim().to_string();
                        if relative.is_empty() { continue; }
                        let store_name = parts.get(1)
                            .map(|v| v.trim().to_string())
                            .filter(|v| !v.is_empty())
                            .unwrap_or_else(|| relative.clone());
                        entries.push(PersistEntry { relative_path: relative, store_name });
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(entries)
}
