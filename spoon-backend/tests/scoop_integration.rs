mod common;

use std::collections::BTreeMap;

use common::{block_on, temp_dir};
use spoon_backend::Db;
use spoon_backend::layout::RuntimeLayout;
use spoon_backend::scoop::{
    AppliedIntegration, InstalledPackageState, PersistEntry, ScoopPackageDetailsOutcome,
    ShortcutEntry, package_info, package_operation_outcome, sync_main_bucket_registry,
    write_installed_state,
};
use spoon_backend::scoop::state::{
    InstalledPackageCommandSurface, InstalledPackageIdentity, InstalledPackageUninstall,
};

use spoon_backend::CommandStatus;

#[derive(Debug, Clone, serde::Serialize)]
struct DesiredPolicy {
    key: &'static str,
    value: &'static str,
}

/// Regression: package_info reads from typed canonical state (bucket + architecture)
/// and not from raw JSON probing. Validates that detail and outcome surfaces
/// derive their installed-version, bins, shortcuts, env_add_path, env_set, persist,
/// and integrations from the canonical InstalledPackageState record.
#[test]
fn scoop_package_info_reads_canonical_state() {
    let root = temp_dir("scoop-canonical-info");
    let manifest_path = root
        .join("scoop")
        .join("buckets")
        .join("main")
        .join("bucket")
        .join("ripgrep.json");
    std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    std::fs::write(
        &manifest_path,
        serde_json::json!({
            "version": "14.1.0",
            "description": "Fast search tool",
            "homepage": "https://example.invalid/ripgrep",
            "license": "MIT",
            "url": "https://example.invalid/ripgrep.zip",
            "hash": "sha256:cafebabe",
            "bin": ["rg.exe"],
            "shortcuts": [
                { "name": "ripgrep", "target": "rg.exe" }
            ],
            "env_add_path": ".",
            "env_set": { "RIPGREP_CONFIG_PATH": "$dir\\config" },
            "persist": ["config"]
        })
        .to_string(),
    )
    .unwrap();
    block_on(sync_main_bucket_registry(&root)).unwrap();

    let current_root = root
        .join("scoop")
        .join("apps")
        .join("ripgrep")
        .join("current");
    std::fs::create_dir_all(&current_root).unwrap();
    std::fs::write(current_root.join("rg.exe"), b"rg").unwrap();

    let layout = RuntimeLayout::from_root(&root);
    let db = block_on(Db::open(&layout.scoop.db_path())).unwrap();
    block_on(write_installed_state(
        &db,
        &InstalledPackageState {
            identity: InstalledPackageIdentity {
                package: "ripgrep".to_string(),
                version: "14.1.0".to_string(),
                bucket: "main".to_string(),
                architecture: Some("x64".to_string()),
                cache_size_bytes: Some(2048),
            },
            command_surface: InstalledPackageCommandSurface {
                bins: vec!["rg".to_string()],
                shortcuts: vec![ShortcutEntry {
                    target_path: "rg.exe".to_string(),
                    name: "ripgrep".to_string(),
                    args: None,
                    icon_path: None,
                }],
                env_add_path: vec![".".to_string()],
                env_set: BTreeMap::from([(
                    "RIPGREP_CONFIG_PATH".to_string(),
                    "$dir\\config".to_string(),
                )]),
                persist: vec![PersistEntry {
                    relative_path: "config".to_string(),
                    store_name: "config".to_string(),
                }],
            },
            integrations: vec![AppliedIntegration {
                key: "ripgrep.shell_completions".to_string(),
                value: root.join("completions").display().to_string(),
            }],
            uninstall: InstalledPackageUninstall::default(),
        },
    ))
    .unwrap();

    // Test package_info reads from canonical state
    let desired = vec![DesiredPolicy {
        key: "ripgrep.command_profile",
        value: "fast",
    }];
    let data = block_on(package_info(&root, "ripgrep", desired.clone(), |entry| {
        entry.key
    }));

    match data {
        ScoopPackageDetailsOutcome::Details(details) => {
            // Metadata bucket comes from manifest resolution, not state
            assert_eq!(details.package.name, "ripgrep");
            assert_eq!(details.package.bucket, "main");
            assert_eq!(details.package.latest_version.as_deref(), Some("14.1.0"));

            // Install fields derived from canonical state
            assert!(details.install.installed);
            assert_eq!(details.install.installed_version.as_deref(), Some("14.1.0"));
            assert_eq!(details.install.cache_size_bytes, Some(2048));

            // Bins from canonical state (runtime bins take priority over manifest bins)
            assert_eq!(details.install.bins, vec!["rg"]);

            // Shortcut display from canonical state's typed ShortcutEntry
            assert_eq!(details.integration.system.shortcuts.len(), 1);
            assert!(details.integration.system.shortcuts[0].contains("ripgrep"));
            assert!(details.integration.system.shortcuts[0].contains("rg.exe"));

            // Shims from canonical state bins
            assert!(details.integration.commands.shims.is_some());
            let shims = details.integration.commands.shims.as_ref().unwrap();
            assert!(shims.iter().any(|s| s == "rg"));

            // Environment from canonical state
            assert!(!details.integration.environment.add_path.is_empty());
            assert!(!details.integration.environment.set.is_empty());

            // Persist from canonical state
            assert!(details.integration.environment.persist.is_some());

            // Integrations from canonical state
            assert_eq!(details.integration.policy.applied_values.len(), 1);
            assert_eq!(details.integration.policy.applied_values[0].key, "shell_completions");
        }
        ScoopPackageDetailsOutcome::Error(error) => panic!("unexpected error: {:?}", error),
    }

    // Test package_operation_outcome also reads canonical state
    let outcome = block_on(package_operation_outcome(
        &root,
        "update",
        "ripgrep",
        "ripgrep",
        CommandStatus::Success,
        "update ripgrep",
        vec!["updated".to_string()],
        false,
    ));
    assert!(outcome.is_success());
    assert_eq!(outcome.state.installed_version.as_deref(), Some("14.1.0"));
    assert!(outcome.state.installed);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn scoop_package_info_integrates_manifest_and_installed_state() {
    let root = temp_dir("scoop-info-integration");
    let manifest_path = root
        .join("scoop")
        .join("buckets")
        .join("main")
        .join("bucket")
        .join("python.json");
    std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    std::fs::write(
        &manifest_path,
        serde_json::json!({
            "version": "3.12.1",
            "description": "Python runtime",
            "homepage": "https://example.invalid/python",
            "license": "PSF-2.0",
            "url": "https://example.invalid/python.zip",
            "hash": "sha256:deadbeef",
            "bin": "python.exe",
            "env_add_path": ["Scripts"],
            "env_set": { "PYTHONHOME": "$dir" },
            "persist": ["Lib\\site-packages"]
        })
        .to_string(),
    )
    .unwrap();
    block_on(sync_main_bucket_registry(&root)).unwrap();

    let current_root = root
        .join("scoop")
        .join("apps")
        .join("python")
        .join("current");
    std::fs::create_dir_all(current_root.join("Scripts")).unwrap();
    std::fs::write(current_root.join("python.exe"), b"python").unwrap();
    std::fs::write(current_root.join("Scripts").join("pip.exe"), b"pip").unwrap();

    let layout = RuntimeLayout::from_root(&root);
    let db = block_on(Db::open(&layout.scoop.db_path())).unwrap();
    block_on(write_installed_state(
        &db,
        &InstalledPackageState {
            identity: InstalledPackageIdentity {
                package: "python".to_string(),
                version: "3.12.1".to_string(),
                bucket: "main".to_string(),
                architecture: None,
                cache_size_bytes: None,
            },
            command_surface: InstalledPackageCommandSurface {
                bins: vec!["python".to_string(), "pip".to_string()],
                shortcuts: Vec::<ShortcutEntry>::new(),
                env_add_path: vec!["Scripts".to_string()],
                env_set: BTreeMap::from([("PYTHONHOME".to_string(), "$dir".to_string())]),
                persist: vec![PersistEntry {
                    relative_path: "Lib\\site-packages".to_string(),
                    store_name: "Lib\\site-packages".to_string(),
                }],
            },
            integrations: vec![AppliedIntegration {
                key: "python.pip_config".to_string(),
                value: root.join("pip").display().to_string(),
            }],
            uninstall: InstalledPackageUninstall::default(),
        },
    ))
    .unwrap();

    let desired = vec![DesiredPolicy {
        key: "python.command_profile",
        value: "default",
    }];
    let data = block_on(package_info(&root, "python", desired.clone(), |entry| {
        entry.key
    }));

    match data {
        ScoopPackageDetailsOutcome::Details(success) => {
            assert_eq!(success.package.name, "python");
            assert_eq!(success.package.bucket, "main");
            assert_eq!(success.package.latest_version.as_deref(), Some("3.12.1"));
            assert!(success.install.installed);
            assert_eq!(success.install.installed_version.as_deref(), Some("3.12.1"));
            assert!(
                success
                    .integration
                    .commands
                    .shims
                    .as_ref()
                    .is_some_and(|items| items.iter().any(|item| item == "python"))
            );
            assert_eq!(success.integration.policy.desired.len(), 1);
        }
        ScoopPackageDetailsOutcome::Error(error) => panic!("unexpected error: {:?}", error),
    }

    let _ = std::fs::remove_dir_all(root);
}
