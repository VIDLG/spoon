use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::{BackendError, Result};

use super::models::{PersistEntry, ShimTarget, ShortcutEntry};

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

#[derive(Debug, Clone)]
pub struct PackageAsset {
    pub url: String,
    pub hash: String,
    pub target_name: Option<String>,
}

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
        if other.version.is_some() {
            self.version = other.version.clone();
        }
        if other.url.is_some() {
            self.url = other.url.clone();
        }
        if other.hash.is_some() {
            self.hash = other.hash.clone();
        }
        if other.depends.is_some() {
            self.depends = other.depends.clone();
        }
        if other.extract_dir.is_some() {
            self.extract_dir = other.extract_dir.clone();
        }
        if other.extract_to.is_some() {
            self.extract_to = other.extract_to.clone();
        }
        if other.installer.is_some() {
            self.installer = other.installer.clone();
        }
        if other.bin.is_some() {
            self.bin = other.bin.clone();
        }
        if other.shortcuts.is_some() {
            self.shortcuts = other.shortcuts.clone();
        }
        if other.env_add_path.is_some() {
            self.env_add_path = other.env_add_path.clone();
        }
        if other.env_set.is_some() {
            self.env_set = other.env_set.clone();
        }
        if other.persist.is_some() {
            self.persist = other.persist.clone();
        }
        if other.pre_install.is_some() {
            self.pre_install = other.pre_install.clone();
        }
        if other.post_install.is_some() {
            self.post_install = other.post_install.clone();
        }
        if other.pre_uninstall.is_some() {
            self.pre_uninstall = other.pre_uninstall.clone();
        }
        if other.post_uninstall.is_some() {
            self.post_uninstall = other.post_uninstall.clone();
        }
        if other.uninstaller.is_some() {
            self.uninstaller = other.uninstaller.clone();
        }
        if other.innosetup.is_some() {
            self.innosetup = other.innosetup;
        }
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
struct RawScriptOwner {
    script: Option<RawStringList>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawBinField {
    Single(String),
    Many(Vec<RawBinEntry>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawBinEntry {
    Path(String),
    Tuple(Vec<String>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawPersistField {
    Single(String),
    Many(Vec<RawPersistEntry>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum RawPersistEntry {
    Path(String),
    Tuple(Vec<String>),
}

pub fn current_architecture_key() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "64bit"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "32bit"
    }
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

fn manifest_error(message: impl Into<String>) -> BackendError {
    BackendError::ManifestValidation(message.into())
}

fn normalize_string_items<I>(items: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    items.into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn required_trimmed_string(value: Option<String>, key: &str) -> Result<String> {
    value.map(|item| item.trim().to_string())
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
    Ok(urls
        .into_iter()
        .zip(hashes)
        .map(|(url, hash)| PackageAsset {
            target_name: asset_target_name(&url),
            url,
            hash,
        })
        .collect())
}

fn asset_target_name(url: &str) -> Option<String> {
    let fragment = url.split('#').nth(1)?.trim();
    let trimmed = fragment.strip_prefix('/')?.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.replace('/', "\\"))
    }
}

fn parse_env_set(value: Option<BTreeMap<String, String>>) -> BTreeMap<String, String> {
    value
        .unwrap_or_default()
        .into_iter()
        .map(|(key, value)| (key, value.trim().to_string()))
        .filter(|(_, value)| !value.is_empty())
        .collect()
}

fn parse_persist_entries(value: Option<RawPersistField>) -> Result<Vec<PersistEntry>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let mut entries = Vec::new();
    match value {
        RawPersistField::Single(path) => {
            let path = path.trim();
            if !path.is_empty() {
                entries.push(PersistEntry {
                    relative_path: path.to_string(),
                    store_name: path.to_string(),
                });
            }
        }
        RawPersistField::Many(items) => {
            for item in items {
                match item {
                    RawPersistEntry::Path(path) => {
                        let path = path.trim();
                        if !path.is_empty() {
                            entries.push(PersistEntry {
                                relative_path: path.to_string(),
                                store_name: path.to_string(),
                            });
                        }
                    }
                    RawPersistEntry::Tuple(parts) if !parts.is_empty() => {
                        let relative_path = parts
                            .first()
                            .map(String::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .ok_or_else(|| manifest_error("unsupported persist tuple path"))?;
                        let store_name = parts
                            .get(1)
                            .map(String::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .unwrap_or(relative_path);
                        entries.push(PersistEntry {
                            relative_path: relative_path.to_string(),
                            store_name: store_name.to_string(),
                        });
                    }
                    _ => return Err(manifest_error("unsupported `persist` item shape")),
                }
            }
        }
    }
    Ok(entries)
}

fn parse_shortcuts(value: Option<Vec<Vec<String>>>) -> Result<Vec<ShortcutEntry>> {
    let Some(items) = value else {
        return Ok(Vec::new());
    };
    let mut shortcuts = Vec::new();
    for parts in items {
        if !(2..=4).contains(&parts.len()) {
            return Err(manifest_error("unsupported `shortcuts` tuple length"));
        }
        let target_path = parts
            .first()
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| manifest_error("unsupported shortcuts target path"))?;
        let name = parts
            .get(1)
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| manifest_error("unsupported shortcuts name"))?;
        let args = parts
            .get(2)
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        let icon_path = parts
            .get(3)
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        shortcuts.push(ShortcutEntry {
            target_path: target_path.to_string(),
            name: name.to_string(),
            args,
            icon_path,
        });
    }
    Ok(shortcuts)
}

fn parse_bin_targets(value: Option<RawBinField>) -> Result<Vec<ShimTarget>> {
    let Some(bin_value) = value else {
        return Err(manifest_error(
            "manifest is missing a supported `bin` field",
        ));
    };
    let mut targets = Vec::new();
    match bin_value {
        RawBinField::Single(path) => targets.push(shim_target(path, None, Vec::new())),
        RawBinField::Many(items) => {
            for item in items {
                match item {
                    RawBinEntry::Path(path) => targets.push(shim_target(path, None, Vec::new())),
                    RawBinEntry::Tuple(parts) if !parts.is_empty() => {
                        let path = parts
                            .first()
                            .map(String::as_str)
                            .ok_or_else(|| manifest_error("unsupported bin tuple path"))?;
                        let alias = parts.get(1).map(String::as_str);
                        let args = parts
                            .get(2)
                            .map(String::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(|value| vec![value.to_string()])
                            .unwrap_or_default();
                        targets.push(shim_target(path.to_string(), alias, args));
                    }
                    _ => return Err(manifest_error("unsupported `bin` item shape")),
                }
            }
        }
    }
    Ok(targets)
}

fn shim_target(path: String, alias: Option<&str>, args: Vec<String>) -> ShimTarget {
    let inferred_alias = alias
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            Path::new(&path)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or(path.as_str())
                .to_string()
        });
    ShimTarget {
        relative_path: path,
        alias: inferred_alias,
        args,
    }
}

pub fn resolve_package_source(manifest: &Value) -> Result<ResolvedPackageSource> {
    let raw: RawManifest = serde_json::from_value(manifest.clone())
        .map_err(|err| manifest_error(format!("invalid Scoop manifest shape: {err}")))?;
    let fields = raw.resolve(current_architecture_key());

    let version = required_trimmed_string(fields.version.clone(), "version")?;
    let assets = parse_assets(&fields)?;
    let mut depends = parse_string_list(fields.depends.clone());
    let extract_dir = parse_string_list(fields.extract_dir.clone());
    let extract_to = parse_string_list(fields.extract_to.clone());
    let installer_script = parse_script_lines(fields.installer.clone());
    infer_helper_dependencies(&assets, &fields, &installer_script, &mut depends);
    let bins = parse_bin_targets(fields.bin.clone())?;
    let shortcuts = parse_shortcuts(fields.shortcuts.clone())?;
    let env_add_path = parse_string_list(fields.env_add_path.clone());
    let env_set = parse_env_set(fields.env_set.clone());
    let persist = parse_persist_entries(fields.persist.clone())?;
    let pre_install = parse_string_list(fields.pre_install.clone());
    let post_install = parse_string_list(fields.post_install.clone());
    let pre_uninstall = parse_string_list(fields.pre_uninstall.clone());
    let post_uninstall = parse_string_list(fields.post_uninstall.clone());
    let uninstaller_script = parse_script_lines(fields.uninstaller.clone());

    Ok(ResolvedPackageSource {
        version,
        assets,
        depends,
        extract_dir,
        extract_to,
        installer_script,
        bins,
        shortcuts,
        env_add_path,
        env_set,
        persist,
        pre_install,
        post_install,
        pre_uninstall,
        post_uninstall,
        uninstaller_script,
    })
}

fn parse_script_lines(owner: Option<RawScriptOwner>) -> Vec<String> {
    owner.and_then(|owner| owner.script)
        .map(RawStringList::into_trimmed_vec)
        .unwrap_or_default()
}

fn infer_helper_dependencies(
    assets: &[PackageAsset],
    fields: &RawPackageFields,
    installer_script: &[String],
    depends: &mut Vec<String>,
) {
    let needs_innounp = fields.innosetup.unwrap_or(false)
        || installer_script
            .iter()
            .any(|line| line.contains("Expand-InnoArchive"));
    if needs_innounp
        && !depends
            .iter()
            .any(|item| dependency_lookup_key(item) == "innounp")
    {
        depends.push("innounp".to_string());
    }
    let needs_dark = installer_script
        .iter()
        .any(|line| line.contains("Expand-DarkArchive"));
    if needs_dark
        && !depends
            .iter()
            .any(|item| dependency_lookup_key(item) == "dark")
    {
        depends.push("dark".to_string());
    }
    let needs_7zip = assets.iter().any(|asset| {
        asset
            .target_name
            .as_deref()
            .or_else(|| {
                Path::new(&asset.url)
                    .file_name()
                    .and_then(|value| value.to_str())
            })
            .is_some_and(|name| {
                name.eq_ignore_ascii_case("dl.7z") || name.to_ascii_lowercase().ends_with(".7z")
            })
    });
    if needs_7zip
        && !depends
            .iter()
            .any(|item| dependency_lookup_key(item) == "7zip")
    {
        depends.push("7zip".to_string());
    }
}
