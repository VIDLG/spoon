use std::path::{Path, PathBuf};
use std::process::Command;

use fs_err as fs;

use super::common::{
    find_all_named_files, find_first_named_file, join_windows_path, path_components_lowercase,
    unique_existing_dirs,
};
use super::paths;
use super::wrappers::{
    ensure_managed_toolchain_wrappers, spoon_cl_wrapper_path, spoon_link_wrapper_path,
};
use super::{
    MsvcLifecycleStage, MsvcOperationKind, MsvcRuntimeKind, MsvcValidationStatus,
    ManagedMsvcStateDetail, is_target_arch_dir, msvc_dir, read_canonical_state, runtime_state_path,
    write_canonical_state,
};
use crate::BackendContext;
use crate::CommandStatus;
use crate::{BackendError, Result};

const VALIDATE_CPP_HELLO_TEMPLATE: &str = include_str!("validate_templates/cpp/hello.cpp");
const VALIDATE_RUST_CARGO_TEMPLATE: &str = include_str!("validate_templates/rust/Cargo.toml");
const VALIDATE_RUST_BUILD_RS_TEMPLATE: &str = include_str!("validate_templates/rust/build.rs");
const VALIDATE_RUST_MAIN_TEMPLATE: &str = include_str!("validate_templates/rust/src/main.rs");
const VALIDATE_RUST_HELPER_C_TEMPLATE: &str =
    include_str!("validate_templates/rust/native/helper.c");
const VALIDATE_RUST_CARGO_CONFIG_TEMPLATE: &str =
    include_str!("validate_templates/rust/.cargo/config.toml");

pub(super) struct RustValidationTemplateOptions<'a> {
    pub linker: &'a Path,
    pub sample_label: &'a str,
    pub native_helper_label: &'a str,
    pub linker_label: &'a str,
}

pub async fn validate_toolchain(tool_root: &Path) -> Result<super::MsvcOperationOutcome> {
    let request = super::MsvcRequest::for_tool_root(tool_root);
    validate_toolchain_with_request(&request).await
}

pub async fn validate_toolchain_with_context<P>(
    context: &BackendContext<P>,
) -> Result<super::MsvcOperationOutcome> {
    let request = super::MsvcRequest::from_context(context);
    validate_toolchain_with_request(&request).await
}

