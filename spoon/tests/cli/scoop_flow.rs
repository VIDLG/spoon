#[path = "../common/mod.rs"]
mod common;

use common::assertions::{assert_contains, assert_ok};
use common::cli::{run_in_home, run_in_home_without_test_mode};
use common::scoop::{create_zip_archive, file_url};
use common::setup::create_configured_home;
use spoon::config;
use spoon_backend::layout::RuntimeLayout;
use spoon_backend::scoop::{InstalledPackageState, write_installed_state};

#[test]
fn scoop_status_lists_buckets_and_installed_packages() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let bucket_root = tool_root
        .join("scoop")
        .join("buckets")
        .join("main")
        .join("bucket");
    std::fs::create_dir_all(&bucket_root).unwrap();
    std::fs::write(bucket_root.join("jq.json"), r#"{ "version": "1.8.1" }"#).unwrap();
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &tool_root,
        &spoon::service::scoop::BucketSpec {
            name: "main".to_string(),
            source: Some("https://github.com/ScoopInstaller/Main".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();
    let layout = RuntimeLayout::from_root(&tool_root);
    spoon::runtime::test_block_on(write_installed_state(
        &layout,
        &InstalledPackageState {
            package: "jq".to_string(),
            version: "1.8.1".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
            bins: vec![],
            shortcuts: vec![],
            env_add_path: vec![],
            env_set: std::collections::BTreeMap::new(),
            persist: vec![],
            integrations: std::collections::BTreeMap::new(),
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec![],
        },
    ))
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["scoop", "status"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Scoop runtime:");
    assert_contains(&stdout, "Buckets:");
    assert_contains(&stdout, "Installed packages:");
    assert_contains(
        &stdout,
        "main | master | https://github.com/ScoopInstaller/Main",
    );
    assert_contains(&stdout, "jq | 1.8.1");
}

#[test]
fn scoop_list_lists_installed_packages() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let layout = RuntimeLayout::from_root(&tool_root);
    spoon::runtime::test_block_on(write_installed_state(
        &layout,
        &InstalledPackageState {
            package: "jq".to_string(),
            version: "1.8.1".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
            bins: vec![],
            shortcuts: vec![],
            env_add_path: vec![],
            env_set: std::collections::BTreeMap::new(),
            persist: vec![],
            integrations: std::collections::BTreeMap::new(),
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec![],
        },
    ))
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["scoop", "list"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "jq | 1.8.1");
}

#[test]
fn scoop_info_prints_manifest_and_install_details() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let bucket_root = tool_root
        .join("scoop")
        .join("buckets")
        .join("main")
        .join("bucket");
    std::fs::create_dir_all(&bucket_root).unwrap();
    std::fs::write(
        bucket_root.join("jq.json"),
        r#"{
            "version": "1.8.1",
            "description": "Command-line JSON processor",
            "homepage": "https://jqlang.org",
            "license": "MIT",
            "depends": ["oniguruma"],
            "bin": ["jq.exe"],
            "persist": ["config"],
            "env_add_path": ["bin"]
        }"#,
    )
    .unwrap();
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &tool_root,
        &spoon::service::scoop::BucketSpec {
            name: "main".to_string(),
            source: Some("https://github.com/ScoopInstaller/Main".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();
    let layout = RuntimeLayout::from_root(&tool_root);
    spoon::runtime::test_block_on(write_installed_state(
        &layout,
        &InstalledPackageState {
            package: "jq".to_string(),
            version: "1.8.1".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
            bins: vec!["jq".to_string()],
            shortcuts: vec![],
            env_add_path: vec!["bin".to_string()],
            env_set: std::collections::BTreeMap::from([(
                "JQ_HOME".to_string(),
                "current".to_string(),
            )]),
            persist: vec![spoon_backend::scoop::PersistEntry {
                relative_path: "config".to_string(),
                store_name: "config".to_string(),
            }],
            integrations: std::collections::BTreeMap::new(),
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec![],
        },
    ))
    .unwrap();
    std::fs::create_dir_all(
        layout.scoop.package_current_root("jq"),
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["scoop", "info", "jq"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Package:");
    assert_contains(&stdout, "  name: jq");
    assert_contains(&stdout, "  bucket: main");
    assert_contains(&stdout, "Install:");
    assert_contains(&stdout, "  installed: yes");
    assert_contains(&stdout, "  description: Command-line JSON processor");
    assert_contains(&stdout, "  homepage: https://jqlang.org");
    assert_contains(&stdout, "  license: MIT");
    assert_contains(&stdout, "  depends: [\"oniguruma\"]");
    assert_contains(&stdout, "  manifest:");
    assert_contains(&stdout, "  current:");
    assert_contains(&stdout, "  persist root:");
    assert_contains(&stdout, "Integration:");
    assert_contains(&stdout, "Commands:");
    assert_contains(&stdout, "shims: jq");
    assert_contains(&stdout, "Environment:");
    assert_contains(&stdout, "add_path:");
    assert_contains(&stdout, "set: JQ_HOME=current");
    assert_contains(&stdout, "persist: [\"config\"]");
}

#[test]
fn scoop_cat_prints_raw_manifest_json() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let bucket_root = tool_root
        .join("scoop")
        .join("buckets")
        .join("main")
        .join("bucket");
    std::fs::create_dir_all(&bucket_root).unwrap();
    std::fs::write(
        bucket_root.join("jq.json"),
        "{\n  \"version\": \"1.8.1\",\n  \"description\": \"Command-line JSON processor\"\n}\n",
    )
    .unwrap();
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &tool_root,
        &spoon::service::scoop::BucketSpec {
            name: "main".to_string(),
            source: Some("https://github.com/ScoopInstaller/Main".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["scoop", "cat", "jq"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "\"version\": \"1.8.1\"");
    assert_contains(&stdout, "\"description\": \"Command-line JSON processor\"");
}

#[test]
fn scoop_prefix_prints_current_install_root() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let current_root = RuntimeLayout::from_root(&tool_root)
        .scoop
        .package_current_root("jq");
    std::fs::create_dir_all(&current_root).unwrap();
    let layout = RuntimeLayout::from_root(&tool_root);
    spoon::runtime::test_block_on(write_installed_state(
        &layout,
        &InstalledPackageState {
            package: "jq".to_string(),
            version: "1.8.1".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
            bins: vec![],
            shortcuts: vec![],
            env_add_path: vec![],
            env_set: std::collections::BTreeMap::new(),
            persist: vec![],
            integrations: std::collections::BTreeMap::new(),
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec![],
        },
    ))
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["scoop", "prefix", "jq"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let expected = current_root.display().to_string();
    let normalized_stdout = stdout.replace("\\\\", "\\");
    assert!(
        normalized_stdout.contains(&expected),
        "expected to find prefix path in stdout:\nstdout: {stdout}\nexpected: {expected}"
    );
}

#[test]
fn scoop_install_package_uses_no_update_scoop() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) = run_in_home(&["scoop", "install", "uv"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(
        &stdout,
        "Planned Spoon package action (Scoop): install uv --no-update-scoop",
    );
}

#[test]
fn scoop_install_package_reports_resolved_bucket_source() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    std::fs::create_dir_all(
        tool_root
            .join("scoop")
            .join("buckets")
            .join("extras")
            .join("bucket"),
    )
    .unwrap();
    std::fs::write(
        tool_root
            .join("scoop")
            .join("buckets")
            .join("extras")
            .join("bucket")
            .join("demo.json"),
        r#"{ "version": "1.0.0" }"#,
    )
    .unwrap();
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &tool_root,
        &spoon::service::scoop::BucketSpec {
            name: "extras".to_string(),
            source: Some("https://example.com/extras".to_string()),
            branch: Some("main".to_string()),
        },
    ))
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["scoop", "install", "demo"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(
        &stdout,
        "Resolved Scoop package 'demo' from bucket 'extras'",
    );
    assert_contains(
        &stdout,
        "Planned Spoon package action (Scoop): install demo --no-update-scoop",
    );
}

#[test]
fn scoop_install_package_runs_spoon_owned_runtime_for_real() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let scoop_root = tool_root.join("scoop");
    let bucket_source = temp_home.join("extras-bucket");
    std::fs::create_dir_all(bucket_source.join("bucket")).unwrap();
    let (archive, hash) = create_zip_archive(&temp_home, "demo.exe", b"fake exe");
    std::fs::write(
        bucket_source.join("bucket").join("demo.json"),
        format!(
            "{{ \"version\": \"1.0.0\", \"url\": \"{}\", \"hash\": \"{}\", \"bin\": \"demo.exe\" }}",
            file_url(&archive),
            hash
        ),
    )
    .unwrap();
    let bucket_source_str = bucket_source.display().to_string();
    let (add_ok, add_stdout, add_stderr) = run_in_home(
        &["scoop", "bucket", "add", "extras", &bucket_source_str, "--branch", "main"],
        &temp_home,
        &[],
    );
    assert_ok(add_ok, &add_stdout, &add_stderr);

    let (ok, stdout, stderr) =
        run_in_home_without_test_mode(&["scoop", "install", "demo"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(
        &stdout,
        "Resolved Scoop package 'demo' from bucket 'extras'",
    );
    assert_contains(
        &stdout,
        "Planned Spoon package action (Scoop): install demo --no-update-scoop",
    );
    assert_contains(&stdout, "Installed Scoop package 'demo' into");
    assert!(
        scoop_root
            .join("apps")
            .join("demo")
            .join("current")
            .join("demo.exe")
            .exists()
    );
    assert!(
        config::shims_root_from(&tool_root)
            .join("demo.cmd")
            .exists()
    );
}

#[test]
fn scoop_bucket_commands_manage_registered_buckets() {
    let env = create_configured_home();
    let temp_home = env.home;

    let repo_dir = temp_home.join("extras-repo");
    std::fs::create_dir_all(repo_dir.join("bucket")).unwrap();
    std::fs::write(
        repo_dir.join("bucket").join("demo.json"),
        r#"{ "version": "1.0.0" }"#,
    )
    .unwrap();
    let repo_source = repo_dir.display().to_string();
    let (add_ok, add_stdout, add_stderr) = run_in_home(
        &[
            "scoop",
            "bucket",
            "add",
            "extras",
            &repo_source,
            "--branch",
            "main",
        ],
        &temp_home,
        &[],
    );
    assert_ok(add_ok, &add_stdout, &add_stderr);

    let (list_ok, list_stdout, list_stderr) =
        run_in_home(&["scoop", "bucket", "list"], &temp_home, &[]);
    assert_ok(list_ok, &list_stdout, &list_stderr);
    assert_contains(&list_stdout, "extras | main |");

    let (remove_ok, remove_stdout, remove_stderr) =
        run_in_home(&["scoop", "bucket", "remove", "extras"], &temp_home, &[]);
    assert_ok(remove_ok, &remove_stdout, &remove_stderr);
    assert_contains(&remove_stdout, "Removed bucket 'extras'.");
}

#[test]
fn scoop_bucket_update_refreshes_local_bucket_contents() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    let repo_dir = temp_home.join("extras-repo");
    std::fs::create_dir_all(repo_dir.join("bucket")).unwrap();
    std::fs::write(
        repo_dir.join("bucket").join("demo.json"),
        r#"{ "version": "1.0.0" }"#,
    )
    .unwrap();
    let repo_source = repo_dir.display().to_string();
    let (add_ok, add_stdout, add_stderr) = run_in_home(
        &[
            "scoop",
            "bucket",
            "add",
            "extras",
            &repo_source,
            "--branch",
            "main",
        ],
        &temp_home,
        &[],
    );
    assert_ok(add_ok, &add_stdout, &add_stderr);

    std::fs::write(
        repo_dir.join("bucket").join("demo.json"),
        r#"{ "version": "2.0.0" }"#,
    )
    .unwrap();
    let (update_ok, update_stdout, update_stderr) =
        run_in_home(&["scoop", "bucket", "update", "extras"], &temp_home, &[]);
    assert_ok(update_ok, &update_stdout, &update_stderr);
    assert_contains(&update_stdout, "Updated bucket 'extras'.");

    let cloned_manifest = tool_root
        .join("scoop")
        .join("buckets")
        .join("extras")
        .join("bucket")
        .join("demo.json");
    let cloned = std::fs::read_to_string(cloned_manifest).unwrap();
    assert!(cloned.contains("\"2.0.0\""), "cloned manifest: {cloned}");
}
