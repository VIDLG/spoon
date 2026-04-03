//! Schema bootstrap and migration runner for the SQLite control plane.
//!
//! Migrations are ordered SQL files under `schema/`. Each file is embedded
//! at compile time and applied in sequence when [`run_migrations`] is called.

use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

/// Embedded SQL for the initial control-plane schema.
const MIGRATION_0001: &str = include_str!("schema/0001_control_plane.sql");
const MIGRATION_0002: &str = include_str!("schema/0002_msvc_control_plane.sql");

/// Run all pending schema migrations against the provided database connection.
///
/// This must be called exactly once during control-plane initialization,
/// before any store or repository module issues queries.
pub fn run_migrations(conn: &mut Connection) -> Result<(), rusqlite_migration::Error> {
    let migrations = Migrations::new(vec![M::up(MIGRATION_0001), M::up(MIGRATION_0002)]);
    migrations.to_latest(conn)
}