pub(crate) async fn validate_toolchain_with_request(
    request: &super::MsvcRequest,
) -> Result<super::MsvcOperationOutcome> {
    let tool_root = request.root.as_path();
    let (toolchain_root, _managed_root, mut lines) = validate_toolchain_layout(tool_root)?;

    let command_profile = request.command_profile.as_str();
    let flags = super::wrappers::managed_toolchain_flags_with_request(request).await?;
    lines.extend(ensure_managed_toolchain_wrappers(
        tool_root,
        command_profile,
        &flags,
    )?);
    let cl = spoon_cl_wrapper_path(tool_root);
    let link = spoon_link_wrapper_path(tool_root);
    let windows_h = find_first_named_file(&toolchain_root, &["Windows.h"]).ok_or_else(|| {
        BackendError::Other(format!(
            "Windows SDK header Windows.h was not found under {}",
            toolchain_root.display()
        ))
    })?;

    lines.push(format!(
        "Using managed wrapper compiler at {}",
        cl.display()
    ));
    lines.push(format!(
            "Prepared managed wrapper linker at {} (available for direct downstream use; validate link runs via cl /link)",
            link.display()
        ));
    lines.push(format!(
        "Found Windows SDK header at {}",
        windows_h.display()
    ));

    let include_dirs = flags.include_dirs.clone();
    let lib_dirs = flags.lib_dirs.clone();
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

    lines.push(format!(
        "Discovered {} managed include directories.",
        include_dirs.len()
    ));
    lines.push(format!(
        "Discovered {} managed library directories.",
        lib_dirs.len()
    ));

    let validate_root = paths::msvc_cache_root(tool_root).join("validate");
    if validate_root.exists() {
        let _ = fs::remove_dir_all(&validate_root);
    }
    fs::create_dir_all(&validate_root)
        .map_err(|err| BackendError::fs("create", &validate_root, err))?;
    lines.push(format!(
        "Prepared validation workspace {}.",
        validate_root.display()
    ));

    let cpp_root = validate_root.join("cpp");
    let rust_root = validate_root.join("rust");
    fs::create_dir_all(&cpp_root).map_err(|err| BackendError::fs("create", &cpp_root, err))?;
    fs::create_dir_all(&rust_root).map_err(|err| BackendError::fs("create", &rust_root, err))?;
    let source = write_validation_cpp_template(&cpp_root)?;
    let output = cpp_root.join("hello.exe");
    write_validation_rust_templates(
        &rust_root,
        RustValidationTemplateOptions {
            linker: &link,
            sample_label: "spoon msvc validate rust",
            native_helper_label: "spoon-cl",
            linker_label: "spoon-link",
        },
    )?;
    let rust_output = rust_root
        .join("target")
        .join("debug")
        .join("hello-rust.exe");
    let cargo = locate_cargo();
    write_validation_workspace_scripts(tool_root, &source, &output, &rust_root, cargo.as_deref())?;
    lines.push(format!(
        "Wrote reusable validation scripts under {}.",
        validate_root.display()
    ));

    let path_dirs = flags.path_dirs.clone();
    let compile_output = {
        let mut command = Command::new("cmd.exe");
        command.arg("/C").arg(&cl);
        command
            .current_dir(&cpp_root)
            .env("INCLUDE", join_windows_path(&include_dirs))
            .env("LIB", join_windows_path(&lib_dirs))
            .env("PATH", validation_path(tool_root, &path_dirs))
            .args([
                "/nologo",
                "hello.cpp",
                "user32.lib",
                "/link",
                "/NOLOGO",
                "/OUT:hello.exe",
            ]);
        command
            .output()
            .map_err(|err| BackendError::external(format!("failed to run {}", cl.display()), err))?
    };

    let stdout_lines = String::from_utf8_lossy(&compile_output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let stderr_lines = String::from_utf8_lossy(&compile_output.stderr)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    lines.extend(stdout_lines);
    lines.extend(stderr_lines);

    if !compile_output.status.success() {
        let details = if lines.is_empty() {
            "no compiler output captured".to_string()
        } else {
            lines.join("\n")
        };
        return Err(BackendError::Other(format!(
            "managed MSVC validation compile failed in {}\n{}",
            cpp_root.display(),
            details
        )));
    }
    if !output.exists() {
        return Err(BackendError::Other(format!(
            "managed MSVC validation did not produce {}",
            output.display()
        )));
    }

    let run_output = Command::new(&output)
        .current_dir(&cpp_root)
        .output()
        .map_err(|err| {
            BackendError::external(format!("failed to run {}", output.display()), err)
        })?;
    let run_stdout_lines = String::from_utf8_lossy(&run_output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let run_stderr_lines = String::from_utf8_lossy(&run_output.stderr)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    lines.push("C++ validation:".to_string());
    lines.push("  Compiled managed C++/Win32 validation sample successfully.".to_string());
    lines.extend(run_stdout_lines.into_iter().map(|line| format!("  {line}")));
    lines.extend(run_stderr_lines.into_iter().map(|line| format!("  {line}")));
    if !run_output.status.success() {
        let details = if lines.is_empty() {
            "no runtime output captured".to_string()
        } else {
            lines.join("\n")
        };
        return Err(BackendError::Other(format!(
            "managed MSVC validation sample exited with status {:?}\n{}",
            run_output.status.code(),
            details
        )));
    }
    lines.push(format!(
        "  Ran managed validation sample successfully from {}.",
        output.display()
    ));
    lines.push(format!(
        "  Compiled managed C++/Win32 validation sample successfully into {}.",
        output.display()
    ));
    match cargo {
        Some(cargo_path) => {
            lines.push("Rust validation:".to_string());
            lines.push(format!("  Using Cargo at {}", cargo_path.display()));
            if request.test_mode {
                lines.push(
                        "  Skipped managed Rust validation execution in test mode; generated Cargo sample and build script for inspection."
                            .to_string(),
                    );
            } else {
                let rust_compile_output = {
                    let mut command = Command::new(&cargo_path);
                    command
                        .current_dir(&rust_root)
                        .env("SPOON_VALIDATE_SPOON_CL", spoon_cl_wrapper_path(tool_root))
                        .arg("build")
                        .arg("--quiet");
                    command.output().map_err(|err| {
                        BackendError::external(
                            format!("failed to run {}", cargo_path.display()),
                            err,
                        )
                    })?
                };
                let rust_stdout_lines = String::from_utf8_lossy(&rust_compile_output.stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                let rust_stderr_lines = String::from_utf8_lossy(&rust_compile_output.stderr)
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                lines.push(
                    "  Compiled managed Rust/Cargo validation sample successfully.".to_string(),
                );
                lines.extend(
                    rust_stdout_lines
                        .into_iter()
                        .map(|line| format!("  {line}")),
                );
                lines.extend(
                    rust_stderr_lines
                        .into_iter()
                        .map(|line| format!("  {line}")),
                );
                if !rust_compile_output.status.success() {
                    let details = if lines.is_empty() {
                        "no rust compiler output captured".to_string()
                    } else {
                        lines.join("\n")
                    };
                    return Err(BackendError::Other(format!(
                        "managed MSVC validation cargo sample failed in {}\n{}",
                        rust_root.display(),
                        details
                    )));
                }
                if !rust_output.exists() {
                    return Err(BackendError::Other(format!(
                        "managed MSVC validation did not produce {}",
                        rust_output.display()
                    )));
                }
                let rust_run_output = Command::new(&rust_output)
                    .current_dir(&rust_root)
                    .output()
                    .map_err(|err| {
                        BackendError::external(
                            format!("failed to run {}", rust_output.display()),
                            err,
                        )
                    })?;
                let rust_run_stdout_lines = String::from_utf8_lossy(&rust_run_output.stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                let rust_run_stderr_lines = String::from_utf8_lossy(&rust_run_output.stderr)
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                lines.extend(
                    rust_run_stdout_lines
                        .into_iter()
                        .map(|line| format!("  {line}")),
                );
                lines.extend(
                    rust_run_stderr_lines
                        .into_iter()
                        .map(|line| format!("  {line}")),
                );
                if !rust_run_output.status.success() {
                    let details = if lines.is_empty() {
                        "no rust runtime output captured".to_string()
                    } else {
                        lines.join("\n")
                    };
                    return Err(BackendError::Other(format!(
                        "managed MSVC validation rust sample exited with status {:?}\n{}",
                        rust_run_output.status.code(),
                        details
                    )));
                }
                lines.push(format!(
                    "  Ran managed Rust validation sample successfully from {}.",
                    rust_output.display()
                ));
                lines.push(format!(
                    "  Compiled managed Rust/Cargo validation sample successfully into {}.",
                    rust_output.display()
                ));
            }
        }
        None => {
            lines.push("Rust validation:".to_string());
            lines.push(
                "  Skipped managed Rust validation sample because cargo is not available on PATH."
                    .to_string(),
            );
        }
    }
    lines.push(format!(
        "Kept validation workspace {} for inspection.",
        validate_root.display()
    ));
    let layout = crate::layout::RuntimeLayout::from_root(tool_root);
    let previous = read_canonical_state(&layout).await;
    let canonical_state = super::MsvcCanonicalState {
        runtime_kind: MsvcRuntimeKind::Managed,
        installed: true,
        version: previous.as_ref().and_then(|state| state.version.clone()),
        sdk_version: previous.as_ref().and_then(|state| state.sdk_version.clone()),
        last_operation: Some(MsvcOperationKind::Validate),
        last_stage: Some(MsvcLifecycleStage::Completed),
        validation_status: Some(MsvcValidationStatus::Valid),
        validation_message: Some("validated successfully".to_string()),
        managed: ManagedMsvcStateDetail {
            selected_target_arch: Some(request.normalized_target_arch()),
        },
        official: previous.map(|state| state.official).unwrap_or_default(),
    };
    write_canonical_state(&layout, &canonical_state).await?;
    Ok(super::MsvcOperationOutcome {
        kind: "msvc_operation",
        runtime: super::MsvcRuntimeKind::Managed,
        operation: super::MsvcOperationKind::Validate,
        title: "validate MSVC Toolchain".to_string(),
        status: CommandStatus::Success,
        output: lines,
        streamed: false,
    })
}

pub(super) fn validate_toolchain_layout(
    tool_root: &Path,
) -> Result<(PathBuf, PathBuf, Vec<String>)> {
    let managed_root = paths::msvc_root(tool_root);
    let toolchain_root = msvc_dir(tool_root);
    let state_root = paths::msvc_state_root(tool_root);
    let runtime_state = runtime_state_path(tool_root);
    let mut lines = vec![format!(
        "Inspecting managed toolchain under {}",
        toolchain_root.display()
    )];
    if !toolchain_root.exists() {
        return Err(BackendError::Other(format!(
            "managed MSVC toolchain root does not exist: {}",
            toolchain_root.display()
        )));
    }
    if !runtime_state.exists() {
        return Err(BackendError::Other(format!(
            "managed MSVC runtime state is missing: {}",
            runtime_state.display()
        )));
    }
    let installed_state = state_root.join("installed.json");
    if !installed_state.exists() {
        return Err(BackendError::Other(format!(
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

pub(super) fn include_dirs_for_validation(toolchain_root: &Path) -> Vec<PathBuf> {
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

pub(super) fn lib_dirs_for_validation(toolchain_root: &Path, target_arch: &str) -> Vec<PathBuf> {
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

fn validation_path(tool_root: &Path, extra_path_dirs: &[PathBuf]) -> String {
    let base_path = sanitize_validation_base_path(tool_root);
    if extra_path_dirs.is_empty() {
        base_path
    } else if base_path.is_empty() {
        join_windows_path(extra_path_dirs)
    } else {
        format!("{};{}", join_windows_path(extra_path_dirs), base_path)
    }
}

pub(super) fn write_validation_cpp_template(cpp_root: &Path) -> Result<PathBuf> {
    let source = cpp_root.join("hello.cpp");
    fs::write(&source, VALIDATE_CPP_HELLO_TEMPLATE)
        .map_err(|err| BackendError::fs("write", &source, err))?;
    Ok(source)
}

pub(super) fn write_validation_rust_templates(
    rust_root: &Path,
    options: RustValidationTemplateOptions<'_>,
) -> Result<PathBuf> {
    let rust_src_root = rust_root.join("src");
    let rust_cargo_root = rust_root.join(".cargo");
    let rust_native_root = rust_root.join("native");
    fs::create_dir_all(&rust_src_root)
        .map_err(|err| BackendError::fs("create", &rust_src_root, err))?;
    fs::create_dir_all(&rust_cargo_root)
        .map_err(|err| BackendError::fs("create", &rust_cargo_root, err))?;
    fs::create_dir_all(&rust_native_root)
        .map_err(|err| BackendError::fs("create", &rust_native_root, err))?;

    let rust_source = rust_src_root.join("main.rs");
    fs::write(rust_root.join("Cargo.toml"), VALIDATE_RUST_CARGO_TEMPLATE)
        .map_err(|err| BackendError::fs("write", &rust_root.join("Cargo.toml"), err))?;
    fs::write(rust_root.join("build.rs"), VALIDATE_RUST_BUILD_RS_TEMPLATE)
        .map_err(|err| BackendError::fs("write", &rust_root.join("build.rs"), err))?;
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
    .map_err(|err| BackendError::fs("write", &rust_source, err))?;
    fs::write(
        rust_native_root.join("helper.c"),
        VALIDATE_RUST_HELPER_C_TEMPLATE,
    )
    .map_err(|err| BackendError::fs("write", &rust_native_root.join("helper.c"), err))?;
    fs::write(
        rust_cargo_root.join("config.toml"),
        VALIDATE_RUST_CARGO_CONFIG_TEMPLATE.replace(
            "{{SPOON_LINK}}",
            &options.linker.display().to_string().replace('\\', "\\\\"),
        ),
    )
    .map_err(|err| BackendError::fs("write", &rust_cargo_root.join("config.toml"), err))?;
    Ok(rust_source)
}

pub(super) fn locate_cargo() -> Option<PathBuf> {
    which::which("cargo").ok()
}

fn write_validation_workspace_scripts(
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
    let compiler = spoon_cl_wrapper_path(tool_root).display().to_string();
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
    .map_err(|err| {
        BackendError::Other(format!(
            "failed to write {}: {err}",
            cpp_build_script.display()
        ))
    })?;
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
            spoon_cl = spoon_cl_wrapper_path(tool_root).display(),
        ),
    )
    .map_err(|err| {
        BackendError::Other(format!(
            "failed to write {}: {err}",
            rust_build_script.display()
        ))
    })?;
    Ok(())
}
