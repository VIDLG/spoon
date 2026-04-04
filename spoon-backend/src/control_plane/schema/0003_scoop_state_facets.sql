-- 0003_scoop_state_facets.sql
-- Split Scoop installed state into identity + command-surface + integrations + uninstall facets.

ALTER TABLE installed_packages RENAME TO installed_packages_legacy;

CREATE TABLE IF NOT EXISTS installed_packages (
    package         TEXT    PRIMARY KEY,
    version         TEXT    NOT NULL,
    bucket          TEXT    NOT NULL,
    architecture    TEXT,
    cache_size_bytes INTEGER,
    created_at      TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO installed_packages (package, version, bucket, architecture, cache_size_bytes, created_at, updated_at)
SELECT package, version, bucket, architecture, cache_size_bytes, created_at, updated_at
FROM installed_packages_legacy;

CREATE TABLE IF NOT EXISTS installed_package_command_surface (
    package         TEXT    PRIMARY KEY,
    bins            TEXT    NOT NULL DEFAULT '[]',
    shortcuts       TEXT    NOT NULL DEFAULT '[]',
    env_add_path    TEXT    NOT NULL DEFAULT '[]',
    env_set         TEXT    NOT NULL DEFAULT '{}',
    persist         TEXT    NOT NULL DEFAULT '[]',
    FOREIGN KEY(package) REFERENCES installed_packages(package) ON DELETE CASCADE
);

INSERT INTO installed_package_command_surface (package, bins, shortcuts, env_add_path, env_set, persist)
SELECT package, bins, shortcuts, env_add_path, env_set, persist
FROM installed_packages_legacy;

CREATE TABLE IF NOT EXISTS installed_package_integrations (
    package            TEXT NOT NULL,
    integration_key    TEXT NOT NULL,
    integration_value  TEXT NOT NULL,
    PRIMARY KEY(package, integration_key),
    FOREIGN KEY(package) REFERENCES installed_packages(package) ON DELETE CASCADE
);

INSERT INTO installed_package_integrations (package, integration_key, integration_value)
SELECT legacy.package, json_each.key, json_each.value
FROM installed_packages_legacy AS legacy,
     json_each(legacy.integrations)
WHERE json_valid(legacy.integrations);

CREATE TABLE IF NOT EXISTS installed_package_uninstall (
    package            TEXT PRIMARY KEY,
    pre_uninstall      TEXT NOT NULL DEFAULT '[]',
    uninstaller_script TEXT NOT NULL DEFAULT '[]',
    post_uninstall     TEXT NOT NULL DEFAULT '[]',
    FOREIGN KEY(package) REFERENCES installed_packages(package) ON DELETE CASCADE
);

INSERT INTO installed_package_uninstall (package, pre_uninstall, uninstaller_script, post_uninstall)
SELECT package, pre_uninstall, uninstaller_script, post_uninstall
FROM installed_packages_legacy;

DROP TABLE installed_packages_legacy;
