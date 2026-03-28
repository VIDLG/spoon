use tokio::fs;

use super::model::InstalledPackageState;
use crate::layout::RuntimeLayout;
use crate::{BackendError, Result};

/// Canonical state persistence directory for a given tool root.
fn state_root(layout: &RuntimeLayout) -> &std::path::Path {
    &layout.scoop.package_state_root
}

/// Canonical state file path for a specific package.
fn state_file(layout: &RuntimeLayout, package_name: &str) -> std::path::PathBuf {
    state_root(layout).join(format!("{package_name}.json"))
}

/// Read a single canonical installed-package state.
///
/// Returns `None` if the state file does not exist or cannot be parsed.
pub async fn read_installed_state(
    layout: &RuntimeLayout,
    package_name: &str,
) -> Option<InstalledPackageState> {
    let path = state_file(layout, package_name);
    let content = fs::read_to_string(path).await.ok()?;
    serde_json::from_str(&content).ok()
}

/// Write (create or update) a canonical installed-package state.
pub async fn write_installed_state(
    layout: &RuntimeLayout,
    state: &InstalledPackageState,
) -> Result<()> {
    let root = state_root(layout);
    fs::create_dir_all(root)
        .await
        .map_err(|err| BackendError::fs("create", root, err))?;
    let path = state_file(layout, &state.package);
    let content = serde_json::to_string_pretty(state)
        .map_err(|err| BackendError::external("failed to serialize installed state", err))?;
    fs::write(&path, content)
        .await
        .map_err(|err| BackendError::fs("write", &path, err))?;
    Ok(())
}

/// Remove a canonical installed-package state file.
///
/// Silently succeeds if the file does not exist.
pub async fn remove_installed_state(
    layout: &RuntimeLayout,
    package_name: &str,
) -> Result<()> {
    let path = state_file(layout, package_name);
    if path.exists() {
        fs::remove_file(&path)
            .await
            .map_err(|err| BackendError::fs("remove", &path, err))?;
    }
    Ok(())
}

/// Enumerate all canonical installed-package states.
///
/// Reads every `*.json` file under the package state root and returns those
/// that successfully deserialize as [`InstalledPackageState`].
pub async fn list_installed_states(layout: &RuntimeLayout) -> Vec<InstalledPackageState> {
    let root = state_root(layout);
    let mut states = Vec::new();

    if !root.exists() {
        return states;
    }

    let mut entries = match fs::read_dir(root).await {
        Ok(entries) => entries,
        Err(_) => return states,
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let content = match fs::read_to_string(&path).await {
            Ok(content) => content,
            Err(_) => continue,
        };
        if let Ok(state) = serde_json::from_str::<InstalledPackageState>(&content) {
            states.push(state);
        }
    }

    states
}
