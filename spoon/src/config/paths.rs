use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use std::fs;

use super::io::{
    load_claude_config, load_codex_config, load_global_config, save_claude_config,
    save_codex_config, save_global_config,
};
use super::state::home_dir;

pub fn spoon_home_dir() -> PathBuf {
    home_dir().join(".spoon")
}

pub fn git_config_path() -> PathBuf {
    home_dir().join(".gitconfig")
}

pub fn claude_settings_path() -> PathBuf {
    home_dir().join(".claude").join("settings.json")
}

pub fn codex_config_path() -> PathBuf {
    home_dir().join(".codex").join("config.toml")
}

pub fn codex_auth_path() -> PathBuf {
    home_dir().join(".codex").join("auth.json")
}

pub fn pip_config_path() -> PathBuf {
    let config_root =
        if super::state::test_mode_enabled() || std::env::var_os("SPOON_TEST_HOME").is_some() {
            home_dir().join("AppData").join("Roaming")
        } else {
            dirs::config_dir().unwrap_or_else(|| home_dir().join("AppData").join("Roaming"))
        };
    config_root.join("pip").join("pip.ini")
}

pub fn global_config_path() -> PathBuf {
    spoon_home_dir().join("config.toml")
}

pub fn ensure_global_config_exists() -> Result<PathBuf> {
    let path = global_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    if !path.exists() {
        save_global_config(&load_global_config())?;
    }
    Ok(path)
}

pub fn ensure_claude_settings_exists() -> Result<PathBuf> {
    let path = claude_settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    if !path.exists() {
        save_claude_config(&load_claude_config())?;
    }
    Ok(path)
}

pub fn ensure_codex_config_exists(default_model: &str) -> Result<PathBuf> {
    let path = codex_config_path();
    let auth_path = codex_auth_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    if !path.exists() || !auth_path.exists() {
        save_codex_config(&load_codex_config(default_model))?;
    }
    Ok(path)
}

pub fn ensure_git_config_parent_exists() -> Result<PathBuf> {
    let path = git_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    Ok(path)
}

pub fn ensure_msvc_root_exists() -> Result<PathBuf> {
    let global = load_global_config();
    let trimmed = global.root.trim();
    if trimmed.is_empty() {
        anyhow::bail!("root is not configured");
    }
    let path = msvc_root_from(Path::new(trimmed));
    fs::create_dir_all(&path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(path)
}

pub fn configured_tool_root() -> Option<PathBuf> {
    let config = load_global_config();
    let trimmed = config.root.trim();
    if !trimmed.is_empty() {
        return Some(PathBuf::from(trimmed));
    }
    None
}

// Deprecated: use RuntimeLayout::from_root(root).msvc.root instead (per D-10, D-11)
#[deprecated]
pub fn msvc_domain_root_from(tool_root: &Path) -> PathBuf {
    tool_root.join("msvc")
}

// Deprecated: use RuntimeLayout::from_root(root).scoop.root instead (per D-10, D-11)
#[deprecated]
pub fn scoop_root_from(tool_root: &Path) -> PathBuf {
    tool_root.join("scoop")
}

// Deprecated: use RuntimeLayout::from_root(root).shims instead (per D-10, D-11)
#[deprecated]
pub fn shims_root_from(tool_root: &Path) -> PathBuf {
    tool_root.join("shims")
}

// Deprecated: use RuntimeLayout::from_root(root).scoop.state_root instead (per D-10, D-11)
#[deprecated]
pub fn scoop_state_root_from(tool_root: &Path) -> PathBuf {
    scoop_root_from(tool_root).join("state")
}

// Deprecated: use RuntimeLayout::from_root(root).scoop.bucket_registry_path instead (per D-10, D-11)
#[deprecated]
pub fn scoop_bucket_registry_path_from(tool_root: &Path) -> PathBuf {
    scoop_state_root_from(tool_root).join("buckets.json")
}

// Deprecated: use RuntimeLayout::from_root(root).scoop.apps_root.join("git").join("current").join("usr").join("bin") instead (per D-10, D-11)
#[deprecated]
pub fn scoop_git_usr_bin_from(tool_root: &Path) -> PathBuf {
    scoop_root_from(tool_root)
        .join("apps")
        .join("git")
        .join("current")
        .join("usr")
        .join("bin")
}

// Deprecated: use RuntimeLayout::from_root(root).msvc.managed.root instead (per D-10, D-11)
#[deprecated]
pub fn msvc_root_from(tool_root: &Path) -> PathBuf {
    msvc_domain_root_from(tool_root).join("managed")
}

pub fn native_msvc_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        "x86" => "x86",
        "aarch64" => "arm64",
        "arm" => "arm",
        _ => "x64",
    }
}

pub fn msvc_arch_config_value(global: &super::model::GlobalConfig) -> String {
    match global.msvc_arch.trim().to_ascii_lowercase().as_str() {
        "" | "auto" => "auto".to_string(),
        "x64" | "x86_64" => "x64".to_string(),
        "x86" | "i686" => "x86".to_string(),
        "arm64" | "aarch64" => "arm64".to_string(),
        "arm" => "arm".to_string(),
        _ => "auto".to_string(),
    }
}

pub fn msvc_arch_from_config(global: &super::model::GlobalConfig) -> String {
    match msvc_arch_config_value(global).as_str() {
        "auto" => native_msvc_arch().to_string(),
        explicit => explicit.to_string(),
    }
}

// Deprecated: use RuntimeLayout::from_root(root).msvc.official.instance_root instead (per D-10, D-11)
#[deprecated]
pub fn official_msvc_root_from(tool_root: &Path) -> PathBuf {
    msvc_domain_root_from(tool_root)
        .join("official")
        .join("instance")
}

// Deprecated: use RuntimeLayout::from_root(root).msvc.official.cache_root instead (per D-10, D-11)
#[deprecated]
pub fn official_msvc_cache_root_from(tool_root: &Path) -> PathBuf {
    msvc_domain_root_from(tool_root)
        .join("official")
        .join("cache")
}

// Deprecated: use RuntimeLayout::from_root(root).msvc.official.state_root instead (per D-10, D-11)
#[deprecated]
pub fn official_msvc_state_root_from(tool_root: &Path) -> PathBuf {
    msvc_domain_root_from(tool_root)
        .join("official")
        .join("state")
}

// Deprecated: use RuntimeLayout::from_root(root).msvc.managed.cache_root instead (per D-10, D-11)
#[deprecated]
pub fn msvc_cache_root_from(tool_root: &Path) -> PathBuf {
    msvc_root_from(tool_root).join("cache")
}

// Deprecated: use RuntimeLayout::from_root(root).msvc.managed.state_root instead (per D-10, D-11)
#[deprecated]
pub fn msvc_state_root_from(tool_root: &Path) -> PathBuf {
    msvc_root_from(tool_root).join("state")
}

// Deprecated: use RuntimeLayout::from_root(root).msvc.managed.toolchain_root instead (per D-10, D-11)
#[deprecated]
pub fn msvc_toolchain_root_from(tool_root: &Path) -> PathBuf {
    msvc_root_from(tool_root).join("toolchain")
}

// Deprecated: use RuntimeLayout::from_root(root).msvc.managed.manifest_root instead (per D-10, D-11)
#[deprecated]
pub fn msvc_manifest_root_from(tool_root: &Path) -> PathBuf {
    msvc_cache_root_from(tool_root).join("manifest")
}
