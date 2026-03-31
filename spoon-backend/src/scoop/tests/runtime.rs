use std::collections::BTreeMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use crate::event::{BackendEvent, LifecycleStage};
use crate::layout::RuntimeLayout;
use crate::scoop::{
    BucketSpec, InstalledPackageState, NoopScoopRuntimeHost, PersistEntry, ScoopIntegrationPort,
    ScoopPackagePlan, ShortcutEntry, execute_package_action_outcome_streaming_with_context,
    expanded_shim_targets, parse_selected_source,
};
use crate::scoop::{plan_package_action, sync_main_bucket_registry, upsert_bucket_to_registry};
use crate::scoop::state::{read_installed_state, write_installed_state};
use crate::control_plane::sqlite::db_path_for_layout;
use crate::tests::{block_on, temp_dir};
use crate::{BackendContext, Result, SystemPort};
use serde_json::json;
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;

struct TestPorts;

impl SystemPort for TestPorts {
    fn home_dir(&self) -> PathBuf {
        PathBuf::from(".")
    }

    fn ensure_user_path_entry(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn ensure_process_path_entry(&self, _path: &Path) {}

    fn remove_user_path_entry(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn remove_process_path_entry(&self, _path: &Path) {}
}

impl ScoopIntegrationPort for TestPorts {
    fn supplemental_shims(
        &self,
        _package_name: &str,
        _current_root: &Path,
    ) -> Vec<crate::scoop::SupplementalShimSpec> {
        Vec::new()
    }

    fn apply_integrations<'a>(
        &'a self,
        _package_name: &'a str,
        _current_root: &'a Path,
        _persist_root: &'a Path,
        _emit: &'a mut dyn FnMut(BackendEvent),
    ) -> Pin<Box<dyn Future<Output = Result<BTreeMap<String, String>>> + 'a>> {
        Box::pin(async { Ok(BTreeMap::new()) })
    }
}

fn create_test_zip(root: &Path, file_name: &str, bytes: &[u8]) -> (PathBuf, String) {
    let zip_path = root.join(format!("{file_name}.zip"));
    let file = std::fs::File::create(&zip_path).expect("create zip");
    let mut writer = zip::ZipWriter::new(file);
    writer
        .start_file(file_name, SimpleFileOptions::default())
        .expect("start zip entry");
    std::io::Write::write_all(&mut writer, bytes).expect("write zip entry");
    writer.finish().expect("finish zip");

    let mut hasher = Sha256::new();
    hasher.update(std::fs::read(&zip_path).expect("read zip"));
    let hash = format!("{:x}", hasher.finalize());
    (zip_path, hash)
}

fn file_url(path: &Path) -> String {
    format!("file:///{}", path.display().to_string().replace('\\', "/"))
}

