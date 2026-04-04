use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tokio::fs;

use crate::Result;
use crate::{BackendError, BackendEvent};
use crate::scoop::{ResolvedPackageSource, ShimTarget};
use super::super::ScoopRuntimeHost;
use super::environment::{resolve_env_add_paths, resolve_env_set_entries};

pub(super) fn shim_target_path(
    current_root: &Path,
    source: &ResolvedPackageSource,
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

    if let Some(asset_target) = source.assets.iter().find_map(|asset| {
        let relative = asset.target_name.as_deref()?;
        let candidate = current_root.join(relative);
        let stem = Path::new(relative)
            .file_stem()
            .and_then(|value| value.to_str());
        (candidate.exists() && stem == Some(target.alias.as_str())).then_some(candidate)
    }) {
        return asset_target;
    }

    let alias_candidate = current_root.join(format!("{}.exe", target.alias));
    if alias_candidate.exists() {
        return alias_candidate;
    }

    primary
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

pub fn expanded_shim_targets(
    package_name: &str,
    current_root: &Path,
    source: &ResolvedPackageSource,
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

pub(crate) async fn write_shims(
    package_name: &str,
    shims_root: &Path,
    install_root: &Path,
    persist_root: &Path,
    source: &ResolvedPackageSource,
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
            return Err(BackendError::MissingShimTarget {
                alias: target.alias.clone(),
                path: executable,
            });
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

pub async fn remove_shims(tool_root: &Path, aliases: &[String]) -> Result<()> {
    let shims_root = crate::layout::RuntimeLayout::from_root(tool_root).shims;
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
