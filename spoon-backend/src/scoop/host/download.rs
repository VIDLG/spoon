use std::path::{Path, PathBuf};

use tokio::fs;

use crate::{
    BackendError, BackendEvent, CancellationToken, ReqwestClientBuilder, Result,
    download::{hash_matches, materialize_to_file_with_hash},
    event::progress_kind,
    layout::RuntimeLayout,
    scoop::{PackageAsset, ResolvedPackageSource},
};

pub async fn ensure_downloaded_archive(
    tool_root: &Path,
    package_name: &str,
    source: &ResolvedPackageSource,
    asset: &PackageAsset,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<PathBuf> {
    let layout = RuntimeLayout::from_root(tool_root);
    let cache_path = layout
        .scoop
        .package_cache_file(package_name, &source.version, asset);
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|err| BackendError::fs("create", parent, err))?;
    }
    if cache_path.exists() {
        let bytes = fs::read(&cache_path)
            .await
            .map_err(|err| BackendError::fs("read", &cache_path, err))?;
        if hash_matches(&bytes, &asset.hash) {
            tracing::info!("Reused cached archive: {}", cache_path.display());
            return Ok(cache_path);
        }
        let _ = fs::remove_file(&cache_path).await;
    }

    tracing::info!(
        "Materializing Scoop package '{}' from {}",
        package_name,
        asset.url
    );
    let client = ReqwestClientBuilder::new().proxy(proxy)?.build()?;
    materialize_to_file_with_hash(
        &client,
        &asset.url,
        &cache_path,
        &asset.hash,
        &cache_path.display().to_string(),
        progress_kind::DOWNLOAD,
        cancel,
        emit,
    )
    .await?;
    tracing::info!("Materialized archive into {}", cache_path.display());
    Ok(cache_path)
}
