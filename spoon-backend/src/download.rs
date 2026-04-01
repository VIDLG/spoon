use std::path::Path;

use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::{
    BackendError, BackendEvent, CancellationToken, ProgressEvent, ProgressKind, Result,
    check_token_cancel,
};

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
