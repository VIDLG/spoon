use std::convert::TryFrom;

use rusqlite::{OptionalExtension, params};

use crate::db::Db;
use crate::{BackendError, Result};

use super::super::AppliedIntegration;
use super::model::{
    InstalledPackageCommandSurface, InstalledPackageIdentity, InstalledPackageState,
    InstalledPackageUninstall,
};

#[derive(Debug)]
struct StoredInstalledPackageIdentityRow {
    package: String,
    version: String,
    bucket: String,
    architecture: Option<String>,
    cache_size_bytes: Option<i64>,
}

#[derive(Debug)]
struct StoredInstalledPackageCommandSurfaceRow {
    bins: String,
    shortcuts: String,
    env_add_path: String,
    env_set: String,
    persist: String,
}

#[derive(Debug)]
struct StoredInstalledPackageUninstallRow {
    pre_uninstall: String,
    uninstaller_script: String,
    post_uninstall: String,
}

impl TryFrom<StoredInstalledPackageIdentityRow> for InstalledPackageIdentity {
    type Error = ();

    fn try_from(row: StoredInstalledPackageIdentityRow) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            package: row.package,
            version: row.version,
            bucket: row.bucket,
            architecture: row.architecture,
            cache_size_bytes: row.cache_size_bytes.and_then(|value| u64::try_from(value).ok()),
        })
    }
}

impl TryFrom<StoredInstalledPackageCommandSurfaceRow> for InstalledPackageCommandSurface {
    type Error = ();

    fn try_from(
        row: StoredInstalledPackageCommandSurfaceRow,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            bins: serde_json::from_str(&row.bins).map_err(|_| ())?,
            shortcuts: serde_json::from_str(&row.shortcuts).map_err(|_| ())?,
            env_add_path: serde_json::from_str(&row.env_add_path).map_err(|_| ())?,
            env_set: serde_json::from_str(&row.env_set).map_err(|_| ())?,
            persist: serde_json::from_str(&row.persist).map_err(|_| ())?,
        })
    }
}

impl TryFrom<StoredInstalledPackageUninstallRow> for InstalledPackageUninstall {
    type Error = ();

    fn try_from(
        row: StoredInstalledPackageUninstallRow,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            pre_uninstall: serde_json::from_str(&row.pre_uninstall).map_err(|_| ())?,
            uninstaller_script: serde_json::from_str(&row.uninstaller_script).map_err(|_| ())?,
            post_uninstall: serde_json::from_str(&row.post_uninstall).map_err(|_| ())?,
        })
    }
}

pub async fn read_installed_state(db: &Db, package_name: &str) -> Option<InstalledPackageState> {
    let package_name = package_name.to_string();
    let identity_row = db
        .call(move |conn| {
            conn.query_row(
                "SELECT package, version, bucket, architecture, cache_size_bytes
                 FROM installed_packages WHERE package = ?1",
                params![package_name],
                |row| {
                    Ok(StoredInstalledPackageIdentityRow {
                        package: row.get(0)?,
                        version: row.get(1)?,
                        bucket: row.get(2)?,
                        architecture: row.get(3)?,
                        cache_size_bytes: row.get(4)?,
                    })
                },
            )
            .optional()
        })
        .await
        .ok()??;

    let package = identity_row.package.clone();
    let command_surface_row = db
        .call(move |conn| {
            conn.query_row(
                "SELECT bins, shortcuts, env_add_path, env_set, persist
                 FROM installed_package_command_surface WHERE package = ?1",
                params![package],
                |row| {
                    Ok(StoredInstalledPackageCommandSurfaceRow {
                        bins: row.get(0)?,
                        shortcuts: row.get(1)?,
                        env_add_path: row.get(2)?,
                        env_set: row.get(3)?,
                        persist: row.get(4)?,
                    })
                },
            )
            .optional()
        })
        .await
        .ok()
        .flatten();

    let package = identity_row.package.clone();
    let uninstall_row = db
        .call(move |conn| {
            conn.query_row(
                "SELECT pre_uninstall, uninstaller_script, post_uninstall
                 FROM installed_package_uninstall WHERE package = ?1",
                params![package],
                |row| {
                    Ok(StoredInstalledPackageUninstallRow {
                        pre_uninstall: row.get(0)?,
                        uninstaller_script: row.get(1)?,
                        post_uninstall: row.get(2)?,
                    })
                },
            )
            .optional()
        })
        .await
        .ok()
        .flatten();

    let package = identity_row.package.clone();
    let integrations = db
        .call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT integration_key, integration_value
                 FROM installed_package_integrations
                 WHERE package = ?1
                 ORDER BY integration_key",
            )?;
            let rows = stmt.query_map(params![package], |row| {
                Ok(AppliedIntegration {
                    key: row.get(0)?,
                    value: row.get(1)?,
                })
            })?;
            Ok(rows.filter_map(|row| row.ok()).collect::<Vec<_>>())
        })
        .await
        .ok()
        .unwrap_or_default();

    Some(InstalledPackageState {
        identity: InstalledPackageIdentity::try_from(identity_row).ok()?,
        command_surface: match command_surface_row {
            Some(row) => InstalledPackageCommandSurface::try_from(row).ok()?,
            None => InstalledPackageCommandSurface::default(),
        },
        integrations,
        uninstall: match uninstall_row {
            Some(row) => InstalledPackageUninstall::try_from(row).ok()?,
            None => InstalledPackageUninstall::default(),
        },
    })
}

