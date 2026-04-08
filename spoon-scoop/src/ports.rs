use std::path::Path;

use crate::AppliedIntegration;
use crate::SupplementalShimSpec;
use crate::error::Result;

/// Port abstraction for system-level operations.
///
/// The library handles all filesystem I/O, manifest parsing, and workflow logic.
/// The binary implements this trait to provide PATH manipulation and per-package
/// integration logic.
pub trait ScoopPorts {
    /// Add a directory to the user's permanent PATH.
    fn ensure_user_path_entry(&self, path: &Path) -> Result<()>;

    /// Add a directory to the current process PATH.
    fn ensure_process_path_entry(&self, path: &Path);

    /// Remove a directory from the user's permanent PATH.
    fn remove_user_path_entry(&self, path: &Path) -> Result<()>;

    /// Remove a directory from the current process PATH.
    fn remove_process_path_entry(&self, path: &Path);

    /// Return supplemental shim specifications for a package.
    fn supplemental_shims(&self, package_name: &str, current_root: &Path) -> Vec<SupplementalShimSpec>;

    /// Apply host-specific integrations for a package.
    fn apply_integrations(
        &self,
        package_name: &str,
        current_root: &Path,
        persist_root: &Path,
    ) -> Result<Vec<AppliedIntegration>>;
}
