use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use tokio::fs;
use zip::ZipArchive;

use crate::fsx;
use crate::{BackendError, BackendEvent, Result};
use crate::platform::msiexec_path;

use super::paths;
use super::runtime::SelectedPackageSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveKind {
    Zip,
    Msi,
    SevenZip,
}

impl ArchiveKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Zip => "zip",
            Self::Msi => "msi",
            Self::SevenZip => "7z",
        }
    }
}

pub fn detect_archive_kind(path: &Path) -> Option<ArchiveKind> {
    let ext = path.extension().and_then(|value| value.to_str())?;
    if ext.eq_ignore_ascii_case("zip") {
        Some(ArchiveKind::Zip)
    } else if ext.eq_ignore_ascii_case("msi") {
        Some(ArchiveKind::Msi)
    } else if ext.eq_ignore_ascii_case("7z") {
        Some(ArchiveKind::SevenZip)
    } else {
        None
    }
}

pub fn extract_archive_sync(
    tool_root: &Path,
    archive_path: &Path,
    destination: &Path,
) -> Result<ArchiveKind> {
    let kind = detect_archive_kind(archive_path).ok_or_else(|| {
        BackendError::UnsupportedArchiveKind {
            path: archive_path.to_path_buf(),
        }
    })?;
    match kind {
        ArchiveKind::Zip => extract_zip_archive_sync(archive_path, destination)?,
        ArchiveKind::Msi => extract_msi_archive_sync(archive_path, destination)?,
        ArchiveKind::SevenZip => {
            extract_7z_archive_with_helper_sync(tool_root, archive_path, destination)?
        }
    }
    Ok(kind)
}

fn extract_zip_archive_sync(archive_path: &Path, destination: &Path) -> Result<()> {
    let file =
        File::open(archive_path).map_err(|err| BackendError::fs("open", archive_path, err))?;
    let mut archive = ZipArchive::new(file).map_err(|err| {
        BackendError::external(format!("invalid zip {}", archive_path.display()), err)
    })?;
    std::fs::create_dir_all(destination)
        .map_err(|err| BackendError::fs("create", destination, err))?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|err| BackendError::external("invalid zip entry", err))?;
        let Some(name) = entry.enclosed_name().map(PathBuf::from) else {
            continue;
        };
        let output_path = destination.join(name);
        if entry.is_dir() {
            std::fs::create_dir_all(&output_path)
                .map_err(|err| BackendError::fs("create", &output_path, err))?;
            continue;
        }
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| BackendError::fs("create", parent, err))?;
        }
        let mut output = File::create(&output_path)
            .map_err(|err| BackendError::fs("create", &output_path, err))?;
        io::copy(&mut entry, &mut output).map_err(|err| {
            BackendError::Other(format!(
                "failed to extract {}: {err}",
                output_path.display()
            ))
        })?;
    }
    Ok(())
}

fn extract_msi_archive_sync(archive_path: &Path, destination: &Path) -> Result<()> {
    std::fs::create_dir_all(destination)
        .map_err(|err| BackendError::fs("create", destination, err))?;
    let source_dir = destination.join("SourceDir");
    if source_dir.exists() {
        std::fs::remove_dir_all(&source_dir)
            .map_err(|err| BackendError::fs("remove", &source_dir, err))?;
    }
    let target_dir = source_dir
        .canonicalize()
        .unwrap_or_else(|_| source_dir.to_path_buf());
    let status = Command::new(msiexec_path())
        .arg("/a")
        .arg(archive_path)
        .arg("/qn")
        .arg(format!("TARGETDIR={}", target_dir.display()))
        .status()
        .map_err(|err| {
            BackendError::external(
                format!("failed to launch msiexec for {}", archive_path.display()),
                err,
            )
        })?;
    if !status.success() {
        return Err(BackendError::Other(format!(
            "failed to extract MSI archive {} with msiexec (status {})",
            archive_path.display(),
            status
        )));
    }
    if !source_dir.exists() {
        return Ok(());
    }
    for entry in
        std::fs::read_dir(&source_dir).map_err(|err| BackendError::fs("read", &source_dir, err))?
    {
        let entry = entry.map_err(|err| BackendError::fs("read", &source_dir, err))?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if to.exists() {
            if to.is_dir() {
                std::fs::remove_dir_all(&to).map_err(|err| BackendError::fs("remove", &to, err))?;
            } else {
                std::fs::remove_file(&to).map_err(|err| BackendError::fs("remove", &to, err))?;
            }
        }
        std::fs::rename(&from, &to).map_err(|err| {
            BackendError::Other(format!(
                "failed to move {} to {}: {err}",
                from.display(),
                to.display()
            ))
        })?;
    }
    std::fs::remove_dir_all(&source_dir)
        .map_err(|err| BackendError::fs("remove", &source_dir, err))?;
    Ok(())
}

