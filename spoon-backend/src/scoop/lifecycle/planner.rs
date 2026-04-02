use std::path::Path;

use crate::{BackendError, Result};

use super::super::buckets::{ResolvedBucket, resolve_manifest};
use super::super::planner::ScoopPackagePlan;
use super::super::host::surface::load_manifest_value;
use super::super::package_source::{SelectedPackageSource, parse_selected_source};

#[derive(Debug, Clone)]
pub(crate) struct PlannedPackageLifecycle {
    pub resolved: ResolvedBucket,
    pub source: SelectedPackageSource,
}

pub(crate) async fn plan_package_lifecycle(
    tool_root: &Path,
    plan: &ScoopPackagePlan,
) -> Result<PlannedPackageLifecycle> {
    let resolved = match plan.resolved_manifest.clone() {
        Some(resolved) => resolved,
        None => resolve_manifest(tool_root, &plan.package_name)
            .await
            .ok_or(BackendError::ManifestUnavailable)?,
    };
    let manifest = load_manifest_value(&resolved.manifest_path).await?;
    let source = parse_selected_source(&manifest)?;
    Ok(PlannedPackageLifecycle {
        resolved,
        source,
    })
}
