use std::collections::BTreeMap;

use crate::db::Db;
use crate::layout::RuntimeLayout;
use crate::scoop::state::{
    InstalledPackageCommandSurface, InstalledPackageIdentity, InstalledPackageState,
    InstalledPackageUninstall, read_installed_state, write_installed_state,
};
use crate::scoop::{AppliedIntegration, PersistEntry, ShortcutEntry, runtime_status};
use crate::tests::{block_on, temp_dir};

fn sample_state() -> InstalledPackageState {
    InstalledPackageState {
        identity: InstalledPackageIdentity {
            package: "test-pkg".to_string(),
            version: "1.2.3".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: Some(1024),
        },
        command_surface: InstalledPackageCommandSurface {
            bins: vec!["bin/app.exe".to_string()],
            shortcuts: vec![ShortcutEntry {
                target_path: "bin/app.exe".to_string(),
                name: "Test App".to_string(),
                args: None,
                icon_path: None,
            }],
            env_add_path: vec!["bin".to_string()],
            env_set: BTreeMap::from([("FOO".to_string(), "bar".to_string())]),
            persist: vec![PersistEntry {
                relative_path: "data".to_string(),
                store_name: "data".to_string(),
            }],
        },
        integrations: Vec::new(),
        uninstall: InstalledPackageUninstall::default(),
    }
}

#[test]
fn canonical_installed_state_roundtrips_bucket_and_architecture() {
    let tmp = temp_dir("state-roundtrip");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&tmp);

    let state = InstalledPackageState {
        identity: InstalledPackageIdentity {
            package: "roundtrip-pkg".to_string(),
            version: "2.0.0".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
        },
        ..Default::default()
    };

    block_on(async {
        let db = Db::open(&layout.scoop.db_path())
            .await
            .expect("db should open");
        write_installed_state(&db, &state)
            .await
            .expect("write should succeed");

        let loaded = read_installed_state(&db, "roundtrip-pkg")
            .await
            .expect("state should exist after write");

        assert_eq!(loaded.package(), "roundtrip-pkg");
        assert_eq!(loaded.version(), "2.0.0");
        assert_eq!(loaded.bucket(), "main");
        assert_eq!(loaded.identity.architecture, Some("x64".to_string()));
    });
}

#[test]
fn canonical_state_persists_only_nonderivable_facts() {
    let tmp = temp_dir("state-keys");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&tmp);

    let state = sample_state();

    block_on(async {
        let db = Db::open(&layout.scoop.db_path())
            .await
            .expect("db should open");
        write_installed_state(&db, &state)
            .await
            .expect("write should succeed");

        let identity_json: serde_json::Value = db
            .call(|conn| {
                Ok(conn.query_row(
                    "SELECT json_object(
                        'package', package,
                        'version', version,
                        'bucket', bucket,
                        'architecture', architecture,
                        'cache_size_bytes', cache_size_bytes
                    ) FROM installed_packages WHERE package = ?1",
                    rusqlite::params!["test-pkg"],
                    |row| row.get::<_, String>(0),
                )?)
            })
            .await
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .expect("identity row JSON should parse");

        let surface_json: serde_json::Value = db
            .call(|conn| {
                Ok(conn.query_row(
                    "SELECT json_object(
                        'bins', bins,
                        'shortcuts', shortcuts,
                        'env_add_path', env_add_path,
                        'env_set', env_set,
                        'persist', persist
                    ) FROM installed_package_command_surface WHERE package = ?1",
                    rusqlite::params!["test-pkg"],
                    |row| row.get::<_, String>(0),
                )?)
            })
            .await
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .expect("surface row JSON should parse");

        assert!(identity_json.get("package").is_some());
        assert!(identity_json.get("version").is_some());
        assert!(identity_json.get("bucket").is_some());
        assert!(identity_json.get("architecture").is_some());

        assert!(surface_json.get("bins").is_some());
        assert!(surface_json.get("shortcuts").is_some());
        assert!(surface_json.get("env_add_path").is_some());
        assert!(surface_json.get("env_set").is_some());
        assert!(surface_json.get("persist").is_some());

        let forbidden_keys = ["current", "current_root", "shims_root", "apps_root", "tool_root"];
        for key in &forbidden_keys {
            assert!(identity_json.get(key).is_none(), "identity JSON must not contain '{key}'");
            assert!(surface_json.get(key).is_none(), "surface JSON must not contain '{key}'");
        }
    });
}

