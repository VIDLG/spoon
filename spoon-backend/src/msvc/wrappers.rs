use std::path::{Path, PathBuf};

use fs_err as fs;

use crate::{BackendError, BackendEvent, Result};

use super::ToolchainFlags;
use super::common::join_windows_path;
use super::paths;

pub async fn managed_toolchain_flags(tool_root: &Path) -> Result<ToolchainFlags> {
    let request = super::MsvcRequest::for_tool_root(tool_root);
    managed_toolchain_flags_with_request(&request).await
}

pub(crate) async fn managed_toolchain_flags_with_request(
    request: &super::MsvcRequest,
) -> Result<ToolchainFlags> {
    let tool_root = request.root.as_path();
    let (toolchain_root, _managed_root, _) =
        super::validation::validate_toolchain_layout(tool_root)?;
    let target_arch = request.normalized_target_arch();
    let compiler = super::find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["cl.exe", "cl.cmd", "cl.bat"],
    )
    .ok_or_else(|| {
        BackendError::Other(format!(
            "managed MSVC compiler was not found under {}",
            toolchain_root.display()
        ))
    })?;
    let linker = super::find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["link.exe", "link.cmd", "link.bat"],
    )
    .ok_or_else(|| {
        BackendError::Other(format!(
            "managed MSVC linker was not found under {}",
            toolchain_root.display()
        ))
    })?;
    let librarian = super::find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["lib.exe", "lib.cmd", "lib.bat"],
    )
    .ok_or_else(|| {
        BackendError::Other(format!(
            "managed MSVC librarian was not found under {}",
            toolchain_root.display()
        ))
    })?;
    let resource_compiler = super::find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["rc.exe", "rc.cmd", "rc.bat"],
    );
    let manifest_tool = super::find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["mt.exe", "mt.cmd", "mt.bat"],
    );
    let nmake = super::find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["nmake.exe", "nmake.cmd", "nmake.bat"],
    );
    let dumpbin = super::find_preferred_msvc_binary(
        &toolchain_root,
        &target_arch,
        &["dumpbin.exe", "dumpbin.cmd", "dumpbin.bat"],
    );
    let include_dirs = super::validation::include_dirs_for_validation(&toolchain_root);
    let lib_dirs = super::validation::lib_dirs_for_validation(&toolchain_root, &target_arch);
    if include_dirs.is_empty() {
        return Err(BackendError::Other(format!(
            "no managed include directories were discovered under {}",
            toolchain_root.display()
        )));
    }
    if lib_dirs.is_empty() {
        return Err(BackendError::Other(format!(
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
    Ok(ToolchainFlags {
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

pub(crate) fn spoon_cl_wrapper_path(tool_root: &Path) -> PathBuf {
    paths::shims_root(tool_root).join("spoon-cl.cmd")
}

pub(crate) fn spoon_link_wrapper_path(tool_root: &Path) -> PathBuf {
    paths::shims_root(tool_root).join("spoon-link.cmd")
}

pub(crate) fn spoon_lib_wrapper_path(tool_root: &Path) -> PathBuf {
    paths::shims_root(tool_root).join("spoon-lib.cmd")
}

pub(crate) fn spoon_rc_wrapper_path(tool_root: &Path) -> PathBuf {
    paths::shims_root(tool_root).join("spoon-rc.cmd")
}

pub(crate) fn spoon_mt_wrapper_path(tool_root: &Path) -> PathBuf {
    paths::shims_root(tool_root).join("spoon-mt.cmd")
}

pub(crate) fn spoon_nmake_wrapper_path(tool_root: &Path) -> PathBuf {
    paths::shims_root(tool_root).join("spoon-nmake.cmd")
}

pub(crate) fn spoon_dumpbin_wrapper_path(tool_root: &Path) -> PathBuf {
    paths::shims_root(tool_root).join("spoon-dumpbin.cmd")
}

fn managed_command_profile_is_extended(command_profile: &str) -> bool {
    command_profile.eq_ignore_ascii_case("extended")
}

fn managed_optional_wrapper_specs<'a>(
    tool_root: &Path,
    flags: &'a ToolchainFlags,
) -> [(&'static str, PathBuf, Option<&'a Path>); 4] {
    [
        (
            "spoon-rc",
            spoon_rc_wrapper_path(tool_root),
            flags.resource_compiler.as_deref(),
        ),
        (
            "spoon-mt",
            spoon_mt_wrapper_path(tool_root),
            flags.manifest_tool.as_deref(),
        ),
        (
            "spoon-nmake",
            spoon_nmake_wrapper_path(tool_root),
            flags.nmake.as_deref(),
        ),
        (
            "spoon-dumpbin",
            spoon_dumpbin_wrapper_path(tool_root),
            flags.dumpbin.as_deref(),
        ),
    ]
}

fn write_wrapper_script(
    script_path: &Path,
    tool_path: &Path,
    include_dirs: &[PathBuf],
    lib_dirs: &[PathBuf],
    path_dirs: &[PathBuf],
) -> Result<()> {
    if let Some(parent) = script_path.parent() {
        fs::create_dir_all(parent).map_err(|err| BackendError::fs("create", parent, err))?;
    }
    let include = join_windows_path(include_dirs);
    let lib = join_windows_path(lib_dirs);
    let managed_path = join_windows_path(path_dirs);
    let path_setup = if managed_path.is_empty() {
        String::new()
    } else {
        format!(
            "if defined PATH (\r\n  set \"PATH={managed_path};%PATH%\"\r\n) else (\r\n  set \"PATH={managed_path}\"\r\n)\r\n"
        )
    };
    let content = format!(
        "@echo off\r\nsetlocal\r\nset \"INCLUDE={include}\"\r\nset \"LIB={lib}\"\r\n{path_setup}\"{}\" %*\r\n",
        tool_path.display()
    );
    fs::write(script_path, content).map_err(|err| BackendError::fs("write", script_path, err))
}

pub fn write_managed_toolchain_wrappers(
    tool_root: &Path,
    command_profile: &str,
    flags: &ToolchainFlags,
) -> Result<Vec<String>> {
    let cl_wrapper = spoon_cl_wrapper_path(tool_root);
    let link_wrapper = spoon_link_wrapper_path(tool_root);
    let lib_wrapper = spoon_lib_wrapper_path(tool_root);
    let extended = managed_command_profile_is_extended(command_profile);
    write_wrapper_script(
        &cl_wrapper,
        &flags.compiler,
        &flags.include_dirs,
        &flags.lib_dirs,
        &flags.path_dirs,
    )?;
    write_wrapper_script(
        &link_wrapper,
        &flags.linker,
        &flags.include_dirs,
        &flags.lib_dirs,
        &flags.path_dirs,
    )?;
    write_wrapper_script(
        &lib_wrapper,
        &flags.librarian,
        &flags.include_dirs,
        &flags.lib_dirs,
        &flags.path_dirs,
    )?;
    Ok(vec![
        format!("Wrote managed wrapper {}", cl_wrapper.display()),
        format!("Wrote managed wrapper {}", link_wrapper.display()),
        format!("Wrote managed wrapper {}", lib_wrapper.display()),
    ]
    .into_iter()
    .chain(write_optional_managed_wrappers(tool_root, flags, extended)?)
    .collect())
}

pub(crate) fn ensure_managed_toolchain_wrappers(
    tool_root: &Path,
    command_profile: &str,
    flags: &ToolchainFlags,
) -> Result<Vec<String>> {
    let cl_wrapper = spoon_cl_wrapper_path(tool_root);
    let link_wrapper = spoon_link_wrapper_path(tool_root);
    let lib_wrapper = spoon_lib_wrapper_path(tool_root);
    let extended = managed_command_profile_is_extended(command_profile);
    let optional_layout_ok = managed_optional_wrapper_specs(tool_root, flags)
        .into_iter()
        .all(|(_, path, tool_path)| {
            if extended {
                tool_path.is_none() || path.exists()
            } else {
                !path.exists()
            }
        });
    if cl_wrapper.exists() && link_wrapper.exists() && lib_wrapper.exists() && optional_layout_ok {
        return Ok(Vec::new());
    }
    write_managed_toolchain_wrappers(tool_root, command_profile, flags)
}

fn write_optional_managed_wrappers(
    tool_root: &Path,
    flags: &ToolchainFlags,
    extended: bool,
) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    for (_, wrapper_path, tool_path) in managed_optional_wrapper_specs(tool_root, flags) {
        if extended {
            lines.extend(write_optional_managed_wrapper(
                wrapper_path,
                tool_path,
                flags,
            )?);
        } else if wrapper_path.exists() {
            fs::remove_file(&wrapper_path)
                .map_err(|err| BackendError::fs("remove", &wrapper_path, err))?;
            lines.push(format!(
                "Removed managed wrapper {} (not selected by the default command profile).",
                wrapper_path.display()
            ));
        }
    }
    Ok(lines)
}

fn write_optional_managed_wrapper(
    wrapper_path: PathBuf,
    tool_path: Option<&Path>,
    flags: &ToolchainFlags,
) -> Result<Vec<String>> {
    let Some(tool_path) = tool_path else {
        return Ok(Vec::new());
    };
    write_wrapper_script(
        &wrapper_path,
        tool_path,
        &flags.include_dirs,
        &flags.lib_dirs,
        &flags.path_dirs,
    )?;
    Ok(vec![format!(
        "Wrote managed wrapper {}",
        wrapper_path.display()
    )])
}

pub async fn reapply_managed_command_surface_streaming<F>(
    tool_root: &Path,
    command_profile: &str,
    _emit: F,
) -> Result<Vec<String>>
where
    F: FnMut(BackendEvent),
{
    let runtime_state = super::runtime_state_path(tool_root);
    if !runtime_state.exists() {
        return Ok(vec![
            "Managed MSVC toolchain is not installed; no wrapper changes were applied.".to_string(),
        ]);
    }

    let flags = managed_toolchain_flags(tool_root).await?;
    let mut lines = ensure_managed_toolchain_wrappers(tool_root, command_profile, &flags)?;
    if lines.is_empty() {
        lines.push("Managed wrapper set already matches the selected command profile.".to_string());
    }
    for line in &lines {
        tracing::info!("{line}");
    }
    Ok(lines)
}

pub fn remove_managed_toolchain_wrappers(tool_root: &Path) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    for path in [
        spoon_cl_wrapper_path(tool_root),
        spoon_link_wrapper_path(tool_root),
        spoon_lib_wrapper_path(tool_root),
        spoon_rc_wrapper_path(tool_root),
        spoon_mt_wrapper_path(tool_root),
        spoon_nmake_wrapper_path(tool_root),
        spoon_dumpbin_wrapper_path(tool_root),
    ] {
        if path.exists() {
            fs::remove_file(&path).map_err(|err| BackendError::fs("remove", &path, err))?;
            lines.push(format!("Removed managed wrapper {}.", path.display()));
        }
    }
    Ok(lines)
}
