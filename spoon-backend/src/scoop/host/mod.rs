mod download;
mod execution;
mod hooks;
mod integration;
pub(crate) mod persist;
pub(crate) mod surface;

pub use download::{ensure_downloaded_archive, hash_matches, package_cache_file};
pub(crate) use execution::ContextRuntimeHost;
pub use execution::{
    NoopScoopRuntimeHost, ScoopRuntimeHost, ensure_scoop_shims_activated,
    ensure_scoop_shims_activated_with_context, ensure_scoop_shims_activated_with_host,
};
pub use super::ports::SupplementalShimSpec;
pub use hooks::{HookContext, execute_hook_scripts};
pub use integration::{
    apply_package_integrations, helper_executable_path, reapply_package_integrations_streaming,
    reapply_package_integrations_streaming_with_context,
    reapply_package_integrations_streaming_with_host,
};
pub use persist::{restore_persist_entries_into_root, sync_persist_entries_from_root};
pub use surface::{
    expanded_shim_targets, installed_targets_exist, installer_layout_error, load_manifest_value,
    reapply_package_command_surface_streaming,
    reapply_package_command_surface_streaming_with_context,
    reapply_package_command_surface_streaming_with_host,
    remove_shims, remove_shortcuts,
};
