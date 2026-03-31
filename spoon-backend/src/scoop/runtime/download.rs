use std::path::{Path, PathBuf};

use sha1::Sha1;
use sha2::{Digest, Sha256};
use tokio::fs;

use super::{PackagePayload, SelectedPackageSource};
use crate::{
    BackendError, BackendEvent, CancellationToken, ProgressEvent, ReqwestClientBuilder, Result,
    event::progress_kind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageHashAlgorithm {
    Sha256,
    Sha1,
}

impl PackageHashAlgorithm {
    fn label(self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Sha1 => "sha1",
        }
    }
}

fn parse_package_hash(expected: &str) -> (PackageHashAlgorithm, &str) {
    let trimmed = expected.trim();
    if let Some(value) = trimmed.strip_prefix("sha1:") {
        return (PackageHashAlgorithm::Sha1, value.trim());
    }
    if let Some(value) = trimmed.strip_prefix("sha256:") {
        return (PackageHashAlgorithm::Sha256, value.trim());
    }
    (PackageHashAlgorithm::Sha256, trimmed)
}

pub fn hash_matches(bytes: &[u8], expected: &str) -> bool {
    let (algorithm, expected_hex) = parse_package_hash(expected);
    match algorithm {
        PackageHashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(bytes);
            format!("{:x}", hasher.finalize()).eq_ignore_ascii_case(expected_hex)
        }
        PackageHashAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            hasher.update(bytes);
            format!("{:x}", hasher.finalize()).eq_ignore_ascii_case(expected_hex)
        }
    }
}

fn invalid_package_hash_message(expected: &str, path: &Path) -> String {
    let (algorithm, _) = parse_package_hash(expected);
    format!(
        "invalid package {} for {}",
        algorithm.label(),
        path.display()
    )
}

pub fn package_cache_file(
    tool_root: &Path,
    package_name: &str,
    version: &str,
    payload: &PackagePayload,
) -> PathBuf {
    let ext = payload
        .target_name
        .as_deref()
        .and_then(|value| Path::new(value).extension())
        .or_else(|| Path::new(&payload.url).extension())
        .map(|value| format!(".{}", value.to_string_lossy()))
        .unwrap_or_else(|| ".download".to_string());
    let hash_suffix = payload.hash.chars().take(12).collect::<String>();
    super::super::paths::scoop_root(tool_root)
        .join("cache")
        .join(format!("{package_name}#{version}#{hash_suffix}{ext}"))
}

pub async fn ensure_downloaded_archive(
    tool_root: &Path,
    package_name: &str,
    source: &SelectedPackageSource,
    payload: &PackagePayload,
    proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<PathBuf> {
    let cache_path = package_cache_file(tool_root, package_name, &source.version, payload);
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|err| BackendError::fs("create", parent, err))?;
    }
    if cache_path.exists() {
        let bytes = fs::read(&cache_path)
            .await
            .map_err(|err| BackendError::fs("read", &cache_path, err))?;
        if hash_matches(&bytes, &payload.hash) {
            tracing::info!("Reused cached archive: {}", cache_path.display());
            return Ok(cache_path);
        }
        let _ = fs::remove_file(&cache_path).await;
    }

    if let Some(local_path) = payload.url.strip_prefix("file:///") {
        let local_path = PathBuf::from(local_path.replace('/', "\\"));
        let bytes = fs::read(&local_path)
            .await
            .map_err(|err| BackendError::fs("read", &local_path, err))?;
        if !hash_matches(&bytes, &payload.hash) {
            return Err(BackendError::Other(invalid_package_hash_message(
                &payload.hash,
                &local_path,
            )));
        }
        fs::write(&cache_path, &bytes)
            .await
            .map_err(|err| BackendError::fs("write", &cache_path, err))?;
        tracing::info!("Copied local archive into {}", cache_path.display());
        return Ok(cache_path);
    }

    tracing::info!(
        "Downloading Scoop package '{}' from {}",
        package_name,
        payload.url
    );
    let client = ReqwestClientBuilder::new().proxy(proxy)?.build()?;
    let response = client
        .get(&payload.url)
        .send()
        .await
        .map_err(|err| BackendError::network(&payload.url, err))?;
    let mut response = response
        .error_for_status()
        .map_err(|err| BackendError::network(&payload.url, err))?;
    let total = response.content_length();
    let mut file = fs::File::create(&cache_path)
        .await
        .map_err(|err| BackendError::fs("create", &cache_path, err))?;
    let mut downloaded = 0_u64;
    let mut first_progress = true;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|err| BackendError::network(&payload.url, err))?
    {
        if cancel.is_some_and(CancellationToken::is_cancelled) {
            return Err(BackendError::Cancelled);
        }
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .map_err(|err| BackendError::fs("write", &cache_path, err))?;
        downloaded += chunk.len() as u64;
        if first_progress {
            tracing::info!("Download progress started for {}", cache_path.display());
            first_progress = false;
        }
        emit(BackendEvent::Progress(ProgressEvent::bytes(
            progress_kind::DOWNLOAD,
            cache_path.display().to_string(),
            downloaded,
            total,
        )));
    }
    tokio::io::AsyncWriteExt::flush(&mut file).await.ok();
    let bytes = fs::read(&cache_path)
        .await
        .map_err(|err| BackendError::fs("read", &cache_path, err))?;
    if !hash_matches(&bytes, &payload.hash) {
        return Err(BackendError::Other(invalid_package_hash_message(
            &payload.hash,
            &cache_path,
        )));
    }
    tracing::info!("Downloaded archive into {}", cache_path.display());
    Ok(cache_path)
}
