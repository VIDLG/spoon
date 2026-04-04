use std::path::Path;

use sha1::Sha1;
use sha2::{Digest, Sha256};
use strum_macros::AsRefStr;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::{
    BackendError, BackendEvent, CancellationToken, ProgressEvent, ProgressKind, Result,
    check_token_cancel,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "lowercase")]
enum HashAlgorithm {
    Sha256,
    Sha1,
}

impl HashAlgorithm {
    fn parse_expected(expected: &str) -> (Self, &str) {
        let trimmed = expected.trim();
        if let Some(value) = trimmed.strip_prefix("sha1:") {
            return (Self::Sha1, value.trim());
        }
        if let Some(value) = trimmed.strip_prefix("sha256:") {
            return (Self::Sha256, value.trim());
        }
        (Self::Sha256, trimmed)
    }
}

pub fn hash_matches(bytes: &[u8], expected: &str) -> bool {
    let (algorithm, expected_hex) = HashAlgorithm::parse_expected(expected);
    match algorithm {
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(bytes);
            format!("{:x}", hasher.finalize()).eq_ignore_ascii_case(expected_hex)
        }
        HashAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            hasher.update(bytes);
            format!("{:x}", hasher.finalize()).eq_ignore_ascii_case(expected_hex)
        }
    }
}

fn hash_label(expected: &str) -> &'static str {
    let (algorithm, _) = HashAlgorithm::parse_expected(expected);
    match algorithm {
        HashAlgorithm::Sha256 => "sha256",
        HashAlgorithm::Sha1 => "sha1",
    }
}

pub async fn copy_or_download_to_file(
    client: &reqwest::Client,
    url: &str,
    destination: &Path,
    progress_label: &str,
    progress_kind: ProgressKind,
    cancel: Option<&CancellationToken>,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<()> {
    check_token_cancel(cancel)?;

    if let Some(path) = url.strip_prefix("file:///") {
        fs::copy(path, destination)
            .await
            .map_err(|err| BackendError::fs("copy", destination, err))?;
        return Ok(());
    }

    let path = Path::new(url);
    if path.exists() {
        fs::copy(path, destination)
            .await
            .map_err(|err| BackendError::fs("copy", destination, err))?;
        return Ok(());
    }

    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|err| BackendError::network(url, err))?
        .error_for_status()
        .map_err(|err| BackendError::network(url, err))?;

    let total_bytes = response.content_length();
    let mut file = fs::File::create(destination)
        .await
        .map_err(|err| BackendError::fs("create", destination, err))?;
    let mut downloaded_bytes = 0_u64;
    let mut last_emitted_percent = None;
    let mut last_emitted_mb_tenths = None;

    if let Some(total_bytes) = total_bytes {
        last_emitted_percent = Some(0);
        emit(BackendEvent::Progress(ProgressEvent::bytes(
            progress_kind,
            progress_label,
            0,
            Some(total_bytes),
        )));
    } else {
        emit(BackendEvent::Progress(ProgressEvent::bytes(
            progress_kind,
            progress_label,
            0,
            None,
        )));
    }

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|err| BackendError::network(url, err))?
    {
        check_token_cancel(cancel)?;
        file.write_all(&chunk)
            .await
            .map_err(|err| BackendError::fs("write", destination, err))?;
        downloaded_bytes += chunk.len() as u64;

        if let Some(total_bytes) = total_bytes {
            let percent = if total_bytes == 0 {
                0
            } else {
                ((downloaded_bytes as f64 / total_bytes as f64) * 100.0)
                    .clamp(0.0, 100.0)
                    .round() as u64
            };
            if last_emitted_percent != Some(percent) {
                last_emitted_percent = Some(percent);
                emit(BackendEvent::Progress(ProgressEvent::bytes(
                    progress_kind,
                    progress_label,
                    downloaded_bytes,
                    Some(total_bytes),
                )));
            }
        } else {
            let downloaded_mb_tenths = downloaded_bytes / (1024 * 1024 / 10);
            if last_emitted_mb_tenths != Some(downloaded_mb_tenths) {
                last_emitted_mb_tenths = Some(downloaded_mb_tenths);
                emit(BackendEvent::Progress(ProgressEvent::bytes(
                    progress_kind,
                    progress_label,
                    downloaded_bytes,
                    None,
                )));
            }
        }
    }

    emit(BackendEvent::Progress(ProgressEvent::bytes(
        progress_kind,
        progress_label,
        downloaded_bytes,
        total_bytes,
    )));

    file.flush()
        .await
        .map_err(|err| BackendError::fs("flush", destination, err))?;

    Ok(())
}

pub async fn materialize_to_file_with_hash(
    client: &reqwest::Client,
    url: &str,
    destination: &Path,
    expected_hash: &str,
    progress_label: &str,
    progress_kind: ProgressKind,
    cancel: Option<&CancellationToken>,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<()> {
    copy_or_download_to_file(
        client,
        url,
        destination,
        progress_label,
        progress_kind,
        cancel,
        emit,
    )
    .await?;

    let bytes = fs::read(destination)
        .await
        .map_err(|err| BackendError::fs("read", destination, err))?;
    if hash_matches(&bytes, expected_hash) {
        return Ok(());
    }

    Err(BackendError::Other(format!(
        "invalid package {} for {}",
        hash_label(expected_hash),
        destination.display()
    )))
}
