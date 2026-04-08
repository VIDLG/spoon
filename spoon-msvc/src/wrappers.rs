//! MSVC toolchain wrapper scripts.

use std::path::{Path, PathBuf};

use fs_err as fs;

use spoon_core::CoreError;
use crate::common::join_windows_path;
use crate::paths;
use crate::types::ToolchainFlags;

pub fn spoon_cl_wrapper_path(tool_root: &Path) -> PathBuf {
    paths::shims_root(tool_root).join("spoon-cl.cmd")
}

pub fn spoon_link_wrapper_path(tool_root: &Path) -> PathBuf {
    paths::shims_root(tool_root).join("spoon-link.cmd")
}

pub fn spoon_lib_wrapper_path(tool_root: &Path) -> PathBuf {
    paths::shims_root(tool_root).join("spoon-lib.cmd")
}

pub fn spoon_rc_wrapper_path(tool_root: &Path) -> PathBuf {
    paths::shims_root(tool_root).join("spoon-rc.cmd")
}

pub fn spoon_mt_wrapper_path(tool_root: &Path) -> PathBuf {
    paths::shims_root(tool_root).join("spoon-mt.cmd")
}

pub fn spoon_nmake_wrapper_path(tool_root: &Path) -> PathBuf {
    paths::shims_root(tool_root).join("spoon-nmake.cmd")
}

pub fn spoon_dumpbin_wrapper_path(tool_root: &Path) -> PathBuf {
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
) -> spoon_core::Result<()> {
    if let Some(parent) = script_path.parent() {
        fs::create_dir_all(parent).map_err(|e| CoreError::fs("create_dir_all", parent, e))?;
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
    fs::write(script_path, content).map_err(|e| CoreError::fs("write", script_path, e))
}

pub fn write_managed_toolchain_wrappers(
    tool_root: &Path,
    command_profile: &str,
    flags: &ToolchainFlags,
) -> spoon_core::Result<Vec<String>> {
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

pub fn ensure_managed_toolchain_wrappers(
    tool_root: &Path,
    command_profile: &str,
    flags: &ToolchainFlags,
) -> spoon_core::Result<Vec<String>> {
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
) -> spoon_core::Result<Vec<String>> {
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
                .map_err(|e| CoreError::fs("remove", &wrapper_path, e))?;
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
) -> spoon_core::Result<Vec<String>> {
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

pub fn remove_managed_toolchain_wrappers(tool_root: &Path) -> spoon_core::Result<Vec<String>> {
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
            fs::remove_file(&path).map_err(|e| CoreError::fs("remove", &path, e))?;
            lines.push(format!("Removed managed wrapper {}.", path.display()));
        }
    }
    Ok(lines)
}
