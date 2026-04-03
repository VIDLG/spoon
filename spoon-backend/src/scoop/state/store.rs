use super::model::InstalledPackageState;
use crate::control_plane::ControlPlaneDb;
use crate::layout::RuntimeLayout;
use crate::{BackendError, Result};
use rusqlite::{OptionalExtension, params};
use std::convert::TryFrom;

#[derive(Debug)]
struct StoredInstalledPackageRow {
    package: String,
    version: String,
    bucket: String,
    architecture: Option<String>,
    cache_size_bytes: Option<i64>,
    bins: String,
    shortcuts: String,
    env_add_path: String,
    env_set: String,
    persist: String,
    integrations: String,
    pre_uninstall: String,
    uninstaller_script: String,
    post_uninstall: String,
}

impl TryFrom<StoredInstalledPackageRow> for InstalledPackageState {
    type Error = ();

    fn try_from(row: StoredInstalledPackageRow) -> std::result::Result<Self, Self::Error> {
        Ok(InstalledPackageState {
            package: row.package,
            version: row.version,
            bucket: row.bucket,
            architecture: row.architecture,
            cache_size_bytes: row.cache_size_bytes.and_then(|value| u64::try_from(value).ok()),
            bins: serde_json::from_str(&row.bins).map_err(|_| ())?,
            shortcuts: serde_json::from_str(&row.shortcuts).map_err(|_| ())?,
            env_add_path: serde_json::from_str(&row.env_add_path).map_err(|_| ())?,
            env_set: serde_json::from_str(&row.env_set).map_err(|_| ())?,
            persist: serde_json::from_str(&row.persist).map_err(|_| ())?,
            integrations: serde_json::from_str(&row.integrations).map_err(|_| ())?,
            pre_uninstall: serde_json::from_str(&row.pre_uninstall).map_err(|_| ())?,
            uninstaller_script: serde_json::from_str(&row.uninstaller_script).map_err(|_| ())?,
            post_uninstall: serde_json::from_str(&row.post_uninstall).map_err(|_| ())?,
        })
    }
}

/// Read a single canonical installed-package state.
///
/// Returns `None` if the state row does not exist or cannot be parsed.
pub async fn read_installed_state(
    layout: &RuntimeLayout,
    package_name: &str,
) -> Option<InstalledPackageState> {
    let db = ControlPlaneDb::open(&layout.scoop.control_plane_db_path()).await.ok()?;
    let package_name = package_name.to_string();
    let row = db
        .call(move |conn| {
            conn.query_row(
                "SELECT package, version, bucket, architecture, cache_size_bytes, bins, shortcuts, env_add_path, env_set, persist, integrations, pre_uninstall, uninstaller_script, post_uninstall
                 FROM installed_packages WHERE package = ?1",
                params![package_name],
                |row| {
                    Ok(StoredInstalledPackageRow {
                        package: row.get(0)?,
                        version: row.get(1)?,
                        bucket: row.get(2)?,
                        architecture: row.get(3)?,
                        cache_size_bytes: row.get(4)?,
                        bins: row.get(5)?,
                        shortcuts: row.get(6)?,
                        env_add_path: row.get(7)?,
                        env_set: row.get(8)?,
                        persist: row.get(9)?,
                        integrations: row.get(10)?,
                        pre_uninstall: row.get(11)?,
                        uninstaller_script: row.get(12)?,
                        post_uninstall: row.get(13)?,
                    })
                },
            )
            .optional()
        })
        .await
        .ok()?;
    row.and_then(|row| InstalledPackageState::try_from(row).ok())
}

