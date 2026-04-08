use std::collections::HashSet;
use std::env;

use anyhow::{Context, Result};
#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use winreg::enums::*;

pub fn refresh_process_env_from_registry() -> Result<()> {
    let registry_path = load_registry_path().context("failed to refresh PATH from registry")?;
    if !registry_path.is_empty() {
        let current_path = env::var("PATH").unwrap_or_default();
        let path = merge_path_entries(&current_path, &registry_path);
        unsafe {
            env::set_var("PATH", path);
        }
    }
    Ok(())
}

#[cfg(windows)]
fn load_registry_path() -> Result<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let user_path: String = hkcu
        .open_subkey_with_flags("Environment", KEY_READ)
        .ok()
        .and_then(|key| key.get_value("Path").ok())
        .unwrap_or_default();
    let machine_path: String = hklm
        .open_subkey_with_flags(
            r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
            KEY_READ,
        )
        .ok()
        .and_then(|key| key.get_value("Path").ok())
        .unwrap_or_default();
    Ok(match (user_path.trim(), machine_path.trim()) {
        ("", "") => String::new(),
        ("", machine) => machine.to_string(),
        (user, "") => user.to_string(),
        (user, machine) => format!("{user};{machine}"),
    })
}

#[cfg(not(windows))]
fn load_registry_path() -> Result<String> {
    Ok(String::new())
}

fn merge_path_entries(current: &str, registry: &str) -> String {
    let mut seen = HashSet::new();
    let mut merged = Vec::new();
    for entry in current.split(';').chain(registry.split(';')) {
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
