//! Binary discovery — find MSVC binaries and build toolchain flags.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use spoon_core::{CoreError, Result};

use crate::types::MsvcRequest;

use super::pipeline;

pub fn find_preferred_msvc_binary(
    root: &Path,
    target_arch: &str,
    candidates: &[&str],
) -> Option<PathBuf> {
    let host_arch = pipeline::native_host_arch().to_ascii_lowercase();
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

pub async fn managed_toolchain_flags_with_request(
    request: &MsvcRequest,
) -> Result<crate::types::ToolchainFlags> {
    let tool_root = request.root.as_path();
    let toolchain_root = pipeline::msvc_dir(tool_root);
    let target_arch = request.normalized_target_arch();
    let compiler = find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["cl.exe", "cl.cmd", "cl.bat"],
    )
    .ok_or_else(|| {
        CoreError::Other(format!(
            "managed MSVC compiler was not found under {}",
            toolchain_root.display()
        ))
    })?;
    let linker = find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["link.exe", "link.cmd", "link.bat"],
    )
    .ok_or_else(|| {
        CoreError::Other(format!(
            "managed MSVC linker was not found under {}",
            toolchain_root.display()
        ))
    })?;
    let librarian = find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["lib.exe", "lib.cmd", "lib.bat"],
    )
    .ok_or_else(|| {
        CoreError::Other(format!(
            "managed MSVC librarian was not found under {}",
            toolchain_root.display()
        ))
    })?;
    let resource_compiler = find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["rc.exe", "rc.cmd", "rc.bat"],
    );
    let manifest_tool = find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["mt.exe", "mt.cmd", "mt.bat"],
    );
    let nmake = find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["nmake.exe", "nmake.cmd", "nmake.bat"],
    );
    let dumpbin = find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["dumpbin.exe", "dumpbin.cmd", "dumpbin.bat"],
    );
    let include_dirs = crate::validation::include_dirs_for_validation(&toolchain_root);
    let lib_dirs = crate::validation::lib_dirs_for_validation(&toolchain_root, &target_arch);
    if include_dirs.is_empty() {
        return Err(CoreError::Other(format!(
            "no managed include directories were discovered under {}",
            toolchain_root.display()
        )));
    }
    if lib_dirs.is_empty() {
        return Err(CoreError::Other(format!(
            "no managed library directories were discovered under {}",
            toolchain_root.display()
        )));
    }
    let mut path_dirs = Vec::new();
    if let Some(dir) = compiler.parent().map(Path::to_path_buf) {
        path_dirs.push(dir);
    }
    if let Some(dir) = linker.parent().map(Path::to_path_buf)
        && !path_dirs.iter().any(|existing| existing == &dir)
    {
        path_dirs.push(dir);
    }
    if let Some(dir) = librarian.parent().map(Path::to_path_buf)
        && !path_dirs.iter().any(|existing| existing == &dir)
    {
        path_dirs.push(dir);
    }
    for optional in [&resource_compiler, &manifest_tool, &nmake, &dumpbin] {
        if let Some(dir) = optional
            .as_ref()
            .and_then(|path| path.parent())
            .map(Path::to_path_buf)
            && !path_dirs.iter().any(|existing| existing == &dir)
        {
            path_dirs.push(dir);
        }
    }
    Ok(crate::types::ToolchainFlags {
        compiler,
        linker,
        librarian,
        resource_compiler,
        manifest_tool,
        nmake,
        dumpbin,
        include_dirs,
        lib_dirs,
        path_dirs,
    })
}
