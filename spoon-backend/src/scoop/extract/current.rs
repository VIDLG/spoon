use std::path::Path;

use tokio::fs;

use crate::fsx;
use crate::{BackendError, BackendEvent, Result};

pub async fn remove_path_if_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(path)
        .await
        .map_err(|err| BackendError::fs("stat", path, err))?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        if let Err(dir_err) = fs::remove_dir(path).await {
            fs::remove_file(path).await.map_err(|err| {
                BackendError::Other(format!(
                    "failed to remove {} after {dir_err}: {err}",
                    path.display()
                ))
            })?;
        }
    } else if metadata.is_dir() {
        fs::remove_dir_all(path)
            .await
            .map_err(|err| BackendError::fs("remove", path, err))?;
    } else {
        fs::remove_file(path)
            .await
            .map_err(|err| BackendError::fs("remove", path, err))?;
    }
    Ok(())
}

fn create_current_symlink(version_root: &Path, current_root: &Path) -> bool {
    #[cfg(windows)]
    {
        if std::os::windows::fs::symlink_dir(version_root, current_root).is_ok() {
            return true;
        }
    }
    false
}

pub async fn refresh_current_entry(
    version_root: &Path,
    current_root: &Path,
    _emit: &mut dyn FnMut(BackendEvent),
) -> Result<()> {
    remove_path_if_exists(current_root).await?;
    let mode = if create_current_symlink(version_root, current_root) {
        "symlink"
    } else {
        fsx::copy_path_recursive(version_root, current_root, None).await?;
        "copy"
    };
    tracing::info!(
        "Refreshed current entry using {mode}: {} -> {}",
        current_root.display(),
        version_root.display()
    );
    Ok(())
}

pub async fn copy_path_recursive(source: &Path, target: &Path) -> Result<()> {
    fsx::copy_path_recursive(source, target, None).await
}
