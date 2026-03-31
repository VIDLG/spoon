//! SQLite control-plane for spoon-backend.
//!
//! This module owns the backend's database bootstrap, schema migrations,
//! and the async DB facade. It encapsulates `rusqlite` plus the repo-owned
//! tokio boundary so that
//! business-rule modules (lifecycle, projections, queries) never import
//! driver types directly.
//!
//! # Architecture
//!
//! - [`sqlite`] -- async DB facade ([`ControlPlaneDb`]) and connection
//!   configuration (WAL mode, foreign keys).
//! - [`migrations`] -- embedded SQL migration runner.
//! - [`schema`] -- compiled-in SQL migration files.

pub mod migrations;
pub mod journal_store;
pub mod lock_store;
pub mod doctor_store;
pub mod schema;
pub mod sqlite;

pub use doctor_store::{
    DoctorIssueRecord, list_doctor_issues, replace_legacy_state_issues,
    sync_failed_lifecycle_issues,
};
pub use journal_store::{begin_operation, complete_operation, set_operation_stage};
pub use lock_store::{acquire_lock, release_lock};
pub use sqlite::ControlPlaneDb;
