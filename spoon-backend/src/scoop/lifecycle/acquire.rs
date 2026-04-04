use std::path::{Path, PathBuf};

use crate::{BackendEvent, CancellationToken, Result};

use super::super::host::ensure_downloaded_archive;
use super::super::package_source::{PackageAsset, ResolvedPackageSource};

pub(crate) async fn acquire_assets(
    tool_root: &Path,
    package_name: &str,
    source: &ResolvedPackageSource,
    assets: &[PackageAsset],
    proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<PathBuf>> {
    let mut archive_paths = Vec::new();
    for asset in assets {
        archive_paths.push(
            ensure_downloaded_archive(
                tool_root,
                package_name,
                source,
                asset,
                proxy,
                cancel,
                emit,
            )
            .await?,
        );
    }
    Ok(archive_paths)
}
