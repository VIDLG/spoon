mod common;

use std::collections::BTreeMap;

use common::{block_on, temp_dir};
use spoon_backend::layout::RuntimeLayout;
use spoon_backend::scoop::{
    InstalledPackageState, PersistEntry, ScoopPackageDetailsOutcome, ShortcutEntry, package_info,
    sync_main_bucket_registry, write_installed_state,
};

#[derive(Debug, Clone, serde::Serialize)]
struct DesiredPolicy {
    key: &'static str,
    value: &'static str,
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
    block_on(write_installed_state(
        &layout,
        &InstalledPackageState {
            package: "python".to_string(),
            version: "3.12.1".to_string(),
            bucket: "main".to_string(),
            architecture: None,
            bins: vec!["python".to_string(), "pip".to_string()],
            cache_size_bytes: None,
            shortcuts: Vec::<ShortcutEntry>::new(),
            env_add_path: vec!["Scripts".to_string()],
            env_set: BTreeMap::from([("PYTHONHOME".to_string(), "$dir".to_string())]),
            persist: vec![PersistEntry {
                relative_path: "Lib\\site-packages".to_string(),
                store_name: "Lib\\site-packages".to_string(),
            }],
            integrations: BTreeMap::from([(
                "python.pip_config".to_string(),
                root.join("pip").display().to_string(),
            )]),
            pre_uninstall: Vec::new(),
            uninstaller_script: Vec::new(),
            post_uninstall: Vec::new(),
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
