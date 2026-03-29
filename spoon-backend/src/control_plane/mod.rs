//! SQLite control-plane for spoon-backend.
//!
//! This module owns the backend's database bootstrap, schema migrations,
//! and the async DB facade. It encapsulates `tokio-rusqlite` so that
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
pub mod schema;
pub mod sqlite;

pub use sqlite::ControlPlaneDb;
