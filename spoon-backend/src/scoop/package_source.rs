use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{BackendError, Result};

#[derive(Debug, Clone)]
pub struct SelectedPackageSource {
    pub version: String,
    pub payloads: Vec<PackagePayload>,
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
pub struct PackagePayload {
    pub url: String,
    pub hash: String,
    pub target_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ShimTarget {
    pub relative_path: String,
    pub alias: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistEntry {
    pub relative_path: String,
    pub store_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutEntry {
    pub target_path: String,
    pub name: String,
    pub args: Option<String>,
    pub icon_path: Option<String>,
}

pub fn selected_architecture_key() -> &'static str {
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

fn value_from_manifest<'a>(manifest: &'a Value, key: &str) -> Option<&'a Value> {
    manifest
        .get("architecture")
        .and_then(|value| value.get(selected_architecture_key()))
        .and_then(|value| value.get(key))
        .or_else(|| manifest.get(key))
}

fn manifest_error(message: impl Into<String>) -> BackendError {
    BackendError::ManifestValidation(message.into())
}

fn string_value(manifest: &Value, key: &str) -> Option<String> {
    value_from_manifest(manifest, key)
        .and_then(Value::as_str)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_string_list(manifest: &Value, key: &str) -> Result<Vec<String>> {
    let Some(value) = value_from_manifest(manifest, key) else {
        return Ok(Vec::new());
    };
    match value {
        Value::String(item) => Ok(vec![item.trim().to_string()]
            .into_iter()
            .filter(|item| !item.is_empty())
            .collect()),
        Value::Array(items) => Ok(items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToString::to_string)
            .collect()),
        _ => Err(manifest_error(format!("unsupported `{key}` field shape"))),
    }
}

fn parse_payloads(manifest: &Value) -> Result<Vec<PackagePayload>> {
    let urls = parse_string_list(manifest, "url")?;
    let hashes = parse_string_list(manifest, "hash")?;
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
        .map(|(url, hash)| PackagePayload {
            target_name: payload_target_name(&url),
            url,
            hash,
        })
        .collect())
}

fn payload_target_name(url: &str) -> Option<String> {
    let fragment = url.split('#').nth(1)?.trim();
    let trimmed = fragment.strip_prefix('/')?.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.replace('/', "\\"))
    }
}

fn parse_env_set(manifest: &Value) -> Result<BTreeMap<String, String>> {
    let Some(value) = value_from_manifest(manifest, "env_set") else {
        return Ok(BTreeMap::new());
    };
    let Value::Object(map) = value else {
        return Err(manifest_error("unsupported `env_set` field shape"));
    };
    Ok(map
        .iter()
        .filter_map(|(key, value)| {
            value
                .as_str()
                .map(|value| (key.clone(), value.trim().to_string()))
        })
        .filter(|(_, value)| !value.is_empty())
        .collect())
}

