//! MSVC execution workflows — install, update, uninstall, and validate operations.
//!
//! This module is split into focused submodules:
//! - [`pipeline`] — path helpers and `ensure_*` preparation functions
//! - [`integrity`] — hashing, archive-kind detection
//! - [`discover`] — binary discovery and toolchain flag construction
//! - [`workflow`] — install/update/uninstall lifecycle orchestration
//! - [`validate`] — compile and run C++/Rust validation samples

pub(crate) mod integrity;
pub(crate) mod pipeline;
pub(crate) mod discover;
pub(crate) mod workflow;
pub(crate) mod validate;

// ---------------------------------------------------------------------------
// Re-exports: all items that were `pub` in the original execute.rs
// ---------------------------------------------------------------------------

// From pipeline.rs
pub use pipeline::{
    ensure_cached_companion_cabs,
    ensure_cached_payloads,
    ensure_extracted_archives,
    ensure_extracted_msis,
    ensure_install_image,
    ensure_msi_media_metadata,
    ensure_staged_external_cabs,
    manifest_dir,
    msvc_dir,
    native_host_arch,
    runtime_state_path,
};

// From discover.rs
pub use discover::{
    find_preferred_msvc_binary,
    managed_toolchain_flags_with_request,
};

// From workflow.rs
pub use workflow::{
    cleanup_post_install_cache,
    ensure_materialized_toolchain,
    install_toolchain,
    uninstall_toolchain,
    update_toolchain,
};

// From validate.rs
pub use validate::validate_toolchain_async;
