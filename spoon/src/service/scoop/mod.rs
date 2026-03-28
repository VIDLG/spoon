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
pub(crate) use bucket::bucket_update_streaming;
pub use bucket::{
    RepoSyncOutcome, bucket_action_result, bucket_add, bucket_inventory, bucket_list_report,
    bucket_remove, bucket_update, doctor_report, doctor_summary,
};
pub use report::{
    package_info_report, package_list_report, package_manifest, package_prefix_report,
    runtime_status_report, search_report,
};

pub use spoon_backend::scoop::{
    BucketSpec, ensure_main_bucket_ready, known_bucket_source, latest_version,
    latest_version_async, load_manifest, load_manifest_sync, load_package_manifest,
    load_package_manifest_sync, parse_manifest, plan_package_action, resolve_manifest,
    resolve_package_manifest, search_manifests_async, upsert_bucket_to_registry,
};

pub(crate) use spoon_backend::scoop::{
    ScoopActionPackage, ScoopBucketInventory as BackendScoopBucketInventory,
    ScoopBucketOperationOutcome, ScoopDoctorDetails, ScoopInstalledPackageEntry,
    ScoopPackageActionOutcome, ScoopPackageInstallState, ScoopPackageOperationOutcome,
    ScoopPackagePlan, add_bucket_to_registry_outcome, infer_tool_root, installed_package_states,
    installed_package_states_filtered, load_buckets_from_registry, package_current_root,
    remove_bucket_from_registry_outcome, runtime_status, search_results, update_buckets_outcome,
    update_buckets_streaming_outcome,
};

pub type ScoopPackageDetailsOutcome = spoon_backend::scoop::ScoopPackageDetailsOutcome<ConfigEntry>;

static REAL_BACKEND_TEST_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_real_backend_test_mode(enabled: bool) {
    REAL_BACKEND_TEST_MODE.store(enabled, Ordering::Relaxed);
}

pub(super) fn should_fake() -> bool {
    super::test_mode_enabled() && !REAL_BACKEND_TEST_MODE.load(Ordering::Relaxed)
}

pub(super) fn command_result(
    title: impl Into<String>,
    status: CommandStatus,
    output: Vec<String>,
    streamed: bool,
) -> CommandResult {
    CommandResult {
        title: title.into(),
        status,
        output,
        streamed,
    }
}

pub(super) fn command_result_from_scoop_package_outcome(
    outcome: ScoopPackageOperationOutcome,
) -> CommandResult {
    command_result(
        outcome.title,
        outcome.status,
        outcome.output,
        outcome.streamed,
    )
}

#[derive(Debug, Clone, Copy)]
pub(super) enum RunMode {
    Install,
    Update,
    Uninstall,
}

pub async fn package_info(tool_root: &Path, package_name: &str) -> ScoopPackageDetailsOutcome {
    let desired_policy = desired_policy_entries(package_name);
    spoon_backend::scoop::package_info(
        tool_root,
        package_name,
        desired_policy,
        |entry: &ConfigEntry| entry.key.as_str(),
    )
    .await
}
