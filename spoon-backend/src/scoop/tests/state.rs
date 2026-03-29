use std::collections::BTreeMap;

use crate::layout::RuntimeLayout;
use crate::scoop::doctor::detect_legacy_flat_state_files;
use crate::scoop::runtime::{PersistEntry, ShortcutEntry};
use crate::scoop::state::{
    InstalledPackageState, read_installed_state, write_installed_state,
};
use crate::scoop::runtime_status;
use crate::tests::{block_on, temp_dir};

fn sample_state() -> InstalledPackageState {
    InstalledPackageState {
        package: "test-pkg".to_string(),
        version: "1.2.3".to_string(),
        bucket: "main".to_string(),
        architecture: Some("x64".to_string()),
        cache_size_bytes: Some(1024),
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
        integrations: BTreeMap::new(),
        pre_uninstall: vec![],
        uninstaller_script: vec![],
        post_uninstall: vec![],
    }
}

#[test]
fn canonical_installed_state_roundtrips_bucket_and_architecture() {
    let tmp = temp_dir("state-roundtrip");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&tmp);

    let state = InstalledPackageState {
        package: "roundtrip-pkg".to_string(),
        version: "2.0.0".to_string(),
        bucket: "main".to_string(),
        architecture: Some("x64".to_string()),
        cache_size_bytes: None,
        bins: vec![],
        shortcuts: vec![],
        env_add_path: vec![],
        env_set: BTreeMap::new(),
        persist: vec![],
        integrations: BTreeMap::new(),
        pre_uninstall: vec![],
        uninstaller_script: vec![],
        post_uninstall: vec![],
    };

    block_on(async {
        write_installed_state(&layout, &state)
            .await
            .expect("write should succeed");

        let loaded = read_installed_state(&layout, "roundtrip-pkg")
            .await
            .expect("state should exist after write");

        assert_eq!(loaded.package, "roundtrip-pkg");
        assert_eq!(loaded.version, "2.0.0");
        assert_eq!(loaded.bucket, "main");
        assert_eq!(loaded.architecture, Some("x64".to_string()));
    });
}

#[test]
fn canonical_state_persists_only_nonderivable_facts() {
    let tmp = temp_dir("state-keys");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&tmp);

    let state = sample_state();

    block_on(async {
        write_installed_state(&layout, &state)
            .await
            .expect("write should succeed");

        // Read the raw JSON to inspect persisted keys
        let path = layout.scoop.package_state_root.join("test-pkg.json");
        let raw = tokio::fs::read_to_string(&path)
            .await
            .expect("file should exist");
        let json: serde_json::Value = serde_json::from_str(&raw)
            .expect("json should parse");

        // Keys that MUST be present
        assert!(
            json.get("package").is_some(),
            "JSON must contain 'package'"
        );
        assert!(
            json.get("version").is_some(),
            "JSON must contain 'version'"
        );
        assert!(
            json.get("bucket").is_some(),
            "JSON must contain 'bucket'"
        );
        assert!(
            json.get("architecture").is_some(),
            "JSON must contain 'architecture'"
        );

        // Keys that must NOT be present (derivable from layout)
        let forbidden_keys = ["current", "current_root", "shims_root", "apps_root", "tool_root"];
        for key in &forbidden_keys {
            assert!(
                json.get(key).is_none(),
                "JSON must not contain derivable key '{key}'"
            );
        }
    });
}

