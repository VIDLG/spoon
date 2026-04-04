use std::path::Path;

use crate::BackendError;
use crate::scoop::ResolvedPackageSource;

use super::shims::{expanded_shim_targets, shim_target_path};
use super::super::ScoopRuntimeHost;

pub fn installed_targets_exist(
    package_name: &str,
    current_root: &Path,
    source: &ResolvedPackageSource,
    host: &dyn ScoopRuntimeHost,
) -> bool {
    expanded_shim_targets(package_name, current_root, source, host)
        .iter()
        .any(|target| shim_target_path(current_root, source, target).exists())
        || source
            .shortcuts
            .iter()
            .any(|shortcut| current_root.join(&shortcut.target_path).exists())
}

pub fn installer_layout_error(current_root: &Path, source: &ResolvedPackageSource) -> BackendError {
    let expected_bins = source
        .bins
        .iter()
        .map(|target| target.relative_path.clone())
        .collect::<Vec<_>>();
    let expected_shortcuts = source
        .shortcuts
        .iter()
        .map(|entry| entry.target_path.clone())
        .collect::<Vec<_>>();
    BackendError::InstallerLayoutMissingTargets {
        current_root: current_root.to_path_buf(),
        expected_bins: if expected_bins.is_empty() {
            "-".to_string()
        } else {
            expected_bins.join(", ")
        },
        expected_shortcuts: if expected_shortcuts.is_empty() {
            "-".to_string()
        } else {
            expected_shortcuts.join(", ")
        },
    }
}
