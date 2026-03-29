use std::path::Path;

use serde::Serialize;

use crate::layout::RuntimeLayout;

/// Aggregate backend status snapshot for app consumers.
///
/// Centralizes runtime roots and backend state facts so that the app shell
/// renders status, JSON output, and TUI background refresh from one source
/// instead of re-reading backend state files locally.
#[derive(Debug, Clone, Serialize)]
pub struct BackendStatusSnapshot {
    pub kind: &'static str,
    pub scoop: BackendScoopSummary,
    pub msvc: BackendMsvcSummary,
    pub runtime_roots: BackendRuntimeRoots,
}

/// Derives Scoop runtime summary from the backend query surface.
#[derive(Debug, Clone, Serialize)]
pub struct BackendScoopSummary {
    pub installed: bool,
    pub root: String,
    pub shims: String,
    pub bucket_count: usize,
    pub installed_package_count: usize,
    pub buckets: Vec<BackendBucketEntry>,
    pub installed_packages: Vec<BackendInstalledPackageEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackendBucketEntry {
    pub name: String,
    pub branch: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackendInstalledPackageEntry {
    pub name: String,
    pub version: String,
}

/// Minimal MSVC summary for status display.
#[derive(Debug, Clone, Serialize)]
pub struct BackendMsvcSummary {
    pub managed_status: String,
    pub managed_version: Option<String>,
    pub managed_root: String,
    pub official_status: String,
    pub official_version: Option<String>,
    pub official_root: String,
}

/// Runtime root paths derived from the backend layout.
#[derive(Debug, Clone, Serialize)]
pub struct BackendRuntimeRoots {
    pub root: String,
    pub scoop: String,
    pub shims: String,
    pub managed_msvc: String,
    pub managed_toolchain: String,
    pub official_msvc: String,
    pub scoop_state: String,
    pub scoop_cache: String,
}

impl Default for BackendRuntimeRoots {
    fn default() -> Self {
        Self {
            root: String::new(),
            scoop: String::new(),
            shims: String::new(),
            managed_msvc: String::new(),
            managed_toolchain: String::new(),
            official_msvc: String::new(),
            scoop_state: String::new(),
            scoop_cache: String::new(),
        }
    }
}

impl BackendStatusSnapshot {
    /// Build a status snapshot from the backend query surface.
    ///
    /// Calls [`scoop::runtime_status`](crate::scoop::runtime_status) and
    /// [`msvc::status`](crate::msvc::status) internally so the app never
    /// opens backend state files itself.
    pub async fn from_tool_root(tool_root: &Path) -> Self {
        let layout = RuntimeLayout::from_root(tool_root);
        let scoop_status = crate::scoop::runtime_status(tool_root).await;
        let msvc_status = crate::msvc::status(tool_root).await;

        Self {
            kind: "backend_status_snapshot",
            scoop: BackendScoopSummary {
                installed: scoop_status.success,
                root: layout.scoop.root.display().to_string(),
                shims: layout.shims.display().to_string(),
                bucket_count: scoop_status.runtime.bucket_count,
                installed_package_count: scoop_status.runtime.installed_package_count,
                buckets: scoop_status
                    .buckets
                    .into_iter()
                    .map(|b| BackendBucketEntry {
                        name: b.name,
                        branch: b.branch,
                        source: b.source,
                    })
                    .collect(),
                installed_packages: scoop_status
                    .installed_packages
                    .into_iter()
                    .map(|p| BackendInstalledPackageEntry {
                        name: p.name,
                        version: p.version,
                    })
                    .collect(),
            },
            msvc: BackendMsvcSummary {
                managed_status: msvc_status.managed.status.clone(),
                managed_version: msvc_status.managed.installed_version.clone(),
                managed_root: msvc_status.managed.root.clone(),
                official_status: msvc_status.official.status.clone(),
                official_version: msvc_status.official.installed_version.clone(),
                official_root: msvc_status.official.root.clone(),
            },
            runtime_roots: BackendRuntimeRoots {
                root: layout.root.display().to_string(),
                scoop: layout.scoop.root.display().to_string(),
                shims: layout.shims.display().to_string(),
                managed_msvc: layout.msvc.managed.root.display().to_string(),
                managed_toolchain: layout.msvc.managed.toolchain_root.display().to_string(),
                official_msvc: layout.msvc.official.root.display().to_string(),
                scoop_state: layout.scoop.state_root.display().to_string(),
                scoop_cache: layout.scoop.cache_root.display().to_string(),
            },
        }
    }

    /// Look up an installed package version by package name.
    pub fn installed_package_version(&self, package_name: &str) -> Option<&str> {
        self.scoop
            .installed_packages
            .iter()
            .find(|p| p.name == package_name)
            .map(|p| p.version.as_str())
    }

    /// Check whether a package state file exists via the snapshot.
    pub fn has_installed_package(&self, package_name: &str) -> bool {
        self.installed_package_version(package_name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_kind_is_backend_status_snapshot() {
        assert_eq!(BackendStatusSnapshot::kind_of(), "backend_status_snapshot");
    }
}

impl BackendStatusSnapshot {
    /// Static accessor for the `kind` field, usable in unit tests without async.
    pub const fn kind_of() -> &'static str {
        "backend_status_snapshot"
    }
}