#[test]
fn runtime_status_uses_canonical_installed_state() {
    let tmp = temp_dir("status-canonical");
    std::fs::create_dir_all(&tmp).expect("create temp dir");

    block_on(async {
        let layout = RuntimeLayout::from_root(&tmp);

        // Seed two canonical installed-state records
        let pkg_a = InstalledPackageState {
            package: "alpha-tool".to_string(),
            version: "3.1.0".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
            bins: vec![],
            shortcuts: vec![],
            env_add_path: vec![],
            env_set: BTreeMap::new(),
            persist: vec![],
            integrations: BTreeMap::new(),
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec![],
        };
        let pkg_b = InstalledPackageState {
            package: "beta-lib".to_string(),
            version: "0.5.2".to_string(),
            bucket: "extras".to_string(),
            architecture: None,
            cache_size_bytes: Some(2048),
            bins: vec!["bin/beta.exe".to_string()],
            shortcuts: vec![],
            env_add_path: vec![],
            env_set: BTreeMap::new(),
            persist: vec![],
            integrations: BTreeMap::new(),
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec![],
        };

        write_installed_state(&layout, &pkg_a)
            .await
            .expect("write alpha-tool state");
        write_installed_state(&layout, &pkg_b)
            .await
            .expect("write beta-lib state");

        // Assert runtime_status reports both packages through canonical store
        let status = runtime_status(&tmp).await;
        assert_eq!(status.kind, "scoop_status");
        assert!(status.success);
        assert_eq!(status.runtime.installed_package_count, 2);

        // Packages should be sorted by name
        assert_eq!(status.installed_packages.len(), 2);
        assert_eq!(status.installed_packages[0].name, "alpha-tool");
        assert_eq!(status.installed_packages[0].version, "3.1.0");
        assert_eq!(status.installed_packages[1].name, "beta-lib");
        assert_eq!(status.installed_packages[1].version, "0.5.2");
    });
}

#[test]
fn legacy_flat_scoop_state_is_reported() {
    let tmp = temp_dir("legacy-flat-state");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&tmp);

    block_on(async {
        // Seed a legacy flat state file directly in scoop/state/ (old layout)
        std::fs::create_dir_all(&layout.scoop.state_root).expect("create state root");
        let legacy_path = layout.scoop.state_root.join("old-tool.json");
        let legacy_content = serde_json::json!({
            "name": "old-tool",
            "version": "1.0.0",
            "bucket": "main",
            "architecture": "x64"
        });
        tokio::fs::write(&legacy_path, serde_json::to_string_pretty(&legacy_content).unwrap())
            .await
            .expect("write legacy state file");

        // Also seed a canonical state in packages/ to confirm it is NOT reported
        let canonical_state = InstalledPackageState {
            package: "canonical-tool".to_string(),
            version: "2.0.0".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
            bins: vec![],
            shortcuts: vec![],
            env_add_path: vec![],
            env_set: BTreeMap::new(),
            persist: vec![],
            integrations: BTreeMap::new(),
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec![],
        };
        write_installed_state(&layout, &canonical_state)
            .await
            .expect("write canonical state");

        // Detect legacy files
        let issues = detect_legacy_flat_state_files(&layout).await;

        // Should find exactly the one legacy flat file
        assert_eq!(issues.len(), 1, "expected exactly 1 legacy state issue");
        assert_eq!(issues[0].kind, "legacy scoop state");
        assert!(
            issues[0].path.contains("old-tool.json"),
            "issue path should reference old-tool.json, got: {}",
            issues[0].path
        );
        assert!(
            issues[0].message.contains("legacy scoop state"),
            "issue message should contain 'legacy scoop state', got: {}",
            issues[0].message
        );
        assert!(
            issues[0].message.contains("rebuild state"),
            "issue message should instruct to rebuild state, got: {}",
            issues[0].message
        );
    });
}

#[test]
fn no_legacy_issues_when_state_is_clean() {
    let tmp = temp_dir("clean-state-no-legacy");
    std::fs::create_dir_all(&tmp).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&tmp);

    block_on(async {
        // Only canonical state exists
        let state = InstalledPackageState {
            package: "clean-tool".to_string(),
            version: "1.0.0".to_string(),
            bucket: "main".to_string(),
            architecture: None,
            cache_size_bytes: None,
            bins: vec![],
            shortcuts: vec![],
            env_add_path: vec![],
            env_set: BTreeMap::new(),
            persist: vec![],
            integrations: BTreeMap::new(),
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec![],
        };
        write_installed_state(&layout, &state)
            .await
            .expect("write canonical state");

        let issues = detect_legacy_flat_state_files(&layout).await;
        assert!(issues.is_empty(), "clean state should produce no legacy issues");
    });
}