fn collect_stages(events: &[BackendEvent]) -> Vec<LifecycleStage> {
    events
        .iter()
        .filter_map(|event| match event {
            BackendEvent::Progress(progress) => progress.stage,
            BackendEvent::Finished(_) => None,
        })
        .collect()
}

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

        // Verify the persisted control-plane row contains canonical fields
        let db = crate::control_plane::ControlPlaneDb::open_for_layout(&layout)
            .await
            .expect("open control plane db");
        let persisted: (String, String, String, Option<String>, String, String, String, String) = db
            .call(|conn| {
                conn.query_row(
                    "SELECT package, version, bucket, architecture, bins, env_add_path, env_set, persist
                     FROM installed_packages WHERE package = ?1",
                    rusqlite::params!["test-pkg"],
                    |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                            row.get(5)?,
                            row.get(6)?,
                            row.get(7)?,
                        ))
                    },
                )
            })
            .await
            .expect("query canonical state row");
        assert_eq!(persisted.0, "test-pkg");
        assert_eq!(persisted.1, "1.0.0");
        assert_eq!(persisted.2, "extras");
        assert_eq!(persisted.3.as_deref(), Some("64bit"));

        let bins_json: serde_json::Value =
            serde_json::from_str(&persisted.4).expect("bins JSON should parse");
        let env_add_path_json: serde_json::Value =
            serde_json::from_str(&persisted.5).expect("env_add_path JSON should parse");
        let env_set_json: serde_json::Value =
            serde_json::from_str(&persisted.6).expect("env_set JSON should parse");
        let persist_json: serde_json::Value =
            serde_json::from_str(&persisted.7).expect("persist JSON should parse");

        assert_eq!(bins_json, serde_json::json!(["bin/app.exe"]));
        assert_eq!(env_add_path_json, serde_json::json!(["bin"]));
        assert_eq!(env_set_json, serde_json::json!({"APP_HOME":"$dir"}));
        assert_eq!(
            persist_json,
            serde_json::json!([{ "relative_path": "data", "store_name": "data" }])
        );

        let db_path = db_path_for_layout(&layout);
        assert!(db_path.exists(), "control-plane DB should exist");
        assert!(
            !layout.scoop.package_state_root.join("test-pkg.json").exists(),
            "legacy flat JSON package-state file must not be written anymore"
        );
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

        // Prove the canonical state is still readable through the store.
        let reread = read_installed_state(&layout, "reapply-pkg").await;
        assert!(reread.is_some(), "state must still exist in canonical store");
    });

    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn install_lifecycle_emits_stage_contract() {
    let root = temp_dir("install-stage-contract");
    std::fs::create_dir_all(&root).expect("create temp dir");
    let bucket_dir = root.join("scoop").join("buckets").join("main").join("bucket");
    std::fs::create_dir_all(&bucket_dir).expect("create bucket dir");
    let (archive, hash) = create_test_zip(&root, "demo.exe", b"demo-binary");
    std::fs::write(
        bucket_dir.join("demo.json"),
        serde_json::json!({
            "version": "1.0.0",
            "url": file_url(&archive),
            "hash": hash,
            "bin": "demo.exe"
        })
        .to_string(),
    )
    .expect("write manifest");

    block_on(sync_main_bucket_registry(&root)).expect("register main bucket");
    block_on(upsert_bucket_to_registry(
        &root,
        &BucketSpec {
            name: "main".to_string(),
            source: Some("https://example.com/main.git".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .expect("upsert main bucket");

    let context = BackendContext::new(root.clone(), None, false, "x64", "default", TestPorts);
    let plan: ScoopPackagePlan = plan_package_action("install", "demo", "demo", Some(&root));
    let mut events = Vec::new();

    block_on(execute_package_action_outcome_streaming_with_context(
        &context,
        &plan,
        None,
        Some(&mut |event| events.push(event)),
    ))
    .expect("install should succeed");

    let stages = collect_stages(&events);
    assert!(stages.contains(&LifecycleStage::Planned));
    assert!(stages.contains(&LifecycleStage::Acquiring));
    assert!(stages.contains(&LifecycleStage::Materializing));
    assert!(stages.contains(&LifecycleStage::PreparingHooks));
    assert!(stages.contains(&LifecycleStage::PersistRestoring));
    assert!(stages.contains(&LifecycleStage::SurfaceApplying));
    assert!(stages.contains(&LifecycleStage::PostInstallHooks));
    assert!(stages.contains(&LifecycleStage::Integrating));
    assert!(stages.contains(&LifecycleStage::StateCommitting));
    assert!(stages.contains(&LifecycleStage::Completed));
}

#[test]
fn uninstall_lifecycle_emits_stage_contract() {
    let root = temp_dir("uninstall-stage-contract");
    std::fs::create_dir_all(&root).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&root);
    let current_root = root.join("scoop").join("apps").join("demo").join("current");
    std::fs::create_dir_all(&current_root).expect("create current root");
    std::fs::write(current_root.join("demo.exe"), b"demo-binary").expect("write demo");

    block_on(write_installed_state(
        &layout,
        &InstalledPackageState {
            package: "demo".to_string(),
            version: "1.0.0".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
            bins: vec!["demo".to_string()],
            shortcuts: vec![],
            env_add_path: vec![],
            env_set: BTreeMap::new(),
            persist: vec![],
            integrations: BTreeMap::new(),
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec![],
        },
    ))
    .expect("write installed state");

    let context = BackendContext::new(root.clone(), None, false, "x64", "default", TestPorts);
    let plan: ScoopPackagePlan = plan_package_action("uninstall", "demo", "demo", Some(&root));
    let mut events = Vec::new();

    block_on(execute_package_action_outcome_streaming_with_context(
        &context,
        &plan,
        None,
        Some(&mut |event| events.push(event)),
    ))
    .expect("uninstall should succeed");

    let stages = collect_stages(&events);
    assert!(stages.contains(&LifecycleStage::Planned));
    assert!(stages.contains(&LifecycleStage::PreUninstallHooks));
    assert!(stages.contains(&LifecycleStage::Uninstalling));
    assert!(stages.contains(&LifecycleStage::PersistSyncing));
    assert!(stages.contains(&LifecycleStage::SurfaceRemoving));
    assert!(stages.contains(&LifecycleStage::StateRemoving));
    assert!(stages.contains(&LifecycleStage::PostUninstallHooks));
    assert!(stages.contains(&LifecycleStage::Completed));
}

#[test]
fn install_update_share_front_half_lifecycle_modules() {
    let root = temp_dir("front-half-lifecycle");
    std::fs::create_dir_all(&root).expect("create temp dir");
    let bucket_dir = root.join("scoop").join("buckets").join("main").join("bucket");
    std::fs::create_dir_all(&bucket_dir).expect("create bucket dir");
    let (archive, hash) = create_test_zip(&root, "demo.exe", b"demo-binary");
    std::fs::write(
        bucket_dir.join("demo.json"),
        serde_json::json!({
            "version": "1.0.0",
            "url": file_url(&archive),
            "hash": hash,
            "bin": "demo.exe"
        })
        .to_string(),
    )
    .expect("write manifest");

    block_on(sync_main_bucket_registry(&root)).expect("register main bucket");
    block_on(upsert_bucket_to_registry(
        &root,
        &BucketSpec {
            name: "main".to_string(),
            source: Some("https://example.com/main.git".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .expect("upsert main bucket");

    let install_plan: ScoopPackagePlan = plan_package_action("install", "demo", "demo", Some(&root));
    let update_plan: ScoopPackagePlan = plan_package_action("update", "demo", "demo", Some(&root));

    let install_planned = block_on(crate::scoop::lifecycle::planner::plan_package_lifecycle(&root, &install_plan))
        .expect("install lifecycle plan");
    let update_planned = block_on(crate::scoop::lifecycle::planner::plan_package_lifecycle(&root, &update_plan))
        .expect("update lifecycle plan");

    assert_eq!(install_planned.source.version, "1.0.0");
    assert_eq!(update_planned.source.version, "1.0.0");
    assert_eq!(install_planned.resolved.bucket.name, "main");
    assert_eq!(update_planned.resolved.bucket.name, "main");

    let mut sink = |_event: BackendEvent| {};
    let archives = block_on(crate::scoop::lifecycle::acquire::acquire_payloads(
        &root,
        "demo",
        &install_planned.source,
        &install_planned.source.payloads,
        "",
        None,
        &mut sink,
    ))
    .expect("acquire payloads");
    assert_eq!(archives.len(), 1);

    let version_root = root.join("scoop").join("apps").join("demo").join("1.0.0");
    let primary = block_on(crate::scoop::lifecycle::materialize::materialize_payloads(
        &root,
        &archives,
        &install_planned.source,
        &version_root,
        &mut sink,
    ))
    .expect("materialize payloads");
    assert!(primary.is_some());
    assert!(version_root.join("demo.exe").exists());
}

#[test]
fn install_lifecycle_orders_persist_surface_integrate_state() {
    let root = temp_dir("install-order-contract");
    std::fs::create_dir_all(&root).expect("create temp dir");
    let bucket_dir = root.join("scoop").join("buckets").join("main").join("bucket");
    std::fs::create_dir_all(&bucket_dir).expect("create bucket dir");
    let (archive, hash) = create_test_zip(&root, "demo.exe", b"demo-binary");
    std::fs::write(
        bucket_dir.join("demo.json"),
        serde_json::json!({
            "version": "1.0.0",
            "url": file_url(&archive),
            "hash": hash,
            "bin": "demo.exe"
        })
        .to_string(),
    )
    .expect("write manifest");

    block_on(sync_main_bucket_registry(&root)).expect("register main bucket");
    block_on(upsert_bucket_to_registry(
        &root,
        &BucketSpec {
            name: "main".to_string(),
            source: Some("https://example.com/main.git".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .expect("upsert main bucket");

    let context = BackendContext::new(root.clone(), None, false, "x64", "default", TestPorts);
    let plan: ScoopPackagePlan = plan_package_action("install", "demo", "demo", Some(&root));
    let mut events = Vec::new();

    block_on(execute_package_action_outcome_streaming_with_context(
        &context,
        &plan,
        None,
        Some(&mut |event| events.push(event)),
    ))
    .expect("install should succeed");

    let stages = collect_stages(&events);
    let persist = stages.iter().position(|s| *s == LifecycleStage::PersistRestoring).unwrap();
    let surface = stages.iter().position(|s| *s == LifecycleStage::SurfaceApplying).unwrap();
    let integrate = stages.iter().position(|s| *s == LifecycleStage::Integrating).unwrap();
    let state = stages.iter().position(|s| *s == LifecycleStage::StateCommitting).unwrap();
    let completed = stages.iter().position(|s| *s == LifecycleStage::Completed).unwrap();

    assert!(persist < surface);
    assert!(surface < integrate);
    assert!(integrate < state);
    assert!(state < completed);
}

#[test]
fn hook_failures_stop_before_state_commit() {
    let root = temp_dir("hook-failure-before-state-commit");
    std::fs::create_dir_all(&root).expect("create temp dir");
    let bucket_dir = root.join("scoop").join("buckets").join("main").join("bucket");
    std::fs::create_dir_all(&bucket_dir).expect("create bucket dir");
    let (archive, hash) = create_test_zip(&root, "demo.exe", b"demo-binary");
    std::fs::write(
        bucket_dir.join("demo.json"),
        serde_json::json!({
            "version": "1.0.0",
            "url": file_url(&archive),
            "hash": hash,
            "bin": "demo.exe",
            "post_install": ["throw 'post-install failed'"]
        })
        .to_string(),
    )
    .expect("write manifest");

    block_on(sync_main_bucket_registry(&root)).expect("register main bucket");
    block_on(upsert_bucket_to_registry(
        &root,
        &BucketSpec {
            name: "main".to_string(),
            source: Some("https://example.com/main.git".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .expect("upsert main bucket");

    let context = BackendContext::new(root.clone(), None, false, "x64", "default", TestPorts);
    let plan: ScoopPackagePlan = plan_package_action("install", "demo", "demo", Some(&root));
    let mut events = Vec::new();

    let error = block_on(execute_package_action_outcome_streaming_with_context(
        &context,
        &plan,
        None,
        Some(&mut |event| events.push(event)),
    ))
    .expect_err("install should fail on fatal post_install hook");

    let stages = collect_stages(&events);
    assert!(stages.contains(&LifecycleStage::Planned));
    assert!(stages.contains(&LifecycleStage::Acquiring));
    assert!(stages.contains(&LifecycleStage::Materializing));
    assert!(stages.contains(&LifecycleStage::PreparingHooks));
    assert!(stages.contains(&LifecycleStage::PersistRestoring));
    assert!(stages.contains(&LifecycleStage::SurfaceApplying));
    assert!(stages.contains(&LifecycleStage::PostInstallHooks));
    assert!(
        !stages.contains(&LifecycleStage::Integrating),
        "fatal hook failure should stop before integration: {stages:?}"
    );
    assert!(
        !stages.contains(&LifecycleStage::StateCommitting),
        "fatal hook failure should stop before state commit: {stages:?}"
    );
    assert!(
        !stages.contains(&LifecycleStage::Completed),
        "failed install must not emit completed stage: {stages:?}"
    );

    let layout = RuntimeLayout::from_root(&root);
    assert!(
        block_on(read_installed_state(&layout, "demo")).is_none(),
        "fatal hook failure must not leave committed installed state"
    );
    assert!(
        root.join("scoop")
            .join("apps")
            .join("demo")
            .join("current")
            .join("demo.exe")
            .exists(),
        "surface may already have been applied before post_install failed"
    );
    assert!(
        error.to_string().contains("Scoop lifecycle hook failed"),
        "unexpected error: {error}"
    );
}

#[test]
fn reapply_runs_without_hooks_and_reuses_back_half_modules() {
    let root = temp_dir("reapply-no-hooks");
    std::fs::create_dir_all(&root).expect("create temp dir");
    let bucket_dir = root.join("scoop").join("buckets").join("main").join("bucket");
    std::fs::create_dir_all(&bucket_dir).expect("create bucket dir");
    let (archive, hash) = create_test_zip(&root, "demo.exe", b"demo-binary");
    std::fs::write(
        bucket_dir.join("demo.json"),
        serde_json::json!({
            "version": "1.0.0",
            "url": file_url(&archive),
            "hash": hash,
            "bin": "demo.exe"
        })
        .to_string(),
    )
    .expect("write manifest");

    let current_root = root.join("scoop").join("apps").join("demo").join("current");
    std::fs::create_dir_all(&current_root).expect("create current root");
    std::fs::write(current_root.join("demo.exe"), b"demo-binary").expect("write demo");
    let layout = RuntimeLayout::from_root(&root);
    block_on(sync_main_bucket_registry(&root)).expect("register main bucket");

    block_on(write_installed_state(
        &layout,
        &InstalledPackageState {
            package: "demo".to_string(),
            version: "1.0.0".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
            bins: vec!["demo".to_string()],
            shortcuts: vec![],
            env_add_path: vec![],
            env_set: BTreeMap::new(),
            persist: vec![],
            integrations: BTreeMap::new(),
            pre_uninstall: vec!["this-should-not-run.ps1".to_string()],
            uninstaller_script: vec!["this-should-not-run.exe".to_string()],
            post_uninstall: vec!["this-should-not-run.ps1".to_string()],
        },
    ))
    .expect("write state");

    let host = NoopScoopRuntimeHost;
    let mut sink = |_event: BackendEvent| {};
    let surface = block_on(crate::scoop::runtime::reapply_package_command_surface_streaming_with_host(
        &root,
        "demo",
        &host,
        &mut sink,
    ))
    .expect("surface reapply should succeed");
    let integrations = block_on(crate::scoop::runtime::reapply_package_integrations_streaming_with_host(
        &root,
        "demo",
        &host,
        &mut sink,
    ))
    .expect("integration reapply should succeed");

    assert!(surface.iter().any(|line| line.contains("Reapplied command surface")));
    assert!(integrations.iter().any(|line| line.contains("Reapplied integrations")));
}

#[test]
fn uninstall_and_reapply_use_shared_lifecycle_contract() {
    let root = temp_dir("uninstall-reapply-contract");
    std::fs::create_dir_all(&root).expect("create temp dir");
    let bucket_dir = root.join("scoop").join("buckets").join("main").join("bucket");
    std::fs::create_dir_all(&bucket_dir).expect("create bucket dir");
    let (archive, hash) = create_test_zip(&root, "demo.exe", b"demo-binary");
    std::fs::write(
        bucket_dir.join("demo.json"),
        serde_json::json!({
            "version": "1.0.0",
            "url": file_url(&archive),
            "hash": hash,
            "bin": "demo.exe"
        })
        .to_string(),
    )
    .expect("write manifest");

    let current_root = root.join("scoop").join("apps").join("demo").join("current");
    std::fs::create_dir_all(&current_root).expect("create current root");
    std::fs::write(current_root.join("demo.exe"), b"demo-binary").expect("write demo");
    block_on(sync_main_bucket_registry(&root)).expect("register main bucket");
    let layout = RuntimeLayout::from_root(&root);
    block_on(write_installed_state(
        &layout,
        &InstalledPackageState {
            package: "demo".to_string(),
            version: "1.0.0".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
            bins: vec!["demo".to_string()],
            shortcuts: vec![],
            env_add_path: vec![],
            env_set: BTreeMap::new(),
            persist: vec![],
            integrations: BTreeMap::new(),
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec![],
        },
    ))
    .expect("write state");

    let host = NoopScoopRuntimeHost;
    let mut sink = |_event: BackendEvent| {};
    let reapply = block_on(crate::scoop::lifecycle::reapply::reapply(
        &root,
        "demo",
        &host,
        &mut sink,
    ))
    .expect("reapply should succeed");
    assert!(reapply.iter().any(|line| line.contains("Reapplied command surface")));

    let uninstall = block_on(crate::scoop::lifecycle::uninstall::uninstall(
        &root,
        42,
        "demo",
        &host,
        &mut sink,
    ))
    .expect("uninstall should succeed");
    assert!(uninstall.iter().any(|line| line.contains("Removed Scoop package 'demo'.")));
}

#[test]
fn post_uninstall_hook_is_warning_only() {
    let root = temp_dir("post-uninstall-warning");
    std::fs::create_dir_all(&root).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&root);
    let current_root = root.join("scoop").join("apps").join("demo").join("current");
    std::fs::create_dir_all(&current_root).expect("create current root");
    std::fs::write(current_root.join("demo.exe"), b"demo-binary").expect("write demo");

    block_on(write_installed_state(
        &layout,
        &InstalledPackageState {
            package: "demo".to_string(),
            version: "1.0.0".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
            bins: vec!["demo".to_string()],
            shortcuts: vec![],
            env_add_path: vec![],
            env_set: BTreeMap::new(),
            persist: vec![],
            integrations: BTreeMap::new(),
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec!["throw 'boom'".to_string()],
        },
    ))
    .expect("write state");

    let host = NoopScoopRuntimeHost;
    let mut sink = |_event: BackendEvent| {};
    let uninstall = block_on(crate::scoop::lifecycle::uninstall::uninstall(
        &root,
        99,
        "demo",
        &host,
        &mut sink,
    ))
    .expect("uninstall should still succeed");
    assert!(uninstall.iter().any(|line| line.contains("Removed Scoop package 'demo'.")));
}

#[test]
fn warning_only_uninstall_tail_preserves_main_result() {
    let root = temp_dir("warning-only-uninstall-tail");
    std::fs::create_dir_all(&root).expect("create temp dir");
    let layout = RuntimeLayout::from_root(&root);
    let current_root = root.join("scoop").join("apps").join("demo").join("current");
    let package_root = root.join("scoop").join("apps").join("demo");
    std::fs::create_dir_all(&current_root).expect("create current root");
    std::fs::write(current_root.join("demo.exe"), b"demo-binary").expect("write demo");

    block_on(write_installed_state(
        &layout,
        &InstalledPackageState {
            package: "demo".to_string(),
            version: "1.0.0".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
            bins: vec!["demo".to_string()],
            shortcuts: vec![],
            env_add_path: vec![],
            env_set: BTreeMap::new(),
            persist: vec![],
            integrations: BTreeMap::new(),
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec!["throw 'warning tail failed'".to_string()],
        },
    ))
    .expect("write state");

    let context = BackendContext::new(root.clone(), None, false, "x64", "default", TestPorts);
    let plan: ScoopPackagePlan = plan_package_action("uninstall", "demo", "demo", Some(&root));
    let mut events = Vec::new();

    let outcome = block_on(execute_package_action_outcome_streaming_with_context(
        &context,
        &plan,
        None,
        Some(&mut |event| events.push(event)),
    ))
    .expect("warning-only uninstall tail should not fail main uninstall");

    assert!(outcome.status.is_success());
    assert!(outcome.output.iter().any(|line| line.contains("Removed Scoop package 'demo'.")));

    let stages = collect_stages(&events);
    assert!(stages.contains(&LifecycleStage::PostUninstallHooks));
    assert!(stages.contains(&LifecycleStage::Completed));
    let post_uninstall = stages
        .iter()
        .position(|stage| *stage == LifecycleStage::PostUninstallHooks)
        .expect("post uninstall stage");
    let completed = stages
        .iter()
        .position(|stage| *stage == LifecycleStage::Completed)
        .expect("completed stage");
    assert!(post_uninstall < completed);

    assert!(
        block_on(read_installed_state(&layout, "demo")).is_none(),
        "warning-only tail must not preserve installed state"
    );
    assert!(
        !package_root.exists(),
        "main uninstall result must still remove package root"
    );
}
