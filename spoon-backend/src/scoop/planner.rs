use std::path::{Path, PathBuf};

use crate::{BackendError, Result};

use super::buckets::{ResolvedBucket, resolve_manifest, resolve_manifest_sync};
use super::host::load_manifest_value;
use super::package_source::{SelectedPackageSource, parse_selected_source};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoopPackageAction {
    Install,
    Update,
    Uninstall,
    Reapply,
    Other,
}

#[derive(Debug, Clone)]
pub struct ScoopPackagePlan {
    pub action: ScoopPackageAction,
    pub display_name: String,
    pub package_name: String,
    pub args: Vec<String>,
    pub resolved_manifest: Option<ResolvedBucket>,
}

#[derive(Debug, Clone)]
pub(crate) struct PlannedPackageLifecycle {
    pub resolved: ResolvedBucket,
    pub source: SelectedPackageSource,
}

impl ScoopPackagePlan {
    pub fn title(&self) -> String {
        format!("{} {}", self.action.as_str(), self.display_name)
    }

    pub fn command_line(&self) -> String {
        format!(
            "Planned Spoon package action (Scoop): {}",
            self.args.join(" ")
        )
    }

    pub fn resolution_line(&self) -> Option<String> {
        let resolved = self.resolved_manifest.as_ref()?;
        let branch = if resolved.bucket.branch.trim().is_empty() {
            "master"
        } else {
            &resolved.bucket.branch
        };
        Some(format!(
            "Resolved Scoop package '{}' from bucket '{}' [{branch}] at {}.",
            self.package_name, resolved.bucket.name, resolved.bucket.source
        ))
    }
}

impl ScoopPackageAction {
    pub fn from_str(action: &str) -> Self {
        match action {
            "install" => Self::Install,
            "update" => Self::Update,
            "uninstall" => Self::Uninstall,
            "reapply" => Self::Reapply,
            _ => Self::Other,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Update => "update",
            Self::Uninstall => "uninstall",
            Self::Reapply => "reapply",
            Self::Other => "run",
        }
    }

    fn build_args(self, package_name: &str, fallback_action: &str) -> Vec<String> {
        match self {
            Self::Install => vec![
                "install".to_string(),
                package_name.to_string(),
                "--no-update-scoop".to_string(),
            ],
            Self::Update => vec!["update".to_string(), package_name.to_string()],
            Self::Uninstall => vec!["uninstall".to_string(), package_name.to_string()],
            Self::Reapply => vec!["reapply".to_string(), package_name.to_string()],
            Self::Other => vec![fallback_action.to_string(), package_name.to_string()],
        }
    }

    fn should_resolve_manifest(self) -> bool {
        matches!(self, Self::Install | Self::Update)
    }
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
        None => resolve_manifest(tool_root, &plan.package_name)
            .await
            .ok_or(BackendError::ManifestUnavailable)?,
    };
    let manifest = load_manifest_value(&resolved.manifest_path).await?;
    let source = parse_selected_source(&manifest)?;
    Ok(PlannedPackageLifecycle { resolved, source })
}

pub fn infer_tool_root(explicit_root: Option<&Path>, config_root: Option<&str>) -> Option<PathBuf> {
    explicit_root.map(Path::to_path_buf).or_else(|| {
        let configured = config_root?;
        let trimmed = configured.trim();
        (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
    })
}