pub async fn write_installed_state(db: &Db, state: &InstalledPackageState) -> Result<()> {
    let package = state.identity.package.clone();
    let version = state.identity.version.clone();
    let bucket = state.identity.bucket.clone();
    let architecture = state.identity.architecture.clone();
    let cache_size_bytes = state
        .identity
        .cache_size_bytes
        .and_then(|value| i64::try_from(value).ok());
    let bins = serde_json::to_string(&state.command_surface.bins)
        .map_err(|err| BackendError::external("failed to serialize installed state bins", err))?;
    let shortcuts = serde_json::to_string(&state.command_surface.shortcuts).map_err(|err| {
        BackendError::external("failed to serialize installed state shortcuts", err)
    })?;
    let env_add_path = serde_json::to_string(&state.command_surface.env_add_path).map_err(|err| {
        BackendError::external("failed to serialize installed state env_add_path", err)
    })?;
    let env_set = serde_json::to_string(&state.command_surface.env_set)
        .map_err(|err| BackendError::external("failed to serialize installed state env_set", err))?;
    let persist = serde_json::to_string(&state.command_surface.persist)
        .map_err(|err| BackendError::external("failed to serialize installed state persist", err))?;
    let pre_uninstall = serde_json::to_string(&state.uninstall.pre_uninstall).map_err(|err| {
        BackendError::external("failed to serialize installed state pre_uninstall", err)
    })?;
    let uninstaller_script =
        serde_json::to_string(&state.uninstall.uninstaller_script).map_err(|err| {
            BackendError::external("failed to serialize installed state uninstaller_script", err)
        })?;
    let post_uninstall = serde_json::to_string(&state.uninstall.post_uninstall).map_err(|err| {
        BackendError::external("failed to serialize installed state post_uninstall", err)
    })?;
    let integrations = state.integrations.clone();

    db.call(move |conn| {
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO installed_packages
                (package, version, bucket, architecture, cache_size_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(package) DO UPDATE SET
                version = excluded.version,
                bucket = excluded.bucket,
                architecture = excluded.architecture,
                cache_size_bytes = excluded.cache_size_bytes,
                updated_at = datetime('now')",
            params![package, version, bucket, architecture, cache_size_bytes],
        )?;
        tx.execute(
            "INSERT INTO installed_package_command_surface
                (package, bins, shortcuts, env_add_path, env_set, persist)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(package) DO UPDATE SET
                bins = excluded.bins,
                shortcuts = excluded.shortcuts,
                env_add_path = excluded.env_add_path,
                env_set = excluded.env_set,
                persist = excluded.persist",
            params![
                package,
                bins,
                shortcuts,
                env_add_path,
                env_set,
                persist,
            ],
        )?;
        tx.execute(
            "INSERT INTO installed_package_uninstall
                (package, pre_uninstall, uninstaller_script, post_uninstall)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(package) DO UPDATE SET
                pre_uninstall = excluded.pre_uninstall,
                uninstaller_script = excluded.uninstaller_script,
                post_uninstall = excluded.post_uninstall",
            params![package, pre_uninstall, uninstaller_script, post_uninstall],
        )?;
        tx.execute(
            "DELETE FROM installed_package_integrations WHERE package = ?1",
            params![package],
        )?;
        for integration in integrations {
            tx.execute(
                "INSERT INTO installed_package_integrations
                    (package, integration_key, integration_value)
                 VALUES (?1, ?2, ?3)",
                params![package, integration.key, integration.value],
            )?;
        }
        tx.commit()?;
        Ok(())
    })
    .await
}

pub async fn remove_installed_state(db: &Db, package_name: &str) -> Result<()> {
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

pub async fn list_installed_states(db: &Db) -> Vec<InstalledPackageState> {
    let packages = db
        .call(|conn| {
            let mut stmt = conn.prepare("SELECT package FROM installed_packages ORDER BY package")?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            Ok(rows.filter_map(|row| row.ok()).collect::<Vec<_>>())
        })
        .await
        .unwrap_or_default();

    let mut states = Vec::new();
    for package in packages {
        if let Some(state) = read_installed_state(db, &package).await {
            states.push(state);
        }
    }
    states
}
