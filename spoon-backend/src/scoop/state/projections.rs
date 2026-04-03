use schemars::JsonSchema;

use crate::layout::RuntimeLayout;

use super::model::InstalledPackageState;
use super::store::list_installed_states;

#[derive(Debug, Clone, serde::Serialize, JsonSchema)]
pub struct InstalledPackageSummary {
    pub name: String,
    pub version: String,
}

/// Project a canonical [`InstalledPackageState`] into a lightweight summary
/// entry suitable for package list / status surfaces.
pub fn installed_package_summary(state: &InstalledPackageState) -> InstalledPackageSummary {
    InstalledPackageSummary {
        name: state.package.clone(),
        version: state.version.trim().to_string(),
    }
}

/// Enumerate all canonical installed states through the store and project
/// each one into a summary entry.
pub async fn list_installed_summaries(
    layout: &RuntimeLayout,
) -> Vec<InstalledPackageSummary> {
    let states = list_installed_states(layout).await;
    let mut summaries: Vec<InstalledPackageSummary> =
        states.iter().map(installed_package_summary).collect();
    summaries.sort_by(|a, b| a.name.cmp(&b.name));
    summaries
}

/// Enumerate all canonical installed states through the store and project
/// each one into a summary entry, applying an optional filter.
pub async fn list_installed_summaries_filtered<F>(
    layout: &RuntimeLayout,
    mut filter: Option<F>,
) -> Vec<InstalledPackageSummary>
where
    F: FnMut(&InstalledPackageState) -> bool,
{
    let mut states = list_installed_states(layout).await;
    if let Some(ref mut f) = filter {
        states.retain(|s| f(s));
    }
    states.sort_by(|a, b| a.package.cmp(&b.package));
    states.iter().map(installed_package_summary).collect()
}

/// Enumerate all canonical installed states through the store, sorted by
/// package name.
pub async fn list_all_installed_states(layout: &RuntimeLayout) -> Vec<InstalledPackageState> {
    let mut states = list_installed_states(layout).await;
    states.sort_by(|a, b| a.package.cmp(&b.package));
    states
}

/// Enumerate all canonical installed states through the store, applying an
/// optional filter, sorted by package name.
pub async fn list_installed_states_filtered<F>(
    layout: &RuntimeLayout,
    mut filter: Option<F>,
) -> Vec<InstalledPackageState>
where
    F: FnMut(&InstalledPackageState) -> bool,
{
    let mut states = list_installed_states(layout).await;
    if let Some(ref mut f) = filter {
        states.retain(|s| f(s));
    }
    states.sort_by(|a, b| a.package.cmp(&b.package));
    states
}
