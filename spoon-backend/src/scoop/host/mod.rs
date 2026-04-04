mod download;
mod helpers;
mod hooks;
mod integration;
mod runtime;
pub(crate) mod persist;
pub(crate) mod surface;

pub use crate::download::hash_matches;
pub use download::ensure_downloaded_archive;
pub use helpers::helper_executable_path;
pub use runtime::{
    NoopPorts, ScoopRuntimeHost,
};
pub use super::ports::SupplementalShimSpec;
pub use hooks::{HookExecutionContext, HookPhase, execute_hook_scripts};
pub use integration::reapply_package_integrations;
pub use persist::{restore_persist_entries_into_root, sync_persist_entries_from_root};
pub use surface::{
    ensure_scoop_shims_activated_with_host,
    expanded_shim_targets, installed_targets_exist, installer_layout_error,
    reapply_package_command_surface,
    remove_shims, remove_shortcuts,
};