fn parse_persist_entries(manifest: &Value) -> Result<Vec<PersistEntry>> {
    let Some(value) = value_from_manifest(manifest, "persist") else {
        return Ok(Vec::new());
    };
    let mut entries = Vec::new();
    match value {
        Value::String(path) => {
            let path = path.trim();
            if !path.is_empty() {
                entries.push(PersistEntry {
                    relative_path: path.to_string(),
                    store_name: path.to_string(),
                });
            }
        }
        Value::Array(items) => {
            for item in items {
                match item {
                    Value::String(path) => {
                        let path = path.trim();
                        if !path.is_empty() {
                            entries.push(PersistEntry {
                                relative_path: path.to_string(),
                                store_name: path.to_string(),
                            });
                        }
                    }
                    Value::Array(parts) if !parts.is_empty() => {
                        let relative_path = parts
                            .first()
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .ok_or_else(|| manifest_error("unsupported persist tuple path"))?;
                        let store_name = parts
                            .get(1)
                            .and_then(Value::as_str)
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
        _ => return Err(manifest_error("unsupported `persist` field shape")),
    }
    Ok(entries)
}

fn parse_shortcuts(manifest: &Value) -> Result<Vec<ShortcutEntry>> {
    let Some(value) = value_from_manifest(manifest, "shortcuts") else {
        return Ok(Vec::new());
    };
    let Value::Array(items) = value else {
        return Err(manifest_error("unsupported `shortcuts` field shape"));
    };
    let mut shortcuts = Vec::new();
    for item in items {
        let Value::Array(parts) = item else {
            return Err(manifest_error("unsupported `shortcuts` item shape"));
        };
        if !(2..=4).contains(&parts.len()) {
            return Err(manifest_error("unsupported `shortcuts` tuple length"));
        }
        let target_path = parts
            .first()
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| manifest_error("unsupported shortcuts target path"))?;
        let name = parts
            .get(1)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| manifest_error("unsupported shortcuts name"))?;
        let args = parts
            .get(2)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        let icon_path = parts
            .get(3)
            .and_then(Value::as_str)
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

fn parse_bin_targets(manifest: &Value) -> Result<Vec<ShimTarget>> {
    let Some(bin_value) = value_from_manifest(manifest, "bin") else {
        return Err(manifest_error(
            "manifest is missing a supported `bin` field",
        ));
    };
    let mut targets = Vec::new();
    match bin_value {
        Value::String(path) => targets.push(ShimTarget {
            relative_path: path.to_string(),
            alias: Path::new(path)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or(path)
                .to_string(),
            args: Vec::new(),
        }),
        Value::Array(items) => {
            for item in items {
                match item {
                    Value::String(path) => targets.push(ShimTarget {
                        relative_path: path.to_string(),
                        alias: Path::new(path)
                            .file_stem()
                            .and_then(|value| value.to_str())
                            .unwrap_or(path)
                            .to_string(),
                        args: Vec::new(),
                    }),
                    Value::Array(parts) if !parts.is_empty() => {
                        let path = parts
                            .first()
                            .and_then(Value::as_str)
                            .ok_or_else(|| manifest_error("unsupported bin tuple path"))?;
                        let alias = parts.get(1).and_then(Value::as_str).unwrap_or_else(|| {
                            Path::new(path)
                                .file_stem()
                                .and_then(|value| value.to_str())
                                .unwrap_or(path)
                        });
                        let args = parts
                            .get(2)
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(|value| vec![value.to_string()])
                            .unwrap_or_default();
                        targets.push(ShimTarget {
                            relative_path: path.to_string(),
                            alias: alias.to_string(),
                            args,
                        });
                    }
                    _ => return Err(manifest_error("unsupported `bin` item shape")),
                }
            }
        }
        _ => return Err(manifest_error("unsupported `bin` field shape")),
    }
    Ok(targets)
}

pub fn parse_selected_source(manifest: &Value) -> Result<SelectedPackageSource> {
    let version = string_value(manifest, "version")
        .ok_or_else(|| manifest_error("manifest is missing `version`"))?;
    let payloads = parse_payloads(manifest)?;
    let mut depends = parse_string_list(manifest, "depends")?;
    let extract_dir = parse_string_list(manifest, "extract_dir")?;
    let extract_to = parse_string_list(manifest, "extract_to")?;
    let installer_script = parse_installer_script_lines(manifest)?;
    infer_helper_dependencies(&payloads, manifest, &installer_script, &mut depends);
    let bins = parse_bin_targets(manifest)?;
    let shortcuts = parse_shortcuts(manifest)?;
    let env_add_path = parse_string_list(manifest, "env_add_path")?;
    let env_set = parse_env_set(manifest)?;
    let persist = parse_persist_entries(manifest)?;
    let pre_install = parse_lifecycle_script_lines(manifest, "pre_install")?;
    let post_install = parse_lifecycle_script_lines(manifest, "post_install")?;
    let pre_uninstall = parse_lifecycle_script_lines(manifest, "pre_uninstall")?;
    let post_uninstall = parse_lifecycle_script_lines(manifest, "post_uninstall")?;
    let uninstaller_script = parse_uninstaller_script_lines(manifest)?;
    Ok(SelectedPackageSource {
        version,
        payloads,
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

fn parse_lifecycle_script_lines(manifest: &Value, key: &str) -> Result<Vec<String>> {
    let Some(value) = value_from_manifest(manifest, key) else {
        return Ok(Vec::new());
    };
    parse_script_lines(value)
}

fn parse_uninstaller_script_lines(manifest: &Value) -> Result<Vec<String>> {
    let Some(uninstaller) = value_from_manifest(manifest, "uninstaller") else {
        return Ok(Vec::new());
    };
    let Some(script) = uninstaller.get("script") else {
        return Ok(Vec::new());
    };
    parse_script_lines(script)
}

fn parse_installer_script_lines(manifest: &Value) -> Result<Vec<String>> {
    let Some(installer) = value_from_manifest(manifest, "installer") else {
        return Ok(Vec::new());
    };
    let Some(script) = installer.get("script") else {
        return Ok(Vec::new());
    };
    parse_script_lines(script)
}

fn parse_script_lines(value: &Value) -> Result<Vec<String>> {
    match value {
        Value::String(line) => Ok(vec![line.trim().to_string()]
            .into_iter()
            .filter(|line| !line.is_empty())
            .collect()),
        Value::Array(items) => Ok(items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToString::to_string)
            .collect()),
        _ => Err(manifest_error("unsupported script field shape")),
    }
}

fn infer_helper_dependencies(
    payloads: &[PackagePayload],
    manifest: &Value,
    installer_script: &[String],
    depends: &mut Vec<String>,
) {
    let needs_innounp = value_from_manifest(manifest, "innosetup")
        .and_then(Value::as_bool)
        .unwrap_or(false)
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
    let needs_7zip = payloads.iter().any(|payload| {
        payload
            .target_name
            .as_deref()
            .or_else(|| {
                Path::new(&payload.url)
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
