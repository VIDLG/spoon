use std::path::{Path, PathBuf};

use crate::{BackendError, Result};

use crate::domain::scoop::facts::{
    ResolvedPackageSource, buckets::ResolvedBucket, load_manifest_value, resolve_manifest_sync,
    resolve_package_manifest, resolve_package_source,
};

// Re-export unified types from spoon-scoop.
pub use spoon_scoop::{ScoopPackageAction, ScoopPackagePlan};

#[derive(Debug, Clone)]
pub(crate) struct PlannedPackageLifecycle {
    pub resolved: ResolvedBucket,
    pub source: ResolvedPackageSource,
}

pub fn plan_package_action(
    action: &str,
    display_name: &str,
    package_name: &str,
    tool_root: Option<&Path>,
) -> ScoopPackagePlan {
    let action_kind = ScoopPackageAction::from_str(action);
    let resolved_manifest = tool_root
        .filter(|_| action_kind.should_resolve_manifest())
        .and_then(|root| resolve_manifest_sync(root, package_name));
    ScoopPackagePlan {
        action: action_kind,
        display_name: display_name.to_string(),
        package_name: package_name.to_string(),
        args: action_kind.build_args(package_name, action),
        resolved_manifest,
    }
}

pub(crate) async fn plan_package_lifecycle(
    tool_root: &Path,
    plan: &ScoopPackagePlan,
) -> Result<PlannedPackageLifecycle> {
    let resolved = match plan.resolved_manifest.clone() {
        Some(resolved) => resolved,
        None => resolve_package_manifest(&plan.package_name, tool_root)
            .await
            .ok_or(BackendError::ManifestUnavailable)?,
    };
    let manifest = load_manifest_value(&resolved.manifest_path).await?;
    let source = resolve_package_source(&manifest)?;
    Ok(PlannedPackageLifecycle { resolved, source })
}

pub fn infer_tool_root(explicit_root: Option<&Path>, config_root: Option<&str>) -> Option<PathBuf> {
    explicit_root.map(Path::to_path_buf).or_else(|| {
        let configured = config_root?;
        let trimmed = configured.trim();
        (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
    })
}
