use std::path::{Path, PathBuf};

use crate::layout::RuntimeLayout;

pub fn scoop_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root).scoop.root
}

pub fn shims_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root).shims
}

pub fn scoop_state_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root).scoop.state_root
}

pub fn scoop_bucket_registry_path(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root)
        .scoop
        .bucket_registry_path
}

pub fn scoop_package_state_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root).scoop.package_state_root
}

pub fn scoop_cache_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root).scoop.cache_root
}

pub fn scoop_buckets_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root).scoop.buckets_root
}

pub fn scoop_bucket_root(tool_root: &Path, bucket_name: &str) -> PathBuf {
    scoop_buckets_root(tool_root).join(bucket_name)
}

pub fn packages_state_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root).scoop.package_state_root
}

pub fn package_state_path(tool_root: &Path, package_name: &str) -> PathBuf {
    packages_state_root(tool_root).join(format!("{package_name}.json"))
}

pub fn package_app_root(tool_root: &Path, package_name: &str) -> PathBuf {
    RuntimeLayout::from_root(tool_root)
        .scoop
        .apps_root
        .join(package_name)
}

pub fn package_version_root(tool_root: &Path, package_name: &str, version: &str) -> PathBuf {
    package_app_root(tool_root, package_name).join(version)
}

pub fn package_current_root(tool_root: &Path, package_name: &str) -> PathBuf {
    package_app_root(tool_root, package_name).join("current")
}

pub fn package_persist_root(tool_root: &Path, package_name: &str) -> PathBuf {
    RuntimeLayout::from_root(tool_root)
        .scoop
        .persist_root
        .join(package_name)
}
