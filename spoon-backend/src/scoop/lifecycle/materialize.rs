use std::path::{Path, PathBuf};

use crate::{BackendEvent, Result};

use super::super::extract::{extract_archive_to_root, materialize_installer_payloads_to_root};
use super::super::package_source::SelectedPackageSource;

pub(crate) async fn materialize_payloads(
    tool_root: &Path,
    archive_paths: &[PathBuf],
    source: &SelectedPackageSource,
    version_root: &Path,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Option<PathBuf>> {
    let primary_archive = archive_paths.first().cloned();
    if source.installer_script.is_empty() {
        extract_archive_to_root(tool_root, archive_paths, source, version_root, emit).await?;
    } else {
        materialize_installer_payloads_to_root(archive_paths, source, version_root, emit).await?;
    }
    Ok(primary_archive)
}
