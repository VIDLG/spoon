use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use mslnk::ShellLink;
use serde_json::Value;
use tokio::fs;

use crate::Result;
use crate::{BackendError, BackendEvent};
use crate::layout::RuntimeLayout;
use crate::scoop::state::{read_installed_state, write_installed_state};

use super::super::manifest;
use super::super::paths;
use super::super::paths::{package_current_root, package_persist_root};
use super::source::{SelectedPackageSource, ShimTarget, ShortcutEntry, parse_selected_source};
use super::{NoopScoopRuntimeHost, ScoopRuntimeHost};

fn start_menu_shortcuts_root(host: &dyn ScoopRuntimeHost) -> Result<PathBuf> {
    if host.test_mode_enabled() || std::env::var_os("SPOON_TEST_HOME").is_some() {
        return Ok(host
            .home_dir()
            .join(".spoon-test-startmenu")
            .join("Spoon Apps"));
    }
    let data_dir = dirs::data_dir().ok_or_else(|| {
        BackendError::Other("failed to resolve Windows data directory".to_string())
    })?;
    Ok(data_dir
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Spoon Apps"))
}

pub async fn load_manifest_value(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .await
        .map_err(|err| BackendError::fs("read", path, err))?;
    serde_json::from_str(&content)
        .map_err(BackendError::from)
        .map_err(|err| err.context(format!("invalid manifest {}", path.display())))
}

fn shim_target_path(
    current_root: &Path,
    source: &SelectedPackageSource,
    target: &ShimTarget,
) -> PathBuf {
    let primary = current_root.join(&target.relative_path);
    if primary.exists() {
        return primary;
    }

    if let Some(shortcut_target) = source.shortcuts.iter().find_map(|shortcut| {
        let candidate = current_root.join(&shortcut.target_path);
        let stem = Path::new(&shortcut.target_path)
            .file_stem()
            .and_then(|value| value.to_str());
        (candidate.exists() && stem == Some(target.alias.as_str())).then_some(candidate)
    }) {
        return shortcut_target;
    }

    if let Some(payload_target) = source.payloads.iter().find_map(|payload| {
        let relative = payload.target_name.as_deref()?;
        let candidate = current_root.join(relative);
        let stem = Path::new(relative)
            .file_stem()
            .and_then(|value| value.to_str());
        (candidate.exists() && stem == Some(target.alias.as_str())).then_some(candidate)
    }) {
        return payload_target;
    }

    let alias_candidate = current_root.join(format!("{}.exe", target.alias));
    if alias_candidate.exists() {
        return alias_candidate;
    }

    primary
}

pub fn expanded_shim_targets(
    package_name: &str,
    current_root: &Path,
    source: &SelectedPackageSource,
    host: &dyn ScoopRuntimeHost,
) -> Vec<ShimTarget> {
    let mut targets = source.bins.clone();
    let mut seen_aliases = targets
        .iter()
        .map(|target| target.alias.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();

    let inferred_canonical = targets
        .iter()
        .filter_map(|target| {
            let canonical = Path::new(&target.relative_path)
                .file_stem()
                .and_then(|value| value.to_str())?
                .to_string();
            if canonical.eq_ignore_ascii_case(&target.alias) {
                return None;
            }
            let canonical_key = canonical.to_ascii_lowercase();
            if seen_aliases.contains(&canonical_key) {
                return None;
            }
            let canonical_target = ShimTarget {
                relative_path: target.relative_path.clone(),
                alias: canonical.clone(),
                args: target.args.clone(),
            };
            let executable = shim_target_path(current_root, source, &canonical_target);
            executable
                .exists()
                .then_some((canonical_key, canonical_target))
        })
        .collect::<Vec<_>>();

    for (alias_key, target) in inferred_canonical {
        seen_aliases.insert(alias_key);
        targets.push(target);
    }

    for (alias_key, target) in built_in_package_supplemental_shims(package_name, current_root, host)
    {
        if seen_aliases.insert(alias_key) {
            targets.push(target);
        }
    }

    targets
}

fn built_in_package_supplemental_shims(
    package_name: &str,
    current_root: &Path,
    host: &dyn ScoopRuntimeHost,
) -> Vec<(String, ShimTarget)> {
    host.supplemental_shims(package_name, current_root)
        .into_iter()
        .map(|spec| {
            (
                spec.alias.to_ascii_lowercase(),
                ShimTarget {
                    relative_path: spec.relative_path,
                    alias: spec.alias,
                    args: Vec::new(),
                },
            )
        })
        .collect()
}

pub fn installed_targets_exist(
    package_name: &str,
    current_root: &Path,
    source: &SelectedPackageSource,
    host: &dyn ScoopRuntimeHost,
) -> bool {
    expanded_shim_targets(package_name, current_root, source, host)
        .iter()
        .any(|target| shim_target_path(current_root, source, target).exists())
        || source
            .shortcuts
            .iter()
            .any(|shortcut| current_root.join(&shortcut.target_path).exists())
}

pub fn installer_layout_error(current_root: &Path, source: &SelectedPackageSource) -> BackendError {
    let expected_bins = source
        .bins
        .iter()
        .map(|target| target.relative_path.clone())
        .collect::<Vec<_>>();
    let expected_shortcuts = source
        .shortcuts
        .iter()
        .map(|entry| entry.target_path.clone())
        .collect::<Vec<_>>();
    BackendError::Other(format!(
        "installer actions completed but no declared bin/shortcut targets materialized under {} (expected bins: {}; expected shortcuts: {})",
        current_root.display(),
        if expected_bins.is_empty() {
            "-".to_string()
        } else {
            expected_bins.join(", ")
        },
        if expected_shortcuts.is_empty() {
            "-".to_string()
        } else {
            expected_shortcuts.join(", ")
        }
    ))
}

pub(crate) async fn write_shims(
    package_name: &str,
    shims_root: &Path,
    install_root: &Path,
    persist_root: &Path,
    source: &SelectedPackageSource,
    host: &dyn ScoopRuntimeHost,
    _emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    fs::create_dir_all(shims_root)
        .await
        .map_err(|err| BackendError::fs("create", shims_root, err))?;
    let mut aliases = Vec::new();
    let shim_targets = expanded_shim_targets(package_name, install_root, source, host);
    let env_add_paths = resolve_env_add_paths(source, install_root);
    let env_set_entries = resolve_env_set_entries(source, install_root, persist_root);

    for target in &shim_targets {
        let shim_path = shims_root.join(format!("{}.cmd", target.alias));
        let executable = shim_target_path(install_root, source, target);
        if !executable.exists() {
            return Err(BackendError::Other(format!(
                "refusing to write shim '{}' because target was missing: {}",
                target.alias,
                executable.display()
            )));
        }
        let extra_args = if target.args.is_empty() {
            String::new()
        } else {
            format!(" {}", target.args.join(" "))
        };
        let mut lines = vec!["@echo off".to_string(), "setlocal".to_string()];
        if !env_add_paths.is_empty() {
            let joined = env_add_paths
                .iter()
                .map(|path| path.display().to_string().replace('/', "\\"))
                .collect::<Vec<_>>()
                .join(";");
            lines.push(format!("set \"PATH={joined};%PATH%\""));
        }
        for (key, value) in &env_set_entries {
            lines.push(format!("set \"{}={}\"", key, value.replace('/', "\\")));
        }
        lines.push(format!(
            "\"{}\"{} %*",
            executable.display().to_string().replace('/', "\\"),
            extra_args
        ));
        lines.push("set \"SPOON_EXIT=%ERRORLEVEL%\"".to_string());
        lines.push("endlocal & exit /b %SPOON_EXIT%".to_string());
        let content = format!("{}\r\n", lines.join("\r\n"));
        fs::write(&shim_path, content)
            .await
            .map_err(|err| BackendError::fs("write", &shim_path, err))?;
        tracing::info!("Wrote shim '{}': {}", target.alias, shim_path.display());
        aliases.push(target.alias.clone());
    }
    Ok(aliases)
}

pub async fn reapply_package_command_surface_streaming_with_host(
    tool_root: &Path,
    package_name: &str,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    let layout = RuntimeLayout::from_root(tool_root);
    let Some(mut state) = read_installed_state(&layout, package_name).await else {
        return Ok(vec![format!(
            "Skipped command-surface reapply for '{}': installed state was not found.",
            package_name
        )]);
    };
    let current_root = package_current_root(tool_root, package_name);
    if !current_root.exists() {
        return Ok(vec![format!(
            "Skipped command-surface reapply for '{}': current install root was not found.",
            package_name
        )]);
    }
    let resolved = manifest::resolve_package_manifest(package_name, tool_root)
        .await
        .ok_or_else(|| BackendError::Other("package manifest could not be resolved".to_string()))?;
    let manifest = load_manifest_value(&resolved.manifest_path).await?;
    let source = parse_selected_source(&manifest)?;
    remove_shims(tool_root, &state.bins).await?;
    let shims_root = paths::shims_root(tool_root);
    let persist_root = package_persist_root(tool_root, package_name);
    let aliases = write_shims(
        package_name,
        &shims_root,
        &current_root,
        &persist_root,
        &source,
        host,
        emit,
    )
    .await?;
    state.bins = aliases.clone();
    state.env_add_path = source.env_add_path.clone();
    state.env_set = source.env_set.clone();
    write_installed_state(&layout, &state).await?;
    let mut output = vec![format!("Reapplied command surface for '{}'.", package_name)];
    output.push(format!("Managed shims: {}", aliases.join(", ")));
    Ok(output)
}

pub async fn reapply_package_command_surface_streaming(
    tool_root: &Path,
    package_name: &str,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    let host = NoopScoopRuntimeHost;
    reapply_package_command_surface_streaming_with_host(tool_root, package_name, &host, emit).await
}

fn substitute_shortcut_text(value: &str, current_root: &Path, persist_root: &Path) -> String {
    value
        .replace("$dir", &current_root.display().to_string())
        .replace("$persist_dir", &persist_root.display().to_string())
        .replace("$original_dir", &current_root.display().to_string())
}

fn resolve_env_add_paths(source: &SelectedPackageSource, install_root: &Path) -> Vec<PathBuf> {
    source
        .env_add_path
        .iter()
        .filter_map(|entry| {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                None
            } else if trimmed == "." {
                Some(install_root.to_path_buf())
            } else {
                Some(install_root.join(trimmed))
            }
        })
        .collect()
}

fn resolve_env_set_entries(
    source: &SelectedPackageSource,
    install_root: &Path,
    persist_root: &Path,
) -> Vec<(String, String)> {
    source
        .env_set
        .iter()
        .map(|(key, value)| {
            (
                key.clone(),
                substitute_shortcut_text(value, install_root, persist_root),
            )
        })
        .collect()
}

pub(crate) async fn write_shortcuts(
    install_root: &Path,
    persist_root: &Path,
    source: &SelectedPackageSource,
    host: &dyn ScoopRuntimeHost,
    _emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<ShortcutEntry>> {
    if source.shortcuts.is_empty() {
        return Ok(Vec::new());
    }
    let shortcuts_root = start_menu_shortcuts_root(host)?;
    fs::create_dir_all(&shortcuts_root).await.map_err(|err| {
        BackendError::Other(format!(
            "failed to create {}: {err}",
            shortcuts_root.display()
        ))
    })?;

    let mut created = Vec::new();
    for shortcut in &source.shortcuts {
        let target = install_root.join(&shortcut.target_path);
        if !target.exists() {
            tracing::warn!(
                "Skipped shortcut '{}' because target was missing: {}",
                shortcut.name,
                target.display()
            );
            continue;
        }
        let shortcut_path = shortcuts_root.join(&shortcut.name).with_extension("lnk");
        if let Some(parent) = shortcut_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|err| BackendError::fs("create", parent, err))?;
        }
        let args = shortcut
            .args
            .as_ref()
            .map(|value| substitute_shortcut_text(value, install_root, persist_root));
        let icon = shortcut
            .icon_path
            .as_ref()
            .map(|value| install_root.join(value));
        if let Some(icon_path) = &icon
            && !icon_path.exists()
        {
            tracing::warn!(
                "Skipped shortcut icon for '{}' because it was missing: {}",
                shortcut.name,
                icon_path.display()
            );
        }
        let target_owned = target.clone();
        let shortcut_path_owned = shortcut_path.clone();
        let args_owned = args.clone();
        let icon_owned = icon.clone().filter(|path| path.exists());
        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut link = ShellLink::new(&target_owned).map_err(|err| {
                BackendError::external(
                    format!(
                        "failed to create shortcut target for {}",
                        target_owned.display()
                    ),
                    err,
                )
            })?;
            if let Some(arguments) = args_owned {
                link.set_arguments(Some(arguments));
            }
            if let Some(icon_path) = icon_owned {
                link.set_icon_location(Some(icon_path.display().to_string()));
            }
            link.create_lnk(&shortcut_path_owned).map_err(|err| {
                BackendError::external(
                    format!("failed to write {}", shortcut_path_owned.display()),
                    err,
                )
            })?;
            Ok(())
        })
        .await
        .map_err(|err| BackendError::external("shortcut creation join failed", err))??;
        tracing::info!(
            "Created shortcut '{}': {}",
            shortcut.name,
            shortcut_path.display()
        );
        created.push(shortcut.clone());
    }
    Ok(created)
}

pub async fn remove_shims(tool_root: &Path, aliases: &[String]) -> Result<()> {
    let shims_root = paths::shims_root(tool_root);
    for alias in aliases {
        let path = shims_root.join(format!("{alias}.cmd"));
        if path.exists() {
            fs::remove_file(&path)
                .await
                .map_err(|err| BackendError::fs("remove", &path, err))?;
        }
    }
    Ok(())
}

pub async fn remove_shortcuts(
    entries: &[ShortcutEntry],
    host: &dyn ScoopRuntimeHost,
) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    let shortcuts_root = start_menu_shortcuts_root(host)?;
    for shortcut in entries {
        let path = shortcuts_root.join(&shortcut.name).with_extension("lnk");
        if path.exists() {
            fs::remove_file(&path)
                .await
                .map_err(|err| BackendError::fs("remove", &path, err))?;
        }
    }
    Ok(())
}
