use std::path::{Path, PathBuf};

use tokio::fs;

use crate::fsx;
use crate::{BackendError, BackendEvent, Result};

use super::archive::{detect_archive_kind, extract_archive_sync};
use super::current::remove_path_if_exists;
use crate::scoop::package_source::ResolvedPackageSource;

pub async fn materialize_installer_assets_to_root(
    archive_paths: &[PathBuf],
    source: &ResolvedPackageSource,
    install_root: &Path,
    _emit: &mut dyn FnMut(BackendEvent),
) -> Result<()> {
    if install_root.exists() {
        remove_path_if_exists(install_root).await?;
    }
    fs::create_dir_all(install_root)
        .await
        .map_err(|err| BackendError::fs("create", install_root, err))?;
    for (index, archive_path) in archive_paths.iter().enumerate() {
        let destination = if let Some(relative_path) = source
            .assets
            .get(index)
            .and_then(|asset| asset.target_name.as_deref())
        {
            install_root.join(relative_path)
        } else {
            let file_name = archive_path
                .file_name()
                .map(|value| value.to_os_string())
                .ok_or_else(|| {
                    BackendError::Other(
                        "installer asset is missing a target filename".to_string(),
                    )
                })?;
            install_root.join(file_name)
        };
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|err| BackendError::fs("create", parent, err))?;
        }
        fs::copy(archive_path, &destination)
            .await
            .map_err(|err| BackendError::fs("copy", &destination, err))?;
        tracing::info!(
            "Copied installer asset {} into {}",
            index + 1,
            destination.display()
        );
    }
    Ok(())
}

pub async fn extract_archive_to_root(
    tool_root: &Path,
    archive_paths: &[PathBuf],
    source: &ResolvedPackageSource,
    install_root: &Path,
    _emit: &mut dyn FnMut(BackendEvent),
) -> Result<()> {
    if install_root.exists() {
        remove_path_if_exists(install_root).await?;
    }
    fs::create_dir_all(install_root)
        .await
        .map_err(|err| BackendError::fs("create", install_root, err))?;
    for (index, archive_path) in archive_paths.iter().enumerate() {
        if detect_archive_kind(archive_path).is_some() {
            if source.extract_dir.is_empty() {
                let destination = source
                    .extract_to
                    .get(index)
                    .or_else(|| source.extract_to.first())
                    .map(|value| install_root.join(value))
                    .unwrap_or_else(|| install_root.to_path_buf());
                let archive = archive_path.to_path_buf();
                let tool_root = tool_root.to_path_buf();
                let kind = tokio::task::spawn_blocking(move || {
                    extract_archive_sync(&tool_root, &archive, &destination)
                })
                .await
                .map_err(|err| BackendError::external("archive extraction join failed", err))??;
                tracing::info!(
                    "Extracted {} package asset {} into {}",
                    kind.as_ref(),
                    index + 1,
                    install_root.display(),
                );
            } else {
                let staging_root = install_root.with_extension(format!("extract-staging-{index}"));
                if staging_root.exists() {
                    fs::remove_dir_all(&staging_root)
                        .await
                        .map_err(|err| BackendError::fs("remove", &staging_root, err))?;
                }
                let archive = archive_path.to_path_buf();
                let staging = staging_root.clone();
                let tool_root = tool_root.to_path_buf();
                let kind = tokio::task::spawn_blocking(move || {
                    extract_archive_sync(&tool_root, &archive, &staging)
                })
                .await
                .map_err(|err| BackendError::external("archive extraction join failed", err))??;
                for (mapping_index, extract_dir) in source.extract_dir.iter().enumerate() {
                    let extract_dir = extract_dir.trim();
                    if extract_dir.is_empty() {
                        continue;
                    }
                    let extract_to = source
                        .extract_to
                        .get(mapping_index)
                        .map(String::as_str)
                        .unwrap_or("");
                    let source_path = staging_root.join(extract_dir);
                    let target_root = if extract_to.trim().is_empty() {
                        install_root.to_path_buf()
                    } else {
                        install_root.join(extract_to)
                    };
                    if source_path.is_dir() {
                        let mut entries = fs::read_dir(&source_path)
                            .await
                            .map_err(|err| BackendError::fs("read", &source_path, err))?;
                        while let Some(entry) = entries
                            .next_entry()
                            .await
                            .map_err(|err| BackendError::fs("read_entry", &source_path, err))?
                        {
                            fsx::copy_path_recursive(
                                &entry.path(),
                                &target_root.join(entry.file_name()),
                                None,
                            )
                            .await?;
                        }
                    } else {
                        let file_name = source_path.file_name().ok_or_else(|| {
                            BackendError::Other("missing extracted file name".to_string())
                        })?;
                        fsx::copy_path_recursive(&source_path, &target_root.join(file_name), None)
                            .await?;
                    }
                }
                if staging_root.exists() {
                    fs::remove_dir_all(&staging_root)
                        .await
                        .map_err(|err| BackendError::fs("remove", &staging_root, err))?;
                }
                tracing::info!(
                    "Extracted {} package asset {} into {}",
                    kind.as_ref(),
                    index + 1,
                    install_root.display(),
                );
            }
        } else {
            let destination = if let Some(relative_path) = source
                .assets
                .get(index)
                .and_then(|asset| asset.target_name.as_deref())
            {
                install_root.join(relative_path)
            } else {
                let file_name = source
                    .bins
                    .get(index)
                    .or_else(|| source.bins.first())
                    .map(|target| Path::new(&target.relative_path))
                    .and_then(|path| path.file_name())
                    .map(|value| value.to_os_string())
                    .or_else(|| archive_path.file_name().map(|value| value.to_os_string()))
                    .ok_or_else(|| {
                        BackendError::Other(
                            "single-file package is missing a target filename".to_string(),
                        )
                    })?;
                install_root.join(file_name)
            };
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|err| BackendError::fs("create", parent, err))?;
            }
            fs::copy(archive_path, &destination)
                .await
                .map_err(|err| BackendError::fs("copy", &destination, err))?;
            tracing::info!(
                "Copied package asset {} into {}",
                index + 1,
                destination.display()
            );
        }
    }
    Ok(())
}
