//! spoon-scoop — Scoop package manager domain logic.
//!
//! Provides manifest parsing, bucket management, package state queries,
//! and the ScoopPorts trait for filesystem operations.

pub mod bucket;
mod bucket_ops;
mod error;
mod helpers;
pub mod manifest;
pub mod ports;
mod queries;
pub mod response;
pub mod source;
pub mod state;
pub mod workflow;

pub use bucket::*;
pub use bucket_ops::*;
pub use error::{ScoopError, Result};
pub use helpers::*;
pub use manifest::*;
pub use ports::*;
pub use queries::*;
pub use response::*;
pub use source::*;
pub use state::*;
pub use workflow::{
    ScoopPackageAction, ScoopPackagePlan, acquire_assets, apply_install_surface,
    execute_package_action_streaming, infer_tool_root, infer_tool_root_with_overrides,
    install_package, materialize_assets, plan_package_action, plan_package_action_with_display,
    read_installed_state, reapply_command_surface, reapply_integrations,
    remove_installed_state, remove_surface, restore_persist_entries,
    run_integrations, sync_persist_entries, uninstall_package, update_package,
    write_installed_state,
};

#[cfg(test)]
mod tests;
