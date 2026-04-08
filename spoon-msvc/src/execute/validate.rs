//! Toolchain validation — compile and run C++/Rust validation samples.

use std::process::Command;

use fs_err as fs;

use crate::paths;
use crate::wrappers;

use super::discover;

pub async fn validate_toolchain_async(
    request: &crate::types::MsvcRequest,
) -> spoon_core::Result<crate::types::MsvcOperationOutcome> {
    let tool_root = request.root.as_path();
    let (toolchain_root, _managed_root, mut lines) =
        crate::validation::validate_toolchain_layout(tool_root)?;

    let flags = discover::managed_toolchain_flags_with_request(request).await?;
    lines.extend(wrappers::ensure_managed_toolchain_wrappers(
        tool_root,
        &request.command_profile,
        &flags,
    )?);
    let cl = wrappers::spoon_cl_wrapper_path(tool_root);
    let link = wrappers::spoon_link_wrapper_path(tool_root);
    let windows_h = crate::common::find_first_named_file(&toolchain_root, &["Windows.h"])
        .ok_or_else(|| {
            spoon_core::CoreError::Other(format!(
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
        return Err(spoon_core::CoreError::Other(format!(
            "no managed include directories were discovered under {}",
            toolchain_root.display()
        )));
    }
    if lib_dirs.is_empty() {
        return Err(spoon_core::CoreError::Other(format!(
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
        .map_err(|err| spoon_core::CoreError::fs("create", &validate_root, err))?;
    lines.push(format!(
        "Prepared validation workspace {}.",
        validate_root.display()
    ));

    let cpp_root = validate_root.join("cpp");
    let rust_root = validate_root.join("rust");
    fs::create_dir_all(&cpp_root)
        .map_err(|err| spoon_core::CoreError::fs("create", &cpp_root, err))?;
    fs::create_dir_all(&rust_root)
        .map_err(|err| spoon_core::CoreError::fs("create", &rust_root, err))?;
    let source = crate::validation::write_validation_cpp_template(&cpp_root)?;
    let output = cpp_root.join("hello.exe");
    crate::validation::write_validation_rust_templates(
        &rust_root,
        crate::validation::RustValidationTemplateOptions {
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
    let cargo = crate::validation::locate_cargo();
    crate::validation::write_validation_workspace_scripts(
        tool_root,
        &source,
        &output,
        &rust_root,
        cargo.as_deref(),
    )?;
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
            .env("INCLUDE", crate::common::join_windows_path(&include_dirs))
            .env("LIB", crate::common::join_windows_path(&lib_dirs))
            .env("PATH", crate::validation::validation_path(tool_root, &path_dirs))
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
            .map_err(|err| spoon_core::CoreError::external(format!("failed to run {}", cl.display()), err))?
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
        return Err(spoon_core::CoreError::Other(format!(
            "managed MSVC validation compile failed in {}\n{}",
            cpp_root.display(),
            details
        )));
    }
    if !output.exists() {
        return Err(spoon_core::CoreError::Other(format!(
            "managed MSVC validation did not produce {}",
            output.display()
        )));
    }

    let run_output = Command::new(&output)
        .current_dir(&cpp_root)
        .output()
        .map_err(|err| {
            spoon_core::CoreError::external(format!("failed to run {}", output.display()), err)
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
        return Err(spoon_core::CoreError::Other(format!(
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
                        .env("SPOON_VALIDATE_SPOON_CL", wrappers::spoon_cl_wrapper_path(tool_root))
                        .arg("build")
                        .arg("--quiet");
                    command.output().map_err(|err| {
                        spoon_core::CoreError::external(
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
                    return Err(spoon_core::CoreError::Other(format!(
                        "managed MSVC validation cargo sample failed in {}\n{}",
                        rust_root.display(),
                        details
                    )));
                }
                if !rust_output.exists() {
                    return Err(spoon_core::CoreError::Other(format!(
                        "managed MSVC validation did not produce {}",
                        rust_output.display()
                    )));
                }
                let rust_run_output = Command::new(&rust_output)
                    .current_dir(&rust_root)
                    .output()
                    .map_err(|err| {
                        spoon_core::CoreError::external(
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
                    return Err(spoon_core::CoreError::Other(format!(
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
    let layout = spoon_core::RuntimeLayout::from_root(tool_root);
    let previous = crate::state::read_canonical_state(&layout);
    let canonical_state = crate::state::MsvcCanonicalState {
        runtime_kind: crate::types::MsvcRuntimeKind::Managed,
        installed: true,
        version: previous.as_ref().and_then(|state| state.version.clone()),
        sdk_version: previous.as_ref().and_then(|state| state.sdk_version.clone()),
        last_operation: Some(crate::types::MsvcOperationKind::Validate),
        last_stage: Some(crate::types::MsvcLifecycleStage::Completed),
        validation_status: Some(crate::types::MsvcValidationStatus::Valid),
        validation_message: Some("validated successfully".to_string()),
        managed: crate::state::ManagedMsvcStateDetail {
            selected_target_arch: Some(request.normalized_target_arch()),
        },
        official: previous.map(|state| state.official).unwrap_or_default(),
    };
    crate::state::write_canonical_state(&layout, &canonical_state)
        .map_err(|e| spoon_core::CoreError::Other(e.to_string()))?;
    Ok(crate::types::MsvcOperationOutcome {
        kind: "msvc_operation",
        runtime: crate::types::MsvcRuntimeKind::Managed,
        operation: crate::types::MsvcOperationKind::Validate,
        title: "validate MSVC Toolchain".to_string(),
        status: true,
    })
}
