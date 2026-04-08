use std::path::{Path, PathBuf};

use spoon_core::RuntimeLayout;

pub fn msvc_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root).msvc.managed.root
}

pub fn shims_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root).shims
}

pub fn scoop_git_usr_bin(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root)
        .scoop
        .apps_root
        .join("git")
        .join("current")
        .join("usr")
        .join("bin")
}

pub fn msvc_state_root(tool_root: &Path) -> PathBuf {
    msvc_state_root_for_layout(&RuntimeLayout::from_root(tool_root))
}

pub fn msvc_state_root_for_layout(layout: &RuntimeLayout) -> PathBuf {
    layout.msvc.managed.root.join("state")
}

pub fn msvc_cache_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root).msvc.managed.cache_root
}

pub fn msvc_toolchain_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root)
        .msvc
        .managed
        .toolchain_root
}

pub fn msvc_manifest_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root)
        .msvc
        .managed
        .manifest_root
}

pub fn official_msvc_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root)
        .msvc
        .official
        .instance_root
}

pub fn official_msvc_cache_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root).msvc.official.cache_root
}

pub fn official_msvc_state_root(tool_root: &Path) -> PathBuf {
    RuntimeLayout::from_root(tool_root)
        .msvc
        .official
        .root
        .join("state")
}


