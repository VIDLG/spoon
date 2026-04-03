use std::fs;
use std::path::Path;

use serde_json::Value;

use super::manifest;

pub fn value_to_display(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(text) => {
            let text = text.trim();
            if text.is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        }
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        Value::Array(items) => {
            let rendered = items
                .iter()
                .filter_map(value_to_display)
                .collect::<Vec<_>>();
            if rendered.is_empty() {
                None
            } else {
                Some(rendered.join(", "))
            }
        }
        Value::Object(map) => {
            let rendered = map
                .iter()
                .filter_map(|(key, value)| {
                    value_to_display(value).map(|value| format!("{key}={value}"))
                })
                .collect::<Vec<_>>();
            if rendered.is_empty() {
                None
            } else {
                Some(rendered.join(", "))
            }
        }
    }
}

pub fn json_value_or_display(value: &Value) -> Option<Value> {
    match value {
        Value::Null => None,
        Value::Array(items) if items.is_empty() => None,
        Value::Object(map) if map.is_empty() => None,
        _ => Some(value.clone()),
    }
}

pub fn license_display_value(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => {
            let text = text.trim();
            if text.is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        }
        Value::Object(map) => map
            .get("identifier")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| value_to_display(value)),
        _ => value_to_display(value),
    }
}

pub fn integration_display_key(package_name: &str, key: &str) -> String {
    key.strip_prefix(&format!("{package_name}."))
        .unwrap_or(key)
        .to_string()
}

pub fn policy_config_kind(key: &str) -> Option<&'static str> {
    if key.ends_with("config_dir") || key.ends_with("config_root") {
        Some("config directories")
    } else if key.ends_with("config") || key.ends_with("config_file") {
        Some("config files")
    } else {
        None
    }
}

fn collect_urls(value: &Value, urls: &mut Vec<String>, include_hash_urls: bool) {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
        Value::String(text) => {
            let text = text.trim();
            if !text.is_empty()
                && (text.contains("://")
                    || text.starts_with("http://")
                    || text.starts_with("https://")
                    || text.starts_with("file://"))
            {
                urls.push(text.to_string());
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_urls(item, urls, include_hash_urls);
            }
        }
        Value::Object(map) => {
            if let Some(architectures) = map.get("architecture").and_then(Value::as_object) {
                for value in architectures.values() {
                    collect_urls(value, urls, include_hash_urls);
                }
            }
            for (key, value) in map {
                if !include_hash_urls && key.eq_ignore_ascii_case("hash") {
                    continue;
                }
                collect_urls(value, urls, include_hash_urls);
            }
        }
    }
}

pub fn collect_urls_vec(value: &Value, include_hash_urls: bool) -> Vec<String> {
    let mut urls = Vec::new();
    collect_urls(value, &mut urls, include_hash_urls);
    urls
}

pub fn manifest_value(doc: &manifest::ScoopManifest, key: &str) -> Option<Value> {
    serde_json::to_value(doc)
        .ok()?
        .as_object()?
        .get(key)
        .cloned()
}

pub fn manifest_value_owned(doc: &manifest::ScoopManifest, key: &str) -> Option<Value> {
    manifest_value(doc, key)
}

fn bin_item_display(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => {
            let text = text.trim();
            if text.is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        }
        Value::Array(items) => {
            let rendered = items
                .iter()
                .filter_map(value_to_display)
                .collect::<Vec<_>>();
            if rendered.is_empty() {
                None
            } else {
                Some(rendered.join(" -> "))
            }
        }
        _ => value_to_display(value),
    }
}

pub fn collect_bin_items(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items.iter().filter_map(bin_item_display).collect(),
        _ => bin_item_display(value).into_iter().collect(),
    }
}

fn shortcut_item_display(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => {
            let text = text.trim();
            if text.is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        }
        Value::Array(items) => {
            let rendered = items
                .iter()
                .filter_map(value_to_display)
                .collect::<Vec<_>>();
            if rendered.is_empty() {
                None
            } else {
                Some(rendered.join(" -> "))
            }
        }
        Value::Object(map) => {
            let name = map
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let target = map
                .get("target")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let args = map
                .get("args")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty());

            match (name, target, args) {
                (Some(name), Some(target), Some(args)) => {
                    Some(format!("{name} -> {target} {args}"))
                }
                (Some(name), Some(target), None) => Some(format!("{name} -> {target}")),
                (None, Some(target), Some(args)) => Some(format!("{target} {args}")),
                (None, Some(target), None) => Some(target.to_string()),
                (Some(name), None, _) => Some(name.to_string()),
                _ => value_to_display(value),
            }
        }
        _ => value_to_display(value),
    }
}

pub fn collect_shortcut_items(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items.iter().filter_map(shortcut_item_display).collect(),
        _ => shortcut_item_display(value).into_iter().collect(),
    }
}

pub fn string_items(value: Option<Value>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    match value {
        Value::String(text) => {
            let text = text.trim();
            if text.is_empty() {
                Vec::new()
            } else {
                vec![text.to_string()]
            }
        }
        Value::Array(items) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

pub fn string_map_items(value: Option<Value>) -> Vec<(String, String)> {
    let Some(Value::Object(map)) = value else {
        return Vec::new();
    };
    map.iter()
        .filter_map(|(key, value)| {
            value
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| (key.clone(), value.to_string()))
        })
        .collect()
}

fn resolve_install_placeholder(value: &str, current_root: &Path, persist_root: &Path) -> String {
    value
        .replace("$dir", &current_root.display().to_string())
        .replace("$persist_dir", &persist_root.display().to_string())
        .replace("$original_dir", &current_root.display().to_string())
}

pub fn resolve_env_paths(
    entries: Vec<String>,
    current_root: &Path,
    persist_root: &Path,
) -> Vec<String> {
    entries
        .into_iter()
        .map(|entry| resolve_install_placeholder(&entry, current_root, persist_root))
        .map(|entry| {
            let path = Path::new(&entry);
            if path.is_absolute() {
                entry
            } else {
                current_root.join(path).display().to_string()
            }
        })
        .collect()
}

pub fn resolve_env_map(
    entries: Vec<(String, String)>,
    current_root: &Path,
    persist_root: &Path,
) -> Vec<String> {
    entries
        .into_iter()
        .map(|(key, value)| {
            format!(
                "{key}={}",
                resolve_install_placeholder(&value, current_root, persist_root)
            )
        })
        .collect()
}

pub fn directory_size(path: &Path) -> u64 {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return 0;
    };
    if metadata.file_type().is_symlink() {
        return 0;
    }
    if metadata.is_file() {
        return metadata.len();
    }
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| directory_size(&entry.path()))
        .sum()
}
