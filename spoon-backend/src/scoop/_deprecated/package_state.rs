use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{BackendError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoopPackageState {
    pub name: String,
    pub version: String,
    pub bucket: String,
    pub architecture: Option<String>,
}

impl ScoopPackageState {
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(BackendError::ManifestParse)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(BackendError::ManifestParse)
    }
}

pub fn package_manifest_state_path(tool_root: &Path, package_name: &str) -> PathBuf {
    tool_root
        .join("scoop")
        .join("state")
        .join(format!("{}.json", package_name))
}

pub async fn read_package_state(
    tool_root: &Path,
    package_name: &str,
) -> Result<Option<ScoopPackageState>> {
    let path = package_manifest_state_path(tool_root, package_name);
    if !tokio::fs::try_exists(&path)
        .await
        .map_err(BackendError::Io)?
    {
        return Ok(None);
    }

    let content = tokio::fs::read_to_string(&path)
        .await
        .map_err(BackendError::Io)?;
    let state = ScoopPackageState::from_json(&content)?;
    Ok(Some(state))
}

pub async fn write_package_state(
    tool_root: &Path,
    package_name: &str,
    state: &ScoopPackageState,
) -> Result<()> {
    let path = package_manifest_state_path(tool_root, package_name);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(BackendError::Io)?;
    }

    let json = state.to_json()?;
    tokio::fs::write(&path, json)
        .await
        .map_err(BackendError::Io)?;
    Ok(())
}

pub async fn remove_package_state(tool_root: &Path, package_name: &str) -> Result<()> {
    let path = package_manifest_state_path(tool_root, package_name);
    if tokio::fs::try_exists(&path)
        .await
        .map_err(BackendError::Io)?
    {
        tokio::fs::remove_file(&path)
            .await
            .map_err(BackendError::Io)?;
    }
    Ok(())
}
