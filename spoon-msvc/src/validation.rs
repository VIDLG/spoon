//! MSVC validation — compile and run validation samples.

use std::path::{Path, PathBuf};

use fs_err as fs;

use crate::common::{
    find_all_named_files, join_windows_path, path_components_lowercase,
    unique_existing_dirs,
};
use crate::paths;
use crate::wrappers;
use spoon_core::{CoreError, Result};

const VALIDATE_CPP_HELLO_TEMPLATE: &str = include_str!("validate_templates/cpp/hello.cpp");
const VALIDATE_RUST_CARGO_TEMPLATE: &str = include_str!("validate_templates/rust/Cargo.toml");
const VALIDATE_RUST_BUILD_RS_TEMPLATE: &str = include_str!("validate_templates/rust/build.rs");
const VALIDATE_RUST_MAIN_TEMPLATE: &str = include_str!("validate_templates/rust/src/main.rs");
const VALIDATE_RUST_HELPER_C_TEMPLATE: &str =
    include_str!("validate_templates/rust/native/helper.c");
const VALIDATE_RUST_CARGO_CONFIG_TEMPLATE: &str =
    include_str!("validate_templates/rust/.cargo/config.toml");

pub struct RustValidationTemplateOptions<'a> {
    pub linker: &'a Path,
    pub sample_label: &'a str,
    pub native_helper_label: &'a str,
    pub linker_label: &'a str,
}

pub fn validate_toolchain_layout(
    tool_root: &Path,
) -> Result<(PathBuf, PathBuf, Vec<String>)> {
    let managed_root = paths::msvc_root(tool_root);
    let toolchain_root = paths::msvc_toolchain_root(tool_root);
    let state_root = paths::msvc_state_root(tool_root);
    let runtime_state = paths::msvc_state_root(tool_root).join("runtime.json");
    let mut lines = vec![format!(
        "Inspecting managed toolchain under {}",
        toolchain_root.display()
    )];
    if !toolchain_root.exists() {
        return Err(CoreError::Other(format!(
            "managed MSVC toolchain root does not exist: {}",
            toolchain_root.display()
        )));
    }
    if !runtime_state.exists() {
        return Err(CoreError::Other(format!(
            "managed MSVC runtime state is missing: {}",
            runtime_state.display()
        )));
    }
    let installed_state = state_root.join("installed.json");
    if !installed_state.exists() {
        return Err(CoreError::Other(format!(
            "installed MSVC state file is missing: {}",
            installed_state.display()
        )));
    }
    lines.push(format!(
        "Found installed state at {}",
        installed_state.display()
    ));
    lines.push(format!(
        "Found managed runtime state at {}",
        runtime_state.display()
    ));
    Ok((toolchain_root, managed_root, lines))
}

pub fn include_dirs_for_validation(toolchain_root: &Path) -> Vec<PathBuf> {
    let mut dirs = standard_include_dirs_for_validation(toolchain_root);
    dirs.extend(unique_existing_dirs(
        find_all_named_files(
            toolchain_root,
            &[
                "vcruntime.h",
                "yvals.h",
                "sal.h",
                "stdio.h",
                "corecrt.h",
                "Windows.h",
                "winapifamily.h",
            ],
        )
        .into_iter()
        .filter_map(|path| path.parent().map(Path::to_path_buf)),
    ));
    dirs = unique_existing_dirs(dirs);
    dirs.sort_by_key(|path| {
        let lowered = path.display().to_string().to_ascii_lowercase();
        (
            !lowered.contains("\\vc\\tools\\msvc\\"),
            !lowered.contains("\\ucrt"),
            !lowered.contains("\\shared"),
            !lowered.contains("\\um"),
            lowered,
        )
    });
    dirs
}

fn standard_include_dirs_for_validation(toolchain_root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    let vc_tools_root = toolchain_root.join("VC").join("Tools").join("MSVC");
    if let Ok(entries) = fs::read_dir(&vc_tools_root) {
        for entry in entries.flatten() {
            let include = entry.path().join("include");
            if include.is_dir() {
                dirs.push(include);
            }
        }
    }

    let sdk_include_root = toolchain_root
        .join("Windows Kits")
        .join("10")
        .join("Include");
    if let Ok(entries) = fs::read_dir(&sdk_include_root) {
        for entry in entries.flatten() {
            let version_root = entry.path();
            if !version_root.is_dir() {
                continue;
            }
            for segment in ["ucrt", "shared", "um", "winrt", "cppwinrt"] {
                let dir = version_root.join(segment);
                if dir.is_dir() {
                    dirs.push(dir);
                }
            }
        }
    }

    dirs
}

pub fn lib_dirs_for_validation(toolchain_root: &Path, target_arch: &str) -> Vec<PathBuf> {
    let mut dirs = unique_existing_dirs(
        find_all_named_files(toolchain_root, &["kernel32.lib", "ucrt.lib", "libcmt.lib"])
            .into_iter()
            .filter_map(|path| path.parent().map(Path::to_path_buf)),
    )
    .into_iter()
    .filter(|path| is_target_arch_dir(path, target_arch))
    .filter(|path| {
        let components = path_components_lowercase(path);
        !components
            .iter()
            .any(|part| part == "onecore" || part == "enclave" || part == "ucrt_enclave")
    })
    .collect::<Vec<_>>();
    dirs.sort_by_key(|path| {
        let lowered = path.display().to_string().to_ascii_lowercase();
        (
            !lowered.contains("\\vc\\tools\\msvc\\"),
            !lowered.contains("\\ucrt\\"),
            !lowered.contains("\\um\\"),
            lowered,
        )
    });
    dirs
}

pub fn is_target_arch_dir(path: &Path, target_arch: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case(target_arch))
}

