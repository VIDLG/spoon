use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::facts::package_rules::{ManagedPackageKind, package_kind};
use spoon_core::CoreError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainTarget {
    pub msvc: String,
    pub sdk: String,
}

impl ToolchainTarget {
    pub fn label(&self) -> String {
        format!("{} + {}", self.msvc, self.sdk)
    }
}

pub fn version_key(token: &str) -> Vec<u32> {
    token
        .split_once('-')
        .map(|(_, version)| {
            version
                .split(['.', '_'])
                .filter_map(|part| part.parse::<u32>().ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub fn pick_higher_version(current: &mut Option<String>, candidate: String) {
    match current {
        Some(existing) if version_key(existing) >= version_key(&candidate) => {}
        _ => *current = Some(candidate),
    }
}

pub fn installed_state_path(msvc_root: &Path) -> PathBuf {
    msvc_root.join("state").join("installed.json")
}

pub fn write_installed_toolchain_target(
    msvc_root: &Path,
    target: &ToolchainTarget,
) -> Result<(), CoreError> {
    let state_path = installed_state_path(msvc_root);
    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| CoreError::fs("create_dir_all", parent, e))?;
    }
    let content = serde_json::to_string_pretty(target)
        .map_err(|e| CoreError::Other(e.to_string()))?;
    fs::write(&state_path, content)
        .map_err(|e| CoreError::fs("write", &state_path, e))?;
    Ok(())
}

pub fn read_installed_toolchain_target(msvc_root: &Path) -> Option<ToolchainTarget> {
    let content = fs::read_to_string(installed_state_path(msvc_root)).ok()?;
    let target = serde_json::from_str::<ToolchainTarget>(&content).ok()?;
    if target.msvc.trim().is_empty() || target.sdk.trim().is_empty() {
        return None;
    }
    Some(target)
}

pub fn package_token_after_prefix(line: &str, prefix: &str) -> Option<String> {
    line.split_whitespace()
        .map(|token| {
            token.trim_matches(|ch: char| {
                matches!(ch, ',' | ';' | '"' | '\'' | '[' | ']' | '(' | ')')
            })
        })
        .find(|token| token.starts_with(prefix))
        .map(ToString::to_string)
}

pub fn parse_toolchain_target_from_lines(lines: &[String]) -> Option<ToolchainTarget> {
    let mut packages = Vec::new();
    for line in lines {
        if let Some(token) = package_token_after_prefix(line, "msvc-") {
            packages.push(token);
        }
        if let Some(token) = package_token_after_prefix(line, "sdk-") {
            packages.push(token);
        }
    }
    select_latest_toolchain_from_packages(packages.iter().map(String::as_str))
}

pub fn select_latest_toolchain_from_packages<'a>(
    packages: impl IntoIterator<Item = &'a str>,
) -> Option<ToolchainTarget> {
    let mut msvc = None;
    let mut sdk = None;
    for package in packages {
        match package_kind(package) {
            ManagedPackageKind::Msvc => pick_higher_version(&mut msvc, package.to_string()),
            ManagedPackageKind::Sdk => pick_higher_version(&mut sdk, package.to_string()),
            ManagedPackageKind::Autoenv | ManagedPackageKind::Unknown => {}
        }
    }
    Some(ToolchainTarget {
        msvc: msvc?,
        sdk: sdk?,
    })
}
