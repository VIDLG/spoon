use std::path::Path;

use crate::Result;
use crate::event::{LifecycleStage, StageEvent};
use crate::BackendEvent;

use super::super::buckets;
use super::super::planner::{ScoopPackageAction, ScoopPackagePlan};

pub(super) fn emit_stage(emit: &mut dyn FnMut(BackendEvent), stage: LifecycleStage) {
    let event = if matches!(stage, LifecycleStage::Completed) {
        StageEvent::completed(stage)
    } else {
        StageEvent::started(stage)
    };
    emit(BackendEvent::Stage(event));
}

pub(super) async fn effective_runtime_plan(
    tool_root: &Path,
    proxy: &str,
    plan: &ScoopPackagePlan,
) -> Result<ScoopPackagePlan> {
    if !matches!(
        plan.action,
        ScoopPackageAction::Install | ScoopPackageAction::Update
    ) {
        return Ok(plan.clone());
    }
    if buckets::load_buckets_from_registry(tool_root).await.is_empty() {
        buckets::ensure_main_bucket_ready(tool_root, proxy).await?;
    }
    if plan.resolved_manifest.is_some() {
        return Ok(plan.clone());
    }
    Ok(super::super::planner::plan_package_action(
        plan.action.as_str(),
        &plan.display_name,
        &plan.package_name,
        Some(tool_root),
    ))
}
