use std::path::{Path, PathBuf};

use crate::{BackendEvent, Result};

use super::super::extract::{extract_archive_to_root, materialize_installer_assets_to_root};
use super::super::package_source::ResolvedPackageSource;

pub(crate) async fn materialize_assets(
    tool_root: &Path,
    asset_paths: &[PathBuf],
    source: &ResolvedPackageSource,
    version_root: &Path,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Option<PathBuf>> {
    let primary_archive = asset_paths.first().cloned();
    if source.installer_script.is_empty() {
        extract_archive_to_root(tool_root, asset_paths, source, version_root, emit).await?;
    } else {
        materialize_installer_assets_to_root(asset_paths, source, version_root, emit).await?;
    }
    Ok(primary_archive)
}
