use std::path::{Path, PathBuf};

use crate::facts::manifest::SelectedPayload;
use crate::platform::msvc_paths as paths;

pub fn payload_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root).join("archives")
}

pub fn extracted_payload_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root)
        .join("expanded")
        .join("archives")
}

pub fn extracted_msi_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root)
        .join("expanded")
        .join("msi")
}

pub fn install_image_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root).join("image")
}

pub fn msi_metadata_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root)
        .join("metadata")
        .join("msi")
}

pub fn msi_staging_cache_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_cache_root(tool_root).join("stage").join("msi")
}

pub fn payload_cache_entry_name(payload: &SelectedPayload) -> String {
    let leaf = Path::new(&payload.payload.file_name)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("payload.bin");
    format!("{}-{}", payload.payload.sha256.to_ascii_lowercase(), leaf)
}

pub fn payload_cache_entry_path(tool_root: &Path, payload: &SelectedPayload) -> PathBuf {
    payload_cache_dir(tool_root).join(payload_cache_entry_name(payload))
}

pub fn extracted_payload_entry_dir(tool_root: &Path, payload: &SelectedPayload) -> PathBuf {
    extracted_payload_cache_dir(tool_root).join(payload.payload.sha256.to_ascii_lowercase())
}

pub fn msi_metadata_entry_path(tool_root: &Path, payload: &SelectedPayload) -> PathBuf {
    msi_metadata_cache_dir(tool_root).join(format!(
        "{}.txt",
        payload.payload.sha256.to_ascii_lowercase()
    ))
}

pub fn msi_staging_entry_dir(tool_root: &Path, payload: &SelectedPayload) -> PathBuf {
    msi_staging_cache_dir(tool_root).join(payload.payload.sha256.to_ascii_lowercase())
}

pub fn extracted_msi_entry_dir(tool_root: &Path, payload: &SelectedPayload) -> PathBuf {
    extracted_msi_cache_dir(tool_root).join(payload.payload.sha256.to_ascii_lowercase())
}