#[test]
fn runtime_status_uses_canonical_installed_state() {
    let tmp = temp_dir("status-canonical");
    std::fs::create_dir_all(&tmp).expect("create temp dir");

    block_on(async {
        let layout = RuntimeLayout::from_root(&tmp);

        let pkg_a = InstalledPackageState {
            identity: InstalledPackageIdentity {
                package: "alpha-tool".to_string(),
                version: "3.1.0".to_string(),
                bucket: "main".to_string(),
                architecture: Some("x64".to_string()),
                cache_size_bytes: None,
            },
            ..Default::default()
        };
        let pkg_b = InstalledPackageState {
            identity: InstalledPackageIdentity {
                package: "beta-lib".to_string(),
                version: "0.5.2".to_string(),
                bucket: "extras".to_string(),
                architecture: None,
                cache_size_bytes: Some(2048),
            },
            command_surface: InstalledPackageCommandSurface {
                bins: vec!["bin/beta.exe".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        let db = Db::open(&layout.scoop.db_path())
            .await
            .expect("db should open");
        write_installed_state(&db, &pkg_a).await.expect("write alpha-tool state");
        write_installed_state(&db, &pkg_b).await.expect("write beta-lib state");

        let status = runtime_status(&tmp).await;
        assert_eq!(status.kind, "scoop_status");
        assert!(status.success);
        assert_eq!(status.installed_packages.len(), 2);
        assert_eq!(status.installed_packages[0].name, "alpha-tool");
        assert_eq!(status.installed_packages[0].version, "3.1.0");
        assert_eq!(status.installed_packages[1].name, "beta-lib");
        assert_eq!(status.installed_packages[1].version, "0.5.2");
    });
}

#[test]
fn sqlite_control_plane_preserves_runtime_layout_derivation() {
    let tmp = temp_dir("sqlite-layout-derivation");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&tmp);

    block_on(async {
        let state = InstalledPackageState {
            identity: InstalledPackageIdentity {
                package: "layout-tool".to_string(),
                version: "9.9.9".to_string(),
                bucket: "main".to_string(),
                architecture: Some("x64".to_string()),
                cache_size_bytes: None,
            },
            command_surface: InstalledPackageCommandSurface {
                bins: vec!["bin/layout.exe".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        let db = Db::open(&layout.scoop.db_path())
            .await
            .expect("db should open");
        write_installed_state(&db, &state)
            .await
            .expect("write canonical state");

        let db_path = layout.scoop.db_path();
        assert!(db_path.exists(), "control-plane db should exist");
        assert_eq!(db_path.parent(), Some(layout.scoop.state_root.as_path()));

        let loaded = read_installed_state(&db, "layout-tool")
            .await
            .expect("state should load from sqlite");
        assert_eq!(loaded.package(), "layout-tool");
        assert_eq!(loaded.bucket(), "main");
        assert!(!layout.scoop.apps_root.join("layout-tool").join("current").exists());
    });
}

#[test]
fn canonical_state_persists_integrations_as_rows() {
    let tmp = temp_dir("state-integrations-rows");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&tmp);

    let state = InstalledPackageState {
        identity: InstalledPackageIdentity {
            package: "integrated-tool".to_string(),
            version: "1.0.0".to_string(),
            bucket: "main".to_string(),
            architecture: None,
            cache_size_bytes: None,
        },
        integrations: vec![
            AppliedIntegration {
                key: "tool.config".to_string(),
                value: "C:\\cfg\\tool".to_string(),
            },
            AppliedIntegration {
                key: "tool.cache".to_string(),
                value: "C:\\cache\\tool".to_string(),
            },
        ],
        ..Default::default()
    };

    block_on(async {
        let db = Db::open(&layout.scoop.db_path()).await.expect("db should open");
        write_installed_state(&db, &state).await.expect("write should succeed");

        let rows: Vec<(String, String)> = db
            .call(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT integration_key, integration_value
                     FROM installed_package_integrations
                     WHERE package = ?1
                     ORDER BY integration_key",
                )?;
                let mapped = stmt.query_map(rusqlite::params!["integrated-tool"], |row| {
                    Ok((row.get(0)?, row.get(1)?))
                })?;
                Ok(mapped.filter_map(|row| row.ok()).collect::<Vec<_>>())
            })
            .await
            .expect("query integration rows");

        assert_eq!(
            rows,
            vec![
                ("tool.cache".to_string(), "C:\\cache\\tool".to_string()),
                ("tool.config".to_string(), "C:\\cfg\\tool".to_string()),
            ]
        );
    });
}
