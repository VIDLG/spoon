mod actions;
mod download;
mod execution;
mod hooks;
pub mod installed_state;
mod integration;
mod persist;
mod source;
mod surface;

pub use actions::{
    execute_package_action_outcome_streaming,
    execute_package_action_outcome_streaming_with_context,
    execute_package_action_outcome_streaming_with_host, execute_package_action_streaming,
    execute_package_action_streaming_with_context, execute_package_action_streaming_with_host,
};
pub use download::{ensure_downloaded_archive, hash_matches, package_cache_file};
pub use execution::{
    NoopScoopRuntimeHost, ScoopRuntimeHost, SupplementalShimSpec, ensure_scoop_shims_activated,
    ensure_scoop_shims_activated_with_context, ensure_scoop_shims_activated_with_host,
};
pub use hooks::{HookContext, execute_hook_scripts};
pub use installed_state::InstalledPackageState;
// Runtime installed-state persistence is now owned by the canonical state store.
// See crate::scoop::state::{read_installed_state, write_installed_state, remove_installed_state}.
pub use integration::{
    apply_package_integrations, helper_executable_path, reapply_package_integrations_streaming,
    reapply_package_integrations_streaming_with_host, resolved_pip_mirror_url_for_display,
    resolved_pip_mirror_url_for_display_with_host,
};
pub use persist::{restore_persist_entries_into_root, sync_persist_entries_from_root};
pub use source::{
    PackagePayload, PersistEntry, SelectedPackageSource, ShimTarget, ShortcutEntry,
    dependency_lookup_key, parse_selected_source, selected_architecture_key,
};
pub use surface::{
    expanded_shim_targets, installed_targets_exist, installer_layout_error, load_manifest_value,
    reapply_package_command_surface_streaming, reapply_package_command_surface_streaming_with_host,
    remove_shims, remove_shortcuts,
};
