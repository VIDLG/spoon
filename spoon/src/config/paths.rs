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
    let path = spoon_core::RuntimeLayout::from_root(Path::new(trimmed)).msvc.managed.root;
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

pub fn native_msvc_arch() -> &'static str {
    spoon_msvc::paths::native_msvc_arch()
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
