use std::fs;
use std::path::Path;

use spoon_core::CoreError;

fn remove_if_exists(path: &Path) -> Result<bool, CoreError> {
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(path)
        .map_err(|e| CoreError::fs("remove_dir_all", path, e))?;
    Ok(true)
}

pub fn prune(cache_root: &Path) -> Result<Vec<String>, CoreError> {
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

pub fn clear(cache_root: &Path) -> Result<Vec<String>, CoreError> {
    if cache_root.exists() {
        fs::remove_dir_all(cache_root)
            .map_err(|e| CoreError::fs("remove_dir_all", cache_root, e))?;
    }
    fs::create_dir_all(cache_root)
        .map_err(|e| CoreError::fs("create_dir_all", cache_root, e))?;
    Ok(vec![
        format!("Cleared MSVC cache under {}.", cache_root.display()),
        "Retained managed MSVC toolchain and state.".to_string(),
    ])
}
