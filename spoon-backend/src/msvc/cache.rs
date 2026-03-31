use std::fs;
use std::path::Path;

use crate::{BackendError, Result};

fn remove_if_exists(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(path).map_err(BackendError::Io)?;
    Ok(true)
}

pub fn prune(cache_root: &Path) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    for dir in [cache_root.join("validate"), cache_root.join("metadata")] {
        if remove_if_exists(&dir)? {
            lines.push(format!("Pruned MSVC cache directory {}.", dir.display()));
        }
    }
    if lines.is_empty() {
        lines.push("No pruneable MSVC cache directories were present.".to_string());
    }
    Ok(lines)
}

pub fn clear(cache_root: &Path) -> Result<Vec<String>> {
    if cache_root.exists() {
        fs::remove_dir_all(cache_root).map_err(BackendError::Io)?;
    }
    fs::create_dir_all(cache_root).map_err(BackendError::Io)?;
    Ok(vec![
        format!("Cleared MSVC cache under {}.", cache_root.display()),
        "Retained managed MSVC toolchain and state.".to_string(),
    ])
}
