use anyhow::{Context, Result};
use std::collections::HashSet;

#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use winreg::enums::*;

use super::state::test_mode_enabled;

pub fn normalize_scoop_proxy_value(proxy: &str) -> String {
    match spoon_core::normalize_proxy_url(proxy) {
        Ok(Some(normalized)) => normalized
            .strip_prefix("http://")
            .or_else(|| normalized.strip_prefix("https://"))
            .unwrap_or(&normalized)
            .to_string(),
        _ => String::new(),
    }
}

pub fn proxy_env_pairs(proxy: &str) -> Vec<(&'static str, String)> {
    match spoon_core::normalize_proxy_url(proxy) {
        Ok(Some(proxy)) => vec![
            ("HTTP_PROXY", proxy.clone()),
            ("HTTPS_PROXY", proxy.clone()),
            ("ALL_PROXY", proxy),
        ],
        _ => Vec::new(),
    }
}

pub fn apply_scoop_env(scoop_root: &std::path::Path) -> Result<()> {
    set_user_env_var("SCOOP", &scoop_root.display().to_string())
}

pub fn clear_scoop_env() -> Result<()> {
    set_user_env_var("SCOOP", "")
}

pub fn ensure_user_path_entry(path: &std::path::Path) -> Result<()> {
    let target = path.display().to_string();
    let mut items = user_env_path_entries()?;
    if !items.iter().any(|item| item.eq_ignore_ascii_case(&target)) {
        items.push(target.clone());
    }
    set_user_env_var("Path", &items.join(";"))
        .with_context(|| format!("failed to append user PATH entry {target}"))
}

pub fn remove_user_path_entry(path: &std::path::Path) -> Result<()> {
    let target = path.display().to_string();
    let items = user_env_path_entries()?
        .into_iter()
        .filter(|item| !item.eq_ignore_ascii_case(&target))
        .collect::<Vec<_>>();
    set_user_env_var("Path", &items.join(";"))
        .with_context(|| format!("failed to remove user PATH entry {target}"))
}

pub fn apply_process_scoop_env(scoop_root: &std::path::Path) {
    unsafe {
        std::env::set_var("SCOOP", scoop_root.display().to_string());
    }
}

pub fn clear_process_scoop_env() {
    unsafe {
        std::env::remove_var("SCOOP");
    }
}

pub fn ensure_process_path_entry(path: &std::path::Path) {
    let target = path.display().to_string();
    let current = std::env::var("PATH").unwrap_or_default();
    let merged = merge_path_entries(&current, &target);
    unsafe {
        std::env::set_var("PATH", merged);
    }
}

pub fn remove_process_path_entry(path: &std::path::Path) {
    let target = path.display().to_string();
    let updated = std::env::var("PATH")
        .unwrap_or_default()
        .split(';')
        .map(str::trim)
        .filter(|item| !item.is_empty() && !item.eq_ignore_ascii_case(&target))
        .collect::<Vec<_>>()
        .join(";");
    unsafe {
        std::env::set_var("PATH", updated);
    }
}

fn merge_path_entries(current: &str, extra: &str) -> String {
    let mut seen = HashSet::new();
    let mut merged = Vec::new();

    for entry in current.split(';').chain(extra.split(';')) {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_ascii_lowercase();
        if seen.insert(key) {
            merged.push(trimmed.to_string());
        }
    }

    merged.join(";")
}

fn set_user_env_var(name: &str, value: &str) -> Result<()> {
    if test_mode_enabled() || std::env::var_os("SPOON_TEST_HOME").is_some() {
        let _ = (name, value);
        return Ok(());
    }
    set_hkcu_environment_var(name, value)
}

#[cfg(windows)]
fn user_env_path_entries() -> Result<Vec<String>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ)
        .context("failed to open HKCU\\Environment")?;
    let current: String = env.get_value("Path").unwrap_or_default();
    Ok(current
        .split(';')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect())
}

#[cfg(not(windows))]
fn user_env_path_entries() -> Result<Vec<String>> {
    Ok(Vec::new())
}

#[cfg(windows)]
fn set_hkcu_environment_var(name: &str, value: &str) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (env, _) = hkcu
        .create_subkey("Environment")
        .context("failed to open HKCU\\Environment")?;
    if value.trim().is_empty() {
        match env.delete_value(name) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err)
                .with_context(|| format!("failed to delete user environment variable {name}")),
        }
    } else {
        env.set_value(name, &value)
            .with_context(|| format!("failed to set user environment variable {name}"))
    }
}

#[cfg(not(windows))]
fn set_hkcu_environment_var(name: &str, _value: &str) -> Result<()> {
    anyhow::bail!("setting user environment variable {name} is only supported on Windows")
}

#[cfg(test)]
mod tests {
    use super::{normalize_scoop_proxy_value, proxy_env_pairs};

    #[test]
    fn proxy_env_pairs_add_scheme_for_bare_host_port() {
        let pairs = proxy_env_pairs("127.0.0.1:7897");
        assert_eq!(
            pairs[0],
            ("HTTP_PROXY", "http://127.0.0.1:7897".to_string())
        );
        assert_eq!(
            pairs[1],
            ("HTTPS_PROXY", "http://127.0.0.1:7897".to_string())
        );
        assert_eq!(pairs[2], ("ALL_PROXY", "http://127.0.0.1:7897".to_string()));
    }

    #[test]
    fn normalize_scoop_proxy_value_strips_scheme() {
        assert_eq!(
            normalize_scoop_proxy_value("http://127.0.0.1:7897"),
            "127.0.0.1:7897"
        );
        assert_eq!(
            normalize_scoop_proxy_value("https://127.0.0.1:7897/"),
            "127.0.0.1:7897"
        );
    }
}
