use std::path::Path;

use async_recursion::async_recursion;
use tokio::fs;

use crate::{CoreError, CancellationToken, Result, check_token_cancel};

/// Recursively copy a directory or file from `source` to `target`.
#[async_recursion(?Send)]
pub async fn copy_path_recursive(
    source: &Path,
    target: &Path,
    cancel: Option<&CancellationToken>,
) -> Result<()> {
    check_token_cancel(cancel)?;

    let metadata = fs::symlink_metadata(source)
        .await
        .map_err(|err| CoreError::fs("metadata", source, err))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }

    if metadata.is_dir() {
        fs::create_dir_all(target)
            .await
            .map_err(|err| CoreError::fs("create", target, err))?;

        let mut entries = fs::read_dir(source)
            .await
            .map_err(|err| CoreError::fs("read", source, err))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|err| CoreError::fs("read_entry", source, err))?
        {
            check_token_cancel(cancel)?;

            let entry_path = entry.path();
            let target_path = target.join(entry.file_name());
            copy_path_recursive(&entry_path, &target_path, cancel).await?;
        }
    } else if metadata.is_file() {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|err| CoreError::fs("create", parent, err))?;
        }
        fs::copy(source, target)
            .await
            .map_err(|err| CoreError::fs("copy", target, err))?;
    }

    Ok(())
}

#[async_recursion(?Send)]
pub async fn directory_size(path: &Path) -> Result<u64> {
    let metadata = fs::symlink_metadata(path)
        .await
        .map_err(|err| CoreError::fs("metadata", path, err))?;
    if metadata.file_type().is_symlink() {
        return Ok(0);
    }
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let mut entries = fs::read_dir(path)
        .await
        .map_err(|err| CoreError::fs("read", path, err))?;
    let mut total = 0;
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|err| CoreError::fs("read_entry", path, err))?
    {
        total += directory_size(&entry.path()).await?;
    }
    Ok(total)
}
