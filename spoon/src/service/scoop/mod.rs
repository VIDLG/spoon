mod actions;
mod bucket;
mod report;
pub mod runtime;

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use super::{CommandResult, CommandStatus};
pub(crate) use super::{ConfigEntry, desired_policy_entries};
pub use actions::{install_tools, package_action_result, uninstall_tools, update_tools};
pub(crate) use actions::{
    install_tools_streaming, run_package_action_streaming, uninstall_tools_streaming,
    update_tools_streaming,
};
pub(crate) use bucket::bucket_update_with_emit;
pub use bucket::{
    RepoSyncOutcome, bucket_action_result, bucket_add, bucket_inventory, bucket_list_report,
    bucket_list_report_lines, bucket_remove, bucket_update, doctor_report, doctor_summary,
    doctor_summary_lines,
};
pub use report::{
    package_info_report, package_info_report_lines, package_list_report,
    package_list_report_lines, package_manifest, package_manifest_lines,
    package_prefix_report, package_prefix_report_lines, runtime_status_report,
    runtime_status_report_lines, search_report, search_report_lines,
};

pub use spoon_scoop::ensure_main_bucket_ready;
pub use spoon_scoop::{
    latest_version, latest_version_async, load_manifest,
    load_manifest_sync, load_package_manifest, load_package_manifest_sync, parse_manifest,
    resolve_manifest, upsert_bucket_to_registry,
};
pub use spoon_scoop::{BucketSpec, known_bucket_source};

pub(crate) use spoon_scoop::{
    ScoopBucketOperationOutcome,
    add_bucket as add_bucket_to_registry_outcome,
    remove_bucket as remove_bucket_from_registry_outcome,
    update_buckets as update_buckets_outcome,
};
pub(crate) use spoon_scoop::ScoopDoctorDetails;
pub(crate) use spoon_scoop::{
    ScoopActionPackage, ScoopBucketInventory, ScoopPackageActionOutcome,
    ScoopPackageInstallState, ScoopPackageOperationOutcome, ScoopPackagePlan,
};
pub(crate) use spoon_scoop::load_buckets_from_registry;
pub(crate) use spoon_scoop::{
    infer_tool_root_with_overrides as infer_tool_root,
    plan_package_action_with_display as plan_package_action,
};
pub(crate) use spoon_scoop::{installed_package_states, runtime_status, search_results};

pub type ScoopPackageDetailsOutcome = spoon_scoop::ScoopPackageDetailsOutcome<ConfigEntry>;

static REAL_BACKEND_TEST_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_real_backend_test_mode(enabled: bool) {
    REAL_BACKEND_TEST_MODE.store(enabled, Ordering::Relaxed);
}

pub(super) fn should_fake() -> bool {
    super::test_mode_enabled() && !REAL_BACKEND_TEST_MODE.load(Ordering::Relaxed)
}

pub(super) fn configured_proxy() -> String {
    crate::config::load_global_config().proxy.clone()
}

pub(super) fn command_result(
    title: impl Into<String>,
    status: CommandStatus,
) -> CommandResult {
    CommandResult {
        title: title.into(),
        status,
    }
}

pub(super) fn command_result_from_scoop_package_outcome(
    outcome: ScoopPackageOperationOutcome,
) -> CommandResult {
    command_result(outcome.title, outcome.status)
}

#[derive(Debug, Clone, Copy)]
pub(super) enum RunMode {
    Install,
    Update,
    Uninstall,
}

pub async fn package_info(tool_root: &Path, package_name: &str) -> ScoopPackageDetailsOutcome {
    let desired_policy = desired_policy_entries(package_name);
    let mut outcome = spoon_scoop::package_info::<ConfigEntry>(tool_root, package_name).await;
    if let ScoopPackageDetailsOutcome::Details(details) = &mut outcome {
        details.integration.policy.desired = desired_policy;
    }
    outcome
}
