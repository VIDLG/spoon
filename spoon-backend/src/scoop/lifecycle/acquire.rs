use std::path::{Path, PathBuf};

use crate::{BackendEvent, CancellationToken, Result};

use super::super::host::ensure_downloaded_archive;
use super::super::package_source::{PackagePayload, SelectedPackageSource};

pub(crate) async fn acquire_payloads(
    tool_root: &Path,
    package_name: &str,
    source: &SelectedPackageSource,
    payloads: &[PackagePayload],
    proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<PathBuf>> {
    let mut archive_paths = Vec::new();
    for payload in payloads {
        archive_paths.push(
            ensure_downloaded_archive(
                tool_root,
                package_name,
                source,
                payload,
                proxy,
                cancel,
                emit,
            )
            .await?,
        );
    }
    Ok(archive_paths)
}
