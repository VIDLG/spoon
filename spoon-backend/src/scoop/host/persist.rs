use std::path::Path;

use tokio::fs;

use crate::Result;
use crate::BackendError;
use crate::scoop::PersistEntry;

use super::super::extract::{copy_path_recursive, remove_path_if_exists};

async fn ensure_persist_root(persist_root: &Path) -> Result<()> {
    fs::create_dir_all(persist_root)
        .await
        .map_err(|err| BackendError::fs("create", persist_root, err))
}

pub async fn sync_persist_entries_from_root(
    install_root: &Path,
    persist_root: &Path,
    entries: &[PersistEntry],
) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    ensure_persist_root(persist_root).await?;
    for entry in entries {
        let current_path = install_root.join(&entry.relative_path);
        if !current_path.exists() {
            continue;
        }
        let persist_path = persist_root.join(&entry.store_name);
        remove_path_if_exists(&persist_path).await?;
        copy_path_recursive(&current_path, &persist_path).await?;
        tracing::info!(
            "Synced persisted path '{}' into {}",
            entry.relative_path,
            persist_path.display()
        );
    }
    Ok(())
}

pub async fn restore_persist_entries_into_root(
    install_root: &Path,
    persist_root: &Path,
    entries: &[PersistEntry],
) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    ensure_persist_root(persist_root).await?;
    for entry in entries {
        let persist_path = persist_root.join(&entry.store_name);
        let current_path = install_root.join(&entry.relative_path);
        if persist_path.exists() {
            remove_path_if_exists(&current_path).await?;
            copy_path_recursive(&persist_path, &current_path).await?;
            tracing::info!(
                "Restored persisted path '{}' from {}",
                entry.relative_path,
                persist_path.display()
            );
        } else if current_path.exists() {
            copy_path_recursive(&current_path, &persist_path).await?;
            tracing::info!(
                "Seeded persisted path '{}' into {}",
                entry.relative_path,
                persist_path.display()
            );
        }
    }
    Ok(())
}
