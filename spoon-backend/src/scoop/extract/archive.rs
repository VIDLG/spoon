use std::path::{Path, PathBuf};
use std::process::Command;

use strum_macros::AsRefStr;

use crate::archive::extract_zip_archive_sync;
use crate::layout::RuntimeLayout;
use crate::platform::msiexec_path;
use crate::{BackendError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
pub enum ArchiveKind {
    Zip,
    Msi,
    #[strum(serialize = "7z")]
    SevenZip,
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
    let layout = RuntimeLayout::from_root(tool_root);
    vec![
        layout
            .scoop
            .apps_root
            .join("7zip")
            .join("current")
            .join("7z.exe"),
        layout.shims.join("7z.cmd"),
    ]
}
