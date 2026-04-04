use std::path::{Path, PathBuf};

use crate::scoop::ResolvedPackageSource;

pub(crate) fn resolve_env_add_paths(
    source: &ResolvedPackageSource,
    install_root: &Path,
) -> Vec<PathBuf> {
    source
        .env_add_path
        .iter()
        .filter_map(|entry| {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                None
            } else if trimmed == "." {
                Some(install_root.to_path_buf())
            } else {
                Some(install_root.join(trimmed))
            }
        })
        .collect()
}

pub(crate) fn resolve_env_set_entries(
    source: &ResolvedPackageSource,
    install_root: &Path,
    persist_root: &Path,
) -> Vec<(String, String)> {
    source
        .env_set
        .iter()
        .map(|(key, value)| {
            (
                key.clone(),
                value
                    .replace("$dir", &install_root.display().to_string())
                    .replace("$persist_dir", &persist_root.display().to_string())
                    .replace("$original_dir", &install_root.display().to_string()),
            )
        })
        .collect()
}
