use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::package_rules::{ManagedPackageKind, package_kind};

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
) -> crate::Result<()> {
    if let Some(parent) = installed_state_path(msvc_root).parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(target)?;
    fs::write(installed_state_path(msvc_root), content)?;
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        ToolchainTarget, installed_state_path, parse_toolchain_target_from_lines,
        read_installed_toolchain_target, select_latest_toolchain_from_packages,
        write_installed_toolchain_target,
    };

    #[test]
    fn parse_toolchain_target_picks_highest_versions() {
        let lines = vec![
            "available sdk-10.0.22621.7".to_string(),
            "available msvc-14.43.12.1".to_string(),
            "available msvc-14.44.17.14".to_string(),
            "available sdk-10.0.26100.1".to_string(),
        ];

        let target = parse_toolchain_target_from_lines(&lines).expect("target");
        assert_eq!(target.msvc, "msvc-14.44.17.14");
        assert_eq!(target.sdk, "sdk-10.0.26100.1");
        assert_eq!(target.label(), "msvc-14.44.17.14 + sdk-10.0.26100.1");
    }

    #[test]
    fn select_latest_toolchain_from_packages_ignores_autoenv_and_unknowns() {
        let packages = [
            "autoenv",
            "msvc-14.43.12.1",
            "sdk-10.0.22621.7",
            "ninja-1.12.1",
            "msvc-14.44.17.14",
        ];

        let target = select_latest_toolchain_from_packages(packages).expect("target");
        assert_eq!(target.label(), "msvc-14.44.17.14 + sdk-10.0.22621.7");
    }

    #[test]
    fn read_installed_toolchain_target_reads_installed_state_file() {
        let root = std::env::temp_dir().join(format!(
            "spoon-msvc-rules-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let msvc_root = root.join("msvc");
        fs::create_dir_all(&msvc_root).unwrap();
        write_installed_toolchain_target(
            &msvc_root,
            &ToolchainTarget {
                msvc: "msvc-14.44.17.14".to_string(),
                sdk: "sdk-10.0.22621.7".to_string(),
            },
        )
        .unwrap();

        let target = read_installed_toolchain_target(&msvc_root).expect("installed target");
        assert_eq!(target.label(), "msvc-14.44.17.14 + sdk-10.0.22621.7");
        assert!(installed_state_path(&msvc_root).exists());
    }
}
