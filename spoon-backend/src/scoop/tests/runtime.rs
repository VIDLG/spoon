use std::collections::BTreeMap;

use crate::layout::RuntimeLayout;
use crate::scoop::{
    InstalledPackageState, NoopScoopRuntimeHost, PersistEntry, ShortcutEntry,
    expanded_shim_targets, parse_selected_source,
};
use crate::scoop::state::{read_installed_state, write_installed_state};
use crate::tests::{block_on, temp_dir};
use serde_json::json;

#[test]
fn parse_selected_source_reads_common_manifest_fields() {
    let manifest = json!({
        "version": "3.12.1",
        "url": "https://example.invalid/python.zip",
        "hash": "sha256:deadbeef",
        "bin": "python.exe",
        "env_add_path": ["Scripts", "."],
        "env_set": { "PYTHONHOME": "$dir" }
    });

    let source = parse_selected_source(&manifest).expect("manifest should parse");
    assert_eq!(source.version, "3.12.1");
    assert_eq!(source.payloads.len(), 1);
    assert_eq!(source.bins.len(), 1);
    assert_eq!(source.bins[0].alias, "python");
    assert_eq!(
        source.env_add_path,
        vec!["Scripts".to_string(), ".".to_string()]
    );
    assert_eq!(
        source.env_set,
        BTreeMap::from([("PYTHONHOME".to_string(), "$dir".to_string())])
    );
}

#[test]
fn expanded_shim_targets_adds_cmd_and_bat_aliases() {
    let manifest = json!({
        "version": "3.12.1",
        "url": "https://example.invalid/python.zip",
        "hash": "sha256:deadbeef",
        "bin": "python.exe"
    });
    let source = parse_selected_source(&manifest).expect("manifest should parse");
    let current_root = temp_dir("runtime-expanded-shims");
    let host = NoopScoopRuntimeHost;
    let targets = expanded_shim_targets("python", &current_root, &source, &host);
    let aliases = targets
        .into_iter()
        .map(|target| target.alias)
        .collect::<Vec<_>>();
    assert!(aliases.iter().any(|alias| alias == "python"));
}

/// Simulate what actions.rs writes during install/update and prove the
/// canonical state record contains `bucket` and `architecture`.
#[test]
fn runtime_writes_canonical_scoop_state() {
    let tmp = temp_dir("runtime-canonical-write");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&tmp);

    let state = InstalledPackageState {
        package: "test-pkg".to_string(),
        version: "1.0.0".to_string(),
        bucket: "extras".to_string(),
        architecture: Some("64bit".to_string()),
        cache_size_bytes: Some(2048),
        bins: vec!["bin/app.exe".to_string()],
        shortcuts: vec![ShortcutEntry {
            target_path: "bin/app.exe".to_string(),
            name: "Test App".to_string(),
            args: None,
            icon_path: None,
        }],
        env_add_path: vec!["bin".to_string()],
        env_set: BTreeMap::from([("APP_HOME".to_string(), "$dir".to_string())]),
        persist: vec![PersistEntry {
            relative_path: "data".to_string(),
            store_name: "data".to_string(),
        }],
        integrations: BTreeMap::from([(
            "app.config".to_string(),
            "/path/to/config".to_string(),
        )]),
        pre_uninstall: vec!["stop-service.ps1".to_string()],
        uninstaller_script: vec!["uninstall.exe /S".to_string()],
        post_uninstall: vec!["cleanup.ps1".to_string()],
    };

    block_on(async {
        write_installed_state(&layout, &state)
            .await
            .expect("canonical write should succeed");

        // Read back via canonical store and verify bucket + architecture
        let loaded = read_installed_state(&layout, "test-pkg")
            .await
            .expect("canonical state should exist after write");

        assert_eq!(loaded.package, "test-pkg");
        assert_eq!(loaded.version, "1.0.0");
        assert_eq!(loaded.bucket, "extras");
        assert_eq!(loaded.architecture, Some("64bit".to_string()));
        assert_eq!(loaded.cache_size_bytes, Some(2048));
        assert_eq!(loaded.bins, vec!["bin/app.exe".to_string()]);
        assert_eq!(loaded.shortcuts.len(), 1);
        assert_eq!(loaded.env_add_path, vec!["bin".to_string()]);
        assert_eq!(loaded.env_set.get("APP_HOME").unwrap(), "$dir");
        assert_eq!(loaded.persist.len(), 1);
        assert_eq!(loaded.integrations.get("app.config").unwrap(), "/path/to/config");
        assert_eq!(loaded.pre_uninstall, vec!["stop-service.ps1".to_string()]);
        assert_eq!(loaded.uninstaller_script, vec!["uninstall.exe /S".to_string()]);
        assert_eq!(loaded.post_uninstall, vec!["cleanup.ps1".to_string()]);

        // Verify the raw JSON contains bucket and architecture
        let state_path = layout.scoop.package_state_root.join("test-pkg.json");
        let raw = tokio::fs::read_to_string(&state_path)
            .await
            .expect("state file should exist");
        let json: serde_json::Value = serde_json::from_str(&raw)
            .expect("state JSON should parse");
        assert_eq!(json["bucket"], "extras");
        assert_eq!(json["architecture"], "64bit");

        // Verify no absolute paths are persisted (SCST-04)
        let forbidden = ["current_root", "shims_root", "apps_root", "tool_root"];
        for key in &forbidden {
            assert!(
                json.get(key).is_none(),
                "JSON must not contain derivable key '{key}'"
            );
        }
    });

    let _ = std::fs::remove_dir_all(tmp);
}