/// Write (create or update) a canonical installed-package state row.
pub async fn write_installed_state(
    layout: &RuntimeLayout,
    state: &InstalledPackageState,
) -> Result<()> {
    let db = ControlPlaneDb::open(&layout.scoop.control_plane_db_path()).await?;
    let package = state.package.clone();
    let version = state.version.clone();
    let bucket = state.bucket.clone();
    let architecture = state.architecture.clone();
    let cache_size_bytes = state.cache_size_bytes.and_then(|value| i64::try_from(value).ok());
    let bins = serde_json::to_string(&state.bins)
        .map_err(|err| BackendError::external("failed to serialize installed state bins", err))?;
    let shortcuts = serde_json::to_string(&state.shortcuts).map_err(|err| {
        BackendError::external("failed to serialize installed state shortcuts", err)
    })?;
    let env_add_path = serde_json::to_string(&state.env_add_path).map_err(|err| {
        BackendError::external("failed to serialize installed state env_add_path", err)
    })?;
    let env_set = serde_json::to_string(&state.env_set)
        .map_err(|err| BackendError::external("failed to serialize installed state env_set", err))?;
    let persist = serde_json::to_string(&state.persist)
        .map_err(|err| BackendError::external("failed to serialize installed state persist", err))?;
    let integrations = serde_json::to_string(&state.integrations).map_err(|err| {
        BackendError::external("failed to serialize installed state integrations", err)
    })?;
    let pre_uninstall = serde_json::to_string(&state.pre_uninstall).map_err(|err| {
        BackendError::external("failed to serialize installed state pre_uninstall", err)
    })?;
    let uninstaller_script = serde_json::to_string(&state.uninstaller_script).map_err(|err| {
        BackendError::external("failed to serialize installed state uninstaller_script", err)
    })?;
    let post_uninstall = serde_json::to_string(&state.post_uninstall).map_err(|err| {
        BackendError::external("failed to serialize installed state post_uninstall", err)
    })?;

    db.call(move |conn| {
        conn.execute(
            "INSERT INTO installed_packages
                (package, version, bucket, architecture, cache_size_bytes, bins, shortcuts, env_add_path, env_set, persist, integrations, pre_uninstall, uninstaller_script, post_uninstall)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(package) DO UPDATE SET
                version = excluded.version,
                bucket = excluded.bucket,
                architecture = excluded.architecture,
                cache_size_bytes = excluded.cache_size_bytes,
                bins = excluded.bins,
                shortcuts = excluded.shortcuts,
                env_add_path = excluded.env_add_path,
                env_set = excluded.env_set,
                persist = excluded.persist,
                integrations = excluded.integrations,
                pre_uninstall = excluded.pre_uninstall,
                uninstaller_script = excluded.uninstaller_script,
                post_uninstall = excluded.post_uninstall,
                updated_at = datetime('now')",
            params![
                package,
                version,
                bucket,
                architecture,
                cache_size_bytes,
                bins,
                shortcuts,
                env_add_path,
                env_set,
                persist,
                integrations,
                pre_uninstall,
                uninstaller_script,
                post_uninstall,
            ],
        )?;
        Ok(())
    })
    .await
}

/// Remove a canonical installed-package state row.
///
/// Silently succeeds if the row does not exist.
pub async fn remove_installed_state(
    layout: &RuntimeLayout,
    package_name: &str,
) -> Result<()> {
    let db = ControlPlaneDb::open(&layout.scoop.control_plane_db_path()).await?;
    let package_name = package_name.to_string();
    db.call(move |conn| {
        conn.execute(
            "DELETE FROM installed_packages WHERE package = ?1",
            params![package_name],
        )?;
        Ok(())
    })
    .await
}

/// Enumerate all canonical installed-package states from SQLite.
///
/// Returns all rows that successfully deserialize as [`InstalledPackageState`].
pub async fn list_installed_states(layout: &RuntimeLayout) -> Vec<InstalledPackageState> {
    let Ok(db) = ControlPlaneDb::open(&layout.scoop.control_plane_db_path()).await else {
        return Vec::new();
    };

    let rows = db
        .call(|conn| {
            let mut stmt = conn.prepare(
                "SELECT package, version, bucket, architecture, cache_size_bytes, bins, shortcuts, env_add_path, env_set, persist, integrations, pre_uninstall, uninstaller_script, post_uninstall
                 FROM installed_packages",
            )?;
            let mapped = stmt.query_map([], |row| {
                Ok(StoredInstalledPackageRow {
                    package: row.get(0)?,
                    version: row.get(1)?,
                    bucket: row.get(2)?,
                    architecture: row.get(3)?,
                    cache_size_bytes: row.get(4)?,
                    bins: row.get(5)?,
                    shortcuts: row.get(6)?,
                    env_add_path: row.get(7)?,
                    env_set: row.get(8)?,
                    persist: row.get(9)?,
                    integrations: row.get(10)?,
                    pre_uninstall: row.get(11)?,
                    uninstaller_script: row.get(12)?,
                    post_uninstall: row.get(13)?,
                })
            })?;
            Ok(mapped.filter_map(|row| row.ok()).collect::<Vec<_>>())
        })
        .await
        .unwrap_or_default();

    rows.into_iter()
        .filter_map(|row| InstalledPackageState::try_from(row).ok())
        .collect()
}