fn extract_7z_archive_with_helper_sync(
    tool_root: &Path,
    archive_path: &Path,
    destination: &Path,
) -> Result<()> {
    std::fs::create_dir_all(destination)
        .map_err(|err| BackendError::fs("create", destination, err))?;
    let helper = helper_7z_candidates(tool_root)
        .into_iter()
        .find(|candidate| candidate.exists())
        .ok_or_else(|| {
            BackendError::Other(format!(
                "7z archive {} requires managed 7zip helper, but no helper executable was found",
                archive_path.display()
            ))
        })?;
    let output = Command::new(&helper)
        .arg("x")
        .arg("-y")
        .arg(format!("-o{}", destination.display()))
        .arg(archive_path)
        .output()
        .map_err(|err| {
            BackendError::external(
                format!(
                    "failed to launch {} for {}",
                    helper.display(),
                    archive_path.display()
                ),
                err,
            )
        })?;
    if output.status.success() {
        return Ok(());
    }
    Err(BackendError::Other(format!(
        "failed to extract 7z archive {} with helper {} (status {:?})\nstdout:\n{}\nstderr:\n{}",
        archive_path.display(),
        helper.display(),
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )))
}

pub fn helper_7z_candidates(tool_root: &Path) -> Vec<PathBuf> {
    vec![
        paths::scoop_root(tool_root)
            .join("apps")
            .join("7zip")
            .join("current")
            .join("7z.exe"),
        paths::shims_root(tool_root).join("7z.cmd"),
    ]
}

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

pub async fn materialize_installer_payloads_to_root(
    archive_paths: &[PathBuf],
    source: &SelectedPackageSource,
    install_root: &Path,
    _emit: &mut dyn FnMut(BackendEvent),
) -> Result<()> {
    if install_root.exists() {
        remove_path_if_exists(install_root).await?;
    }
    fs::create_dir_all(install_root).await.map_err(|err| {
        BackendError::Other(format!(
            "failed to create {}: {err}",
            install_root.display()
        ))
    })?;
    for (index, archive_path) in archive_paths.iter().enumerate() {
        let destination = if let Some(relative_path) = source
            .payloads
            .get(index)
            .and_then(|payload| payload.target_name.as_deref())
        {
            install_root.join(relative_path)
        } else {
            let file_name = archive_path
                .file_name()
                .map(|value| value.to_os_string())
                .ok_or_else(|| {
                    BackendError::Other(
                        "installer payload is missing a target filename".to_string(),
                    )
                })?;
            install_root.join(file_name)
        };
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|err| BackendError::fs("create", parent, err))?;
        }
        fs::copy(archive_path, &destination).await.map_err(|err| {
            BackendError::Other(format!(
                "failed to copy into {}: {err}",
                destination.display()
            ))
        })?;
        tracing::info!(
            "Copied installer payload {} into {}",
            index + 1,
            destination.display()
        );
    }
    Ok(())
}

pub async fn extract_archive_to_root(
    tool_root: &Path,
    archive_paths: &[PathBuf],
    source: &SelectedPackageSource,
    install_root: &Path,
    _emit: &mut dyn FnMut(BackendEvent),
) -> Result<()> {
    if install_root.exists() {
        remove_path_if_exists(install_root).await?;
    }
    fs::create_dir_all(install_root).await.map_err(|err| {
        BackendError::Other(format!(
            "failed to create {}: {err}",
            install_root.display()
        ))
    })?;
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
                    "Extracted {} package payload {} into {}",
                    kind.label(),
                    index + 1,
                    install_root.display(),
                );
            } else {
                let staging_root = install_root.with_extension(format!("extract-staging-{index}"));
                if staging_root.exists() {
                    fs::remove_dir_all(&staging_root).await.map_err(|err| {
                        BackendError::Other(format!(
                            "failed to remove {}: {err}",
                            staging_root.display()
                        ))
                    })?;
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
                    fs::remove_dir_all(&staging_root).await.map_err(|err| {
                        BackendError::Other(format!(
                            "failed to remove {}: {err}",
                            staging_root.display()
                        ))
                    })?;
                }
                tracing::info!(
                    "Extracted {} package payload {} into {}",
                    kind.label(),
                    index + 1,
                    install_root.display(),
                );
            }
        } else {
            let destination = if let Some(relative_path) = source
                .payloads
                .get(index)
                .and_then(|payload| payload.target_name.as_deref())
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
            fs::copy(archive_path, &destination).await.map_err(|err| {
                BackendError::Other(format!(
                    "failed to copy into {}: {err}",
                    destination.display()
                ))
            })?;
            tracing::info!(
                "Copied package payload {} into {}",
                index + 1,
                destination.display()
            );
        }
    }
    Ok(())
}

pub async fn copy_path_recursive(source: &Path, target: &Path) -> Result<()> {
    fsx::copy_path_recursive(source, target, None).await
}
