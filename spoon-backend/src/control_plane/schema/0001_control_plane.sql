-- 0001_control_plane.sql
-- Initial control-plane schema for spoon-backend.
--
-- Tables:
--   installed_packages  - canonical installed-package metadata
--   operation_journal   - record of lifecycle operations for audit / recovery
--   bucket_registry     - bucket metadata (name, remote, local path)
--   doctor_issues       - doctor-detected repair items
--   operation_locks     - in-flight operation mutual exclusion
--   schema_metadata     - migration tracking

CREATE TABLE IF NOT EXISTS installed_packages (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    package         TEXT    NOT NULL,
    version         TEXT    NOT NULL,
    bucket          TEXT    NOT NULL,
    architecture    TEXT,
    cache_size_bytes INTEGER,
    bins            TEXT    NOT NULL DEFAULT '[]',
    shortcuts       TEXT    NOT NULL DEFAULT '[]',
    env_add_path    TEXT    NOT NULL DEFAULT '[]',
    env_set         TEXT    NOT NULL DEFAULT '{}',
    persist         TEXT    NOT NULL DEFAULT '[]',
    integrations    TEXT    NOT NULL DEFAULT '{}',
    pre_uninstall   TEXT    NOT NULL DEFAULT '[]',
    uninstaller_script TEXT NOT NULL DEFAULT '[]',
    post_uninstall  TEXT    NOT NULL DEFAULT '[]',
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(package)
);

CREATE TABLE IF NOT EXISTS operation_journal (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_type  TEXT    NOT NULL,
    package         TEXT,
    bucket          TEXT,
    status          TEXT    NOT NULL DEFAULT 'pending',
    started_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    finished_at     TEXT,
    details         TEXT
);

CREATE TABLE IF NOT EXISTS bucket_registry (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT    NOT NULL UNIQUE,
    remote_url      TEXT    NOT NULL,
    branch          TEXT    NOT NULL DEFAULT 'master',
    added_at        TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS doctor_issues (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    severity        TEXT    NOT NULL,
    category        TEXT    NOT NULL,
    description     TEXT    NOT NULL,
    package         TEXT,
    bucket          TEXT,
    detected_at     TEXT    NOT NULL DEFAULT (datetime('now')),
    resolved        INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS operation_locks (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    lock_key        TEXT    NOT NULL UNIQUE,
    operation_type  TEXT    NOT NULL,
    acquired_at     TEXT    NOT NULL DEFAULT (datetime('now')),
    expires_at      TEXT
);

CREATE TABLE IF NOT EXISTS schema_metadata (
    version         INTEGER PRIMARY KEY,
    applied_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    description     TEXT
);

-- Seed the baseline migration record.
INSERT OR IGNORE INTO schema_metadata (version, description)
VALUES (1, 'initial control-plane schema');