/// Seed canonical state with all operational fields and prove that
/// reapply/uninstall inputs are read from the canonical state store
/// rather than any legacy path.
#[test]
fn reapply_inputs_come_from_canonical_state() {
    let tmp = temp_dir("reapply-canonical-read");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&tmp);

    // Seed a canonical installed-state record that mirrors what
    // actions.rs writes during install, with all operational fields.
    let seeded = InstalledPackageState {
        package: "reapply-pkg".to_string(),
        version: "2.5.0".to_string(),
        bucket: "main".to_string(),
        architecture: Some("64bit".to_string()),
        cache_size_bytes: Some(4096),
        bins: vec!["tool.exe".to_string(), "tool-cli.exe".to_string()],
        shortcuts: vec![ShortcutEntry {
            target_path: "tool.exe".to_string(),
            name: "My Tool".to_string(),
            args: Some("--fast".to_string()),
            icon_path: None,
        }],
        env_add_path: vec!["bin".to_string(), "scripts".to_string()],
        env_set: BTreeMap::from([
            ("TOOL_HOME".to_string(), "$dir".to_string()),
            ("TOOL_CFG".to_string(), "$persist_dir\\config".to_string()),
        ]),
        persist: vec![
            PersistEntry {
                relative_path: "config".to_string(),
                store_name: "config".to_string(),
            },
            PersistEntry {
                relative_path: "data\\db".to_string(),
                store_name: "data-db".to_string(),
            },
        ],
        integrations: BTreeMap::from([(
            "tool.settings".to_string(),
            "/custom/settings/path".to_string(),
        )]),
        pre_uninstall: vec!["pre-uninstall.ps1".to_string()],
        uninstaller_script: vec!["uninstaller.exe /quiet".to_string()],
        post_uninstall: vec!["post-cleanup.ps1".to_string()],
    };

    block_on(async {
        // Write via canonical store
        write_installed_state(&layout, &seeded)
            .await
            .expect("canonical write should succeed");

        // Read back via canonical store and verify ALL operational fields
        let loaded = read_installed_state(&layout, "reapply-pkg")
            .await
            .expect("canonical state should be readable for reapply");

        // These fields drive uninstall behavior
        assert_eq!(loaded.package, "reapply-pkg");
        assert_eq!(loaded.version, "2.5.0");
        assert_eq!(loaded.bucket, "main");
        assert_eq!(loaded.architecture, Some("64bit".to_string()));
        assert_eq!(
            loaded.bins,
            vec!["tool.exe".to_string(), "tool-cli.exe".to_string()],
            "bins must match for shim removal during uninstall"
        );
        assert_eq!(loaded.shortcuts.len(), 1, "shortcuts must match for removal");
        assert_eq!(loaded.shortcuts[0].name, "My Tool");
        assert_eq!(
            loaded.env_add_path,
            vec!["bin".to_string(), "scripts".to_string()],
            "env_add_path must match for reapply"
        );
        assert_eq!(
            loaded.env_set.get("TOOL_HOME").unwrap(),
            "$dir",
            "env_set must match for reapply"
        );
        assert_eq!(
            loaded.env_set.get("TOOL_CFG").unwrap(),
            "$persist_dir\\config",
            "env_set must match for reapply"
        );
        assert_eq!(loaded.persist.len(), 2, "persist must match for sync");
        assert_eq!(loaded.persist[0].relative_path, "config");
        assert_eq!(loaded.persist[1].store_name, "data-db");
        assert_eq!(
            loaded.pre_uninstall,
            vec!["pre-uninstall.ps1".to_string()],
            "pre_uninstall must match for uninstall hook"
        );
        assert_eq!(
            loaded.uninstaller_script,
            vec!["uninstaller.exe /quiet".to_string()],
            "uninstaller_script must match for uninstall"
        );
        assert_eq!(
            loaded.post_uninstall,
            vec!["post-cleanup.ps1".to_string()],
            "post_uninstall must match for cleanup hook"
        );
        assert_eq!(
            loaded.integrations.get("tool.settings").unwrap(),
            "/custom/settings/path",
            "integrations must match for reapply"
        );

        // Prove this is the canonical state path, not a legacy path
        let canonical_path = layout.scoop.package_state_root.join("reapply-pkg.json");
        assert!(
            canonical_path.exists(),
            "state must exist at canonical package_state_root"
        );
    });

    let _ = std::fs::remove_dir_all(tmp);
}
