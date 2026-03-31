//! Schema bootstrap and migration runner for the SQLite control plane.
//!
//! Migrations are ordered SQL files under `schema/`. Each file is embedded
//! at compile time and applied in sequence when [`run_migrations`] is called.

use rusqlite::Connection;

/// Embedded SQL for the initial control-plane schema.
const MIGRATION_0001: &str = include_str!("schema/0001_control_plane.sql");

/// All migrations in application order.
const MIGRATIONS: &[(&str, &str)] = &[("0001_control_plane", MIGRATION_0001)];

/// Run all pending schema migrations against the provided database connection.
///
/// This must be called exactly once during control-plane initialization,
/// before any store or repository module issues queries.
pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    // Ensure the schema_metadata table exists before we query it.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_metadata (
            version         INTEGER PRIMARY KEY,
            applied_at      TEXT    NOT NULL DEFAULT (datetime('now')),
            description     TEXT
        );",
    )?;

    // Determine which migrations have already been applied.
    let mut stmt = conn.prepare("SELECT version FROM schema_metadata ORDER BY version")?;
    let applied: Vec<i64> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    for (name, sql) in MIGRATIONS {
        let version: i64 = name.trim_start_matches('0').parse().unwrap_or(1);
        if applied.contains(&version) {
            tracing::debug!(migration = name, "already applied, skipping");
            continue;
        }
        tracing::info!(migration = name, "applying migration");
        conn.execute_batch(sql)?;
    }

    Ok(())
}
