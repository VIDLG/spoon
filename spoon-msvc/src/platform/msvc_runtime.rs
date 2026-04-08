use std::path::{Path, PathBuf};

use super::msvc_paths as paths;
use walkdir::WalkDir;

pub fn native_host_arch() -> &'static str {
    crate::paths::native_msvc_arch()
}

pub fn find_preferred_msvc_binary(
    root: &Path,
    target_arch: &str,
    candidates: &[&str],
) -> Option<PathBuf> {
    let host_arch = native_host_arch().to_ascii_lowercase();
    let target_arch = target_arch.to_ascii_lowercase();
    let mut matches = WalkDir::new(root)
        .into_iter()
        .flatten()
        .filter(|entry| {
            entry.file_type().is_file()
                && candidates.iter().any(|candidate| {
                    entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| name.eq_ignore_ascii_case(candidate))
                })
        })
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();
    matches.sort_by_key(|path| {
        let lowered = path.display().to_string().to_ascii_lowercase();
        let host_target = format!("host{}\\{}", host_arch, target_arch);
        let host_native = format!("host{}\\", host_arch);
        (
            !lowered.contains(&host_target),
            !lowered.contains(&host_native),
            lowered,
        )
    });
    matches.into_iter().next()
}

pub fn is_target_arch_dir(path: &Path, target_arch: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case(target_arch))
}

pub fn msvc_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_toolchain_root(tool_root)
}

pub fn runtime_state_path(tool_root: &Path) -> PathBuf {
    paths::msvc_state_root(tool_root).join("runtime.json")
}

pub fn manifest_dir(tool_root: &Path) -> PathBuf {
    paths::msvc_manifest_root(tool_root)
}
