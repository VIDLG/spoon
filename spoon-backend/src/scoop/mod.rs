mod actions;
pub mod buckets;
mod cache;
mod doctor;
mod extract;
pub mod host;
mod info;
pub(crate) mod lifecycle;
pub mod manifest;
pub mod package_source;
pub mod ports;
pub mod planner;
mod projection;
mod query;
pub mod state;

pub use actions::{
    execute_package_action_outcome_streaming,
    execute_package_action_outcome_streaming_with_context,
    execute_package_action_outcome_streaming_with_host, execute_package_action_streaming,
    execute_package_action_streaming_with_context, execute_package_action_streaming_with_host,
};
pub use buckets::{
    Bucket, BucketSpec, BucketUpdateSummary, ScoopBucketInventory, ScoopBucketOperationOutcome,
    add_bucket_to_registry, add_bucket_to_registry_outcome, add_bucket_to_registry_with_context,
    ensure_main_bucket_ready, ensure_main_bucket_ready_with_context, known_bucket_source,
    load_buckets_from_registry, remove_bucket_from_registry, remove_bucket_from_registry_outcome,
    resolve_manifest, sync_main_bucket_registry, update_buckets, update_buckets_outcome,
    update_buckets_streaming, update_buckets_streaming_outcome,
    update_buckets_streaming_outcome_with_context, update_buckets_streaming_with_context,
    update_buckets_with_context, upsert_bucket_to_registry,
};
pub use cache::{clear as clear_cache, prune as prune_cache};
pub use doctor::{ScoopDoctorDetails, ScoopRuntimeDetails, doctor_with_context, doctor_with_host};
pub use extract::{
    copy_path_recursive, detect_archive_kind, extract_archive_sync, extract_archive_to_root,
    helper_7z_candidates, materialize_installer_payloads_to_root, refresh_current_entry,
    remove_path_if_exists,
};
pub use info::{
    ScoopActionPackage, ScoopCommandIntegration, ScoopEnvironmentIntegration,
    ScoopPackageActionOutcome, ScoopPackageDetails, ScoopPackageDetailsError,
    ScoopPackageDetailsOutcome, ScoopPackageError, ScoopPackageInstall, ScoopPackageInstallState,
    ScoopPackageIntegration, ScoopPackageManifestOutcome, ScoopPackageMetadata,
    ScoopPackageOperationOutcome, ScoopPolicyAppliedValue, package_info, package_manifest,
    package_operation_outcome,
};
pub use manifest::{
    ArchConfig, ArchitectureMap, BinEntries, BinEntry, Installer, License, Notes, ScoopBinField,
    ScoopManifest, ScoopStringField, Shortcut, StringOrArray, SuggestMap, latest_version,
    latest_version_async, load_manifest, load_manifest_sync, load_package_manifest,
    load_package_manifest_sync, parse_manifest, resolve_package_manifest, search_manifests_async,
};
pub use package_source::{
    PackagePayload, PersistEntry, SelectedPackageSource, ShimTarget, ShortcutEntry,
    dependency_lookup_key, parse_selected_source, selected_architecture_key,
};
pub use planner::{ScoopPackageAction, ScoopPackagePlan, infer_tool_root, plan_package_action};
pub use ports::{ScoopIntegrationPort, SupplementalShimSpec};
pub use query::{
    ScoopPaths, ScoopRuntimeStatus, ScoopSearchMatch, ScoopSearchResults, ScoopStatus,
    installed_package_states, installed_package_states_filtered, runtime_status, search_results,
};
pub use host::{
    HookContext, NoopScoopRuntimeHost, ScoopRuntimeHost, apply_package_integrations,
    ensure_downloaded_archive, ensure_scoop_shims_activated,
    ensure_scoop_shims_activated_with_context, ensure_scoop_shims_activated_with_host,
    execute_hook_scripts, expanded_shim_targets, hash_matches, helper_executable_path,
    installed_targets_exist, installer_layout_error, load_manifest_value, package_cache_file,
    reapply_package_command_surface_streaming,
    reapply_package_command_surface_streaming_with_context,
    reapply_package_command_surface_streaming_with_host,
    reapply_package_integrations_streaming,
    reapply_package_integrations_streaming_with_context,
    reapply_package_integrations_streaming_with_host, remove_shims, remove_shortcuts,
    restore_persist_entries_into_root, sync_persist_entries_from_root,
};
pub use state::InstalledPackageState;
pub use state::{
    list_installed_states, read_installed_state, remove_installed_state, write_installed_state,
};

#[cfg(test)]
mod tests;
