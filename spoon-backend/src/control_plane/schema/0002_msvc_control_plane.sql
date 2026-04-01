-- 0002_msvc_control_plane.sql
-- Add canonical MSVC control-plane state.

CREATE TABLE IF NOT EXISTS msvc_runtime_state (
    singleton_key       TEXT PRIMARY KEY CHECK (singleton_key = 'msvc'),
    runtime_kind        TEXT    NOT NULL,
    installed           INTEGER NOT NULL DEFAULT 0,
    version             TEXT,
    sdk_version         TEXT,
    last_operation      TEXT,
    last_stage          TEXT,
    validation_status   TEXT,
    validation_message  TEXT,
    managed_detail      TEXT    NOT NULL DEFAULT '{}',
    official_detail     TEXT    NOT NULL DEFAULT '{}',
    created_at          TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT    NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO schema_metadata (version, description)
VALUES (2, 'msvc canonical control-plane state');
