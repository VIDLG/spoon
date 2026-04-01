use std::fs;
use std::path::Path;

use tokio::fs as tokio_fs;

use crate::fsx::directory_size;
use crate::layout::RuntimeLayout;
use crate::{BackendError, Result};

fn recreate_empty_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(BackendError::Io)?;
    }
    fs::create_dir_all(path).map_err(BackendError::Io)?;
    Ok(())
}

pub fn prune(cache_root: &Path) -> Result<Vec<String>> {
    recreate_empty_dir(cache_root)?;
    Ok(vec![format!(
        "Pruned Scoop download cache under {}.",
        cache_root.display()
    )])
}

pub fn clear(cache_root: &Path) -> Result<Vec<String>> {
    recreate_empty_dir(cache_root)?;
    Ok(vec![
        format!("Cleared Scoop cache under {}.", cache_root.display()),
        "Retained Scoop buckets, apps, persist data, and state.".to_string(),
    ])
}

pub async fn package_cache_size(tool_root: &Path, package_name: &str) -> Result<u64> {
    let cache_root = RuntimeLayout::from_root(tool_root).scoop.cache_root;
    let mut entries = match tokio_fs::read_dir(&cache_root).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(err) => return Err(BackendError::fs("read", &cache_root, err)),
    };
    let mut total = 0;
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|err| BackendError::fs("read_entry", &cache_root, err))?
    {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with(package_name) {
            total += directory_size(&path).await?;
        }
    }
    Ok(total)
}
