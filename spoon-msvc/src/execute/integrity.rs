//! Integrity utilities — hashing, archive-kind detection, download helpers.

use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use spoon_core::{CoreError, Result};

use crate::facts::package_rules::ArchiveKind;
use crate::facts::manifest;

pub(crate) fn archive_kind_for_payload(payload: &manifest::SelectedPayload) -> Option<ArchiveKind> {
    crate::facts::package_rules::archive_kind(&payload.payload.file_name)
        .or_else(|| crate::facts::package_rules::archive_kind(&payload.payload.url))
}

pub(crate) fn decode_hex_sha256(hex: &str) -> Result<[u8; 32]> {
    let trimmed = hex.trim();
    if trimmed.len() != 64 {
        return Err(CoreError::Other(format!(
            "expected 64 hex chars for sha256, got {}",
            trimmed.len()
        )));
    }
    let mut out = [0_u8; 32];
    for index in 0..32 {
        let start = index * 2;
        out[index] = u8::from_str_radix(&trimmed[start..start + 2], 16)
            .map_err(|e| CoreError::Other(format!("invalid sha256 hex '{}': {e}", trimmed)))?;
    }
    Ok(out)
}

pub(crate) fn file_sha256(path: &Path) -> Result<[u8; 32]> {
    let mut file = fs_err::File::open(path)
        .map_err(|e| CoreError::fs("open", path, e))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 16 * 1024];
    loop {
        let read = file
            .read(&mut buf)
            .map_err(|e| CoreError::fs("read", path, e))?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(hasher.finalize().into())
}

pub(crate) fn payload_source_description(url: &str) -> String {
    if url.starts_with("file:///") {
        url.to_string()
    } else {
        Path::new(url)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| format!("remote payload {name}"))
            .unwrap_or_else(|| url.to_string())
    }
}

pub(crate) fn download_progress_target_label(file_name: &str) -> &str {
    Path::new(file_name)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(file_name)
}
