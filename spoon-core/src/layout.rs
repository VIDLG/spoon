use std::path::{Path, PathBuf};

/// Top-level layout for the Spoon runtime root directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLayout {
    pub root: PathBuf,
    pub shims: PathBuf,
    pub scoop: ScoopLayout,
    pub msvc: MsvcLayout,
}

/// Layout for Scoop package manager directories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoopLayout {
    pub root: PathBuf,
    pub state_root: PathBuf,
    pub cache_root: PathBuf,
    pub buckets_root: PathBuf,
    pub apps_root: PathBuf,
    pub persist_root: PathBuf,
}

/// Layout for MSVC toolchain directories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsvcLayout {
    pub root: PathBuf,
    pub managed: ManagedMsvcLayout,
    pub official: OfficialMsvcLayout,
}

/// Layout for managed (direct-download) MSVC toolchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedMsvcLayout {
    pub root: PathBuf,
    pub state_root: PathBuf,
    pub cache_root: PathBuf,
    pub toolchain_root: PathBuf,
    pub manifest_root: PathBuf,
}

/// Layout for official (Visual Studio Installer) MSVC toolchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficialMsvcLayout {
    pub root: PathBuf,
    pub instance_root: PathBuf,
    pub cache_root: PathBuf,
    pub state_root: PathBuf,
}

impl RuntimeLayout {
    pub fn from_root(root: &Path) -> Self {
        let root = root.to_path_buf();
        let shims = root.join("shims");

        let scoop_root = root.join("scoop");
        let scoop_state_root = scoop_root.join("state");
        let scoop = ScoopLayout {
            root: scoop_root.clone(),
            state_root: scoop_state_root,
            cache_root: scoop_root.join("cache"),
            buckets_root: scoop_root.join("buckets"),
            apps_root: scoop_root.join("apps"),
            persist_root: scoop_root.join("persist"),
        };

        let msvc_root = root.join("msvc");
        let managed_root = msvc_root.join("managed");
        let official_root = msvc_root.join("official");
        let managed = ManagedMsvcLayout {
            root: managed_root.clone(),
            state_root: managed_root.join("state"),
            cache_root: managed_root.join("cache"),
            toolchain_root: managed_root.join("toolchain"),
            manifest_root: managed_root.join("cache").join("manifest"),
        };
        let official = OfficialMsvcLayout {
            root: official_root.clone(),
            instance_root: official_root.join("instance"),
            cache_root: official_root.join("cache"),
            state_root: official_root.join("state"),
        };

        Self {
            root,
            shims,
            scoop,
            msvc: MsvcLayout {
                root: msvc_root,
                managed,
                official,
            },
        }
    }
}

impl ScoopLayout {
    pub fn bucket_root(&self, bucket_name: &str) -> PathBuf {
        self.buckets_root.join(bucket_name)
    }

    pub fn package_app_root(&self, package_name: &str) -> PathBuf {
        self.apps_root.join(package_name)
    }

    pub fn package_version_root(&self, package_name: &str, version: &str) -> PathBuf {
        self.package_app_root(package_name).join(version)
    }

    pub fn package_current_root(&self, package_name: &str) -> PathBuf {
        self.package_app_root(package_name).join("current")
    }

    pub fn package_persist_root(&self, package_name: &str) -> PathBuf {
        self.persist_root.join(package_name)
    }

    /// Compute the cache file path for a package asset.
    pub fn package_cache_file(&self, package_name: &str, version: &str, target_name: &str) -> PathBuf {
        self.cache_root.join(format!("{}-{}-{}", package_name, version, target_name))
    }
}