fn sanitize_validation_base_path(tool_root: &Path) -> String {
    let legacy_git_usr_bin = paths::scoop_git_usr_bin(tool_root)
        .display()
        .to_string()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_lowercase();
    std::env::var("PATH")
        .unwrap_or_default()
        .split(';')
        .filter(|entry| !entry.trim().is_empty())
        .filter(|entry| {
            entry
                .trim()
                .replace('/', "\\")
                .trim_end_matches('\\')
                .to_ascii_lowercase()
                != legacy_git_usr_bin
        })
        .collect::<Vec<_>>()
        .join(";")
}

pub fn validation_path(tool_root: &Path, extra_path_dirs: &[PathBuf]) -> String {
    let base_path = sanitize_validation_base_path(tool_root);
    if extra_path_dirs.is_empty() {
        base_path
    } else if base_path.is_empty() {
        join_windows_path(extra_path_dirs)
    } else {
        format!("{};{}", join_windows_path(extra_path_dirs), base_path)
    }
}

pub fn write_validation_cpp_template(cpp_root: &Path) -> Result<PathBuf> {
    let source = cpp_root.join("hello.cpp");
    fs::write(&source, VALIDATE_CPP_HELLO_TEMPLATE)
        .map_err(|e| CoreError::fs("write", &source, e))?;
    Ok(source)
}

pub fn write_validation_rust_templates(
    rust_root: &Path,
    options: RustValidationTemplateOptions<'_>,
) -> Result<PathBuf> {
    let rust_src_root = rust_root.join("src");
    let rust_cargo_root = rust_root.join(".cargo");
    let rust_native_root = rust_root.join("native");
    fs::create_dir_all(&rust_src_root)
        .map_err(|e| CoreError::fs("create_dir_all", &rust_src_root, e))?;
    fs::create_dir_all(&rust_cargo_root)
        .map_err(|e| CoreError::fs("create_dir_all", &rust_cargo_root, e))?;
    fs::create_dir_all(&rust_native_root)
        .map_err(|e| CoreError::fs("create_dir_all", &rust_native_root, e))?;

    let rust_source = rust_src_root.join("main.rs");
    fs::write(rust_root.join("Cargo.toml"), VALIDATE_RUST_CARGO_TEMPLATE)
        .map_err(|e| CoreError::fs("write", &rust_root.join("Cargo.toml"), e))?;
    fs::write(rust_root.join("build.rs"), VALIDATE_RUST_BUILD_RS_TEMPLATE)
        .map_err(|e| CoreError::fs("write", &rust_root.join("build.rs"), e))?;
    fs::write(
        &rust_source,
        VALIDATE_RUST_MAIN_TEMPLATE
            .replace("{{VALIDATE_SAMPLE_LABEL}}", options.sample_label)
            .replace(
                "{{VALIDATE_NATIVE_HELPER_LABEL}}",
                options.native_helper_label,
            )
            .replace("{{VALIDATE_LINKER_LABEL}}", options.linker_label),
    )
    .map_err(|e| CoreError::fs("write", &rust_source, e))?;
    fs::write(
        rust_native_root.join("helper.c"),
        VALIDATE_RUST_HELPER_C_TEMPLATE,
    )
    .map_err(|e| CoreError::fs("write", &rust_native_root.join("helper.c"), e))?;
    fs::write(
        rust_cargo_root.join("config.toml"),
        VALIDATE_RUST_CARGO_CONFIG_TEMPLATE.replace(
            "{{SPOON_LINK}}",
            &options.linker.display().to_string().replace('\\', "\\\\"),
        ),
    )
    .map_err(|e| CoreError::fs("write", &rust_cargo_root.join("config.toml"), e))?;
    Ok(rust_source)
}

pub fn locate_cargo() -> Option<PathBuf> {
    which::which("cargo").ok()
}

/// Write build scripts for the managed toolchain validation workspace.
pub fn write_validation_workspace_scripts(
    tool_root: &Path,
    cpp_source: &Path,
    cpp_output: &Path,
    rust_root: &Path,
    cargo_path: Option<&Path>,
) -> Result<()> {
    let cpp_build_script = cpp_source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("build.cmd");
    let rust_build_script = rust_root.join("build.cmd");
    let compiler = wrappers::spoon_cl_wrapper_path(tool_root).display().to_string();
    let cpp_source_name = cpp_source
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("hello.cpp");
    let cpp_output_name = cpp_output
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("hello.exe");
    let cargo = cargo_path
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "cargo".to_string());
    fs::write(
        &cpp_build_script,
        format!(
            concat!(
                "@echo off\r\n",
                "setlocal\r\n",
                "rem validate compile goes through spoon-cl; link is driven by cl /link\r\n",
                "\"{compiler}\" /nologo \"{source}\" user32.lib /link /NOLOGO /OUT:{output}\r\n",
            ),
            compiler = compiler,
            source = cpp_source_name,
            output = cpp_output_name,
        ),
    )
    .map_err(|e| CoreError::Other(format!("failed to write {}: {e}", cpp_build_script.display())))?;
    fs::write(
        &rust_build_script,
        format!(
            concat!(
                "@echo off\r\n",
                "setlocal\r\n",
                "rem validate rust build goes through Cargo + build.rs + spoon-cl\r\n",
                "set \"SPOON_VALIDATE_SPOON_CL={spoon_cl}\"\r\n",
                "\"{cargo}\" build --quiet\r\n",
            ),
            cargo = cargo,
            spoon_cl = wrappers::spoon_cl_wrapper_path(tool_root).display(),
        ),
    )
    .map_err(|e| CoreError::Other(format!("failed to write {}: {e}", rust_build_script.display())))?;
    Ok(())
}
