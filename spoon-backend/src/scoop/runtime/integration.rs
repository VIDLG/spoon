use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::BackendEvent;
use crate::Result;
use crate::layout::RuntimeLayout;
use crate::scoop::state::{read_installed_state, write_installed_state};

use super::super::buckets;
use super::super::paths::{package_current_root, package_persist_root};
use super::{NoopScoopRuntimeHost, ScoopRuntimeHost, parse_selected_source};

pub fn helper_executable_path(tool_root: &Path, package_name: &str) -> Option<PathBuf> {
    let current_root = package_current_root(tool_root, package_name);
    let direct = current_root.join(format!("{package_name}.exe"));
    if direct.exists() {
        return Some(direct);
    }
    let resolved = tokio::runtime::Handle::current()
        .block_on(buckets::resolve_manifest(tool_root, package_name))?;
    let manifest = std::fs::read_to_string(&resolved.manifest_path).ok()?;
    let manifest: Value = serde_json::from_str(&manifest).ok()?;
    let source = parse_selected_source(&manifest).ok()?;
    source
        .bins
        .first()
        .map(|target| current_root.join(&target.relative_path))
        .filter(|path| path.exists())
}

pub fn resolved_pip_mirror_url_for_display_with_host(
    host: &dyn ScoopRuntimeHost,
    policy_value: &str,
) -> String {
    host.resolved_pip_mirror_url_for_display(policy_value)
}

pub fn resolved_pip_mirror_url_for_display(policy_value: &str) -> String {
    let host = NoopScoopRuntimeHost;
    resolved_pip_mirror_url_for_display_with_host(&host, policy_value)
}

pub async fn apply_package_integrations(
    host: &dyn ScoopRuntimeHost,
    package_name: &str,
    current_root: &Path,
    persist_root: &Path,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<BTreeMap<String, String>> {
    host.apply_integrations(package_name, current_root, persist_root, emit)
        .await
}

pub async fn reapply_package_integrations_streaming_with_host(
    tool_root: &Path,
    package_name: &str,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    let layout = RuntimeLayout::from_root(tool_root);
    let Some(mut state) = read_installed_state(&layout, package_name).await else {
        return Ok(vec![format!(
            "Skipped integration reapply for '{}': installed state was not found.",
            package_name
        )]);
    };
    let current_root = package_current_root(tool_root, package_name);
    if !current_root.exists() {
        return Ok(vec![format!(
            "Skipped integration reapply for '{}': current install root was not found.",
            package_name
        )]);
    }
    let persist_root = package_persist_root(tool_root, package_name);
    let integrations =
        apply_package_integrations(host, package_name, &current_root, &persist_root, emit).await?;
    state.integrations = integrations.clone();
    write_installed_state(&layout, &state).await?;
    let mut output = vec![format!("Reapplied integrations for '{}'.", package_name)];
    if integrations.is_empty() {
        output.push("No package integrations were applied.".to_string());
    } else {
        output.extend(
            integrations
                .iter()
                .map(|(key, value)| format!("Applied integration: {key} = {value}")),
        );
    }
    Ok(output)
}

pub async fn reapply_package_integrations_streaming(
    tool_root: &Path,
    package_name: &str,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    let host = NoopScoopRuntimeHost;
    reapply_package_integrations_streaming_with_host(tool_root, package_name, &host, emit).await
}
