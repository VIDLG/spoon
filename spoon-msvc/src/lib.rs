//! spoon-msvc — MSVC toolchain domain logic.
//!
//! Provides types, facts, cache management, manifest parsing,
//! path utilities, and install/update/validate/uninstall workflows.

pub mod cache;
pub mod common;
pub mod detect;
pub mod doctor;
pub mod execute;
pub mod facts;
pub mod manifest;
pub mod msi_extract;
pub mod official;
pub mod paths;
pub mod platform;
pub mod query;
pub mod rules;
pub mod state;
pub mod status;
pub mod types;
pub mod validation;
pub mod wrappers;

pub mod plan;

pub use types::*;
