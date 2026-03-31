#[path = "../common/mod.rs"]
mod common;

use common::assertions::{
    assert_contains, assert_contains_any, assert_not_ok, assert_ok, assert_path_exists,
    assert_path_missing,
};
use common::cli::{run_in_home, run_in_home_without_test_mode};
use common::scoop::{
    create_demo_archive_with_config, create_zip_archive, create_zip_archive_with_entries, file_url,
    register_local_bucket, update_bucket, write_demo_manifest, write_manifest_text,
};
use common::setup::{create_configured_home, create_configured_home_with_proxy};
use sha2::{Digest, Sha256};
use spoon::config;
use spoon_backend::layout::RuntimeLayout;
use spoon_backend::scoop::{read_installed_state, load_buckets_from_registry};

#[test]
fn spoon_scoop_bucket_cli_handles_local_bucket_lifecycle() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    let bucket_source = temp_home.join("extras-bucket");
    let (archive, hash) = create_zip_archive(&temp_home, "demo.exe", b"v1");
    write_demo_manifest(&bucket_source, "1.0.0", &archive, &hash);

    register_local_bucket(&temp_home, "extras", &bucket_source);

    let (list_ok, list_stdout, list_stderr) =
        run_in_home(&["scoop", "bucket", "list"], &temp_home, &[]);
    assert_ok(list_ok, &list_stdout, &list_stderr);
    assert_contains(&list_stdout, "extras | main |");

    let (archive_v2, hash_v2) = create_zip_archive(&temp_home, "demo.exe", b"v2");
    write_demo_manifest(&bucket_source, "2.0.0", &archive_v2, &hash_v2);

    update_bucket(&temp_home, "extras");

    let managed_manifest = tool_root
        .join("scoop")
        .join("buckets")
        .join("extras")
        .join("bucket")
        .join("demo.json");
    let managed_manifest_text = std::fs::read_to_string(managed_manifest).unwrap();
    assert!(
        managed_manifest_text.contains("\"2.0.0\""),
        "managed manifest: {managed_manifest_text}"
    );

    let (remove_ok, remove_stdout, remove_stderr) =
        run_in_home(&["scoop", "bucket", "remove", "extras"], &temp_home, &[]);
    assert_ok(remove_ok, &remove_stdout, &remove_stderr);
    assert_contains(&remove_stdout, "Removed bucket 'extras'.");
}

#[test]
fn spoon_scoop_search_cli_lists_matching_packages_from_registered_buckets() {
    let env = create_configured_home();
    let temp_home = env.home;
    let _tool_root = env.root;

    let bucket_source = temp_home.join("search-bucket");
    let (archive, hash) = create_zip_archive(&temp_home, "demo.exe", b"search");
    write_manifest_text(
        &bucket_source,
        "demo-search",
        &format!(
            "{{\n  \"version\": \"1.2.3\",\n  \"description\": \"Demo search package\",\n  \"url\": \"{}\",\n  \"hash\": \"{hash}\",\n  \"bin\": \"demo.exe\"\n}}",
            file_url(&archive)
        ),
    );
    write_manifest_text(
        &bucket_source,
        "other",
        r#"{ "version": "0.1.0", "description": "Unrelated package" }"#,
    );

    register_local_bucket(&temp_home, "extras", &bucket_source);

    let (search_ok, search_stdout, search_stderr) =
        run_in_home(&["scoop", "search", "demo"], &temp_home, &[]);
    assert_ok(search_ok, &search_stdout, &search_stderr);
    assert_contains(
        &search_stdout,
        "demo-search | 1.2.3 | extras | Demo search package",
    );
    assert!(
        !search_stdout.contains("other |"),
        "stdout: {search_stdout}"
    );
}

#[test]
fn spoon_scoop_bucket_cli_requires_source_for_unknown_bucket() {
    let env = create_configured_home();
    let temp_home = env.home;
    let _tool_root = env.root;

    let (ok, stdout, stderr) = run_in_home(
        &["scoop", "bucket", "add", "custom-bucket"],
        &temp_home,
        &[],
    );
    assert_not_ok(ok, &stdout, &stderr);
    assert!(
        stdout.contains("requires an explicit source")
            || stderr.contains("requires an explicit source"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn spoon_scoop_package_cli_bootstraps_default_main_bucket() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    let bucket_source = temp_home.join("main-bucket");
    let (archive, hash) = create_zip_archive(&temp_home, "demo.exe", b"default-main");
    write_manifest_text(
        &bucket_source,
        "demo",
        &format!(
            "{{\n  \"version\": \"1.0.0\",\n  \"url\": \"{}\",\n  \"hash\": \"{hash}\",\n  \"bin\": \"demo.exe\"\n}}",
            file_url(&archive)
        ),
    );

    let source = bucket_source.display().to_string();
    let (install_ok, install_stdout, install_stderr) = run_in_home_without_test_mode(
        &["scoop", "install", "demo"],
        &temp_home,
        &[("SPOON_TEST_SCOOP_BUCKET_MAIN_SOURCE", &source)],
    );
    assert!(
        install_ok,
        "stdout: {install_stdout}\nstderr: {install_stderr}"
    );
    assert_contains(
        &install_stdout,
        "Resolved Scoop package 'demo' from bucket 'main'",
    );
    assert_contains(
        &install_stdout,
        "Ensured Spoon shims are available on PATH:",
    );
    assert!(
        tool_root
            .join("scoop")
            .join("apps")
            .join("demo")
            .join("current")
            .join("demo.exe")
            .exists(),
        "demo should be installed from bootstrapped main bucket"
    );
    let buckets = spoon::runtime::test_block_on(load_buckets_from_registry(&tool_root));
    assert!(buckets.iter().any(|bucket| bucket.name == "main"));
}

#[test]
fn spoon_scoop_package_cli_handles_install_update_uninstall_with_local_bucket_source() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    let bucket_source = temp_home.join("extras-bucket");
    let (archive_v1, hash_v1) = create_zip_archive(&temp_home, "demo.exe", b"demo-v1");
    write_demo_manifest(&bucket_source, "1.0.0", &archive_v1, &hash_v1);

    register_local_bucket(&temp_home, "extras", &bucket_source);
    let buckets = spoon::runtime::test_block_on(load_buckets_from_registry(&tool_root));
    assert!(buckets.iter().any(|bucket| bucket.name == "extras"));

    let (install_ok, install_stdout, install_stderr) =
        run_in_home_without_test_mode(&["scoop", "install", "demo"], &temp_home, &[]);
    assert!(
        install_ok,
        "stdout: {install_stdout}\nstderr: {install_stderr}"
    );
    assert_contains(
        &install_stdout,
        "Resolved Scoop package 'demo' from bucket 'extras'",
    );
    assert_contains(
        &install_stdout,
        "Ensured Spoon shims are available on PATH:",
    );
    assert_contains(&install_stdout, "Installed Scoop package 'demo' into");

    let current_exe = tool_root
        .join("scoop")
        .join("apps")
        .join("demo")
        .join("current")
        .join("demo.exe");
    let versioned_exe = tool_root
        .join("scoop")
        .join("apps")
        .join("demo")
        .join("1.0.0")
        .join("demo.exe");
    let shim = config::shims_root_from(&tool_root).join("demo.cmd");
    assert_path_exists(&current_exe);
    assert_path_exists(&versioned_exe);
    assert_path_exists(&shim);
    let layout = RuntimeLayout::from_root(&tool_root);
    let state = spoon::runtime::test_block_on(read_installed_state(&layout, "demo")).unwrap();
    assert_eq!(state.version, "1.0.0");

    let (archive_v2, hash_v2) = create_zip_archive(&temp_home, "demo.exe", b"demo-v2");
    write_demo_manifest(&bucket_source, "2.0.0", &archive_v2, &hash_v2);
    update_bucket(&temp_home, "extras");

    let (update_ok, update_stdout, update_stderr) =
        run_in_home_without_test_mode(&["scoop", "update", "demo"], &temp_home, &[]);
    assert_ok(update_ok, &update_stdout, &update_stderr);
    assert_contains_any(
        &update_stdout,
        &[
            "Installed Scoop package 'demo' into",
            "Downloaded archive into",
            "Reused cached archive",
        ],
    );
    let updated_state = spoon::runtime::test_block_on(read_installed_state(&layout, "demo")).unwrap();
    assert_eq!(updated_state.version, "2.0.0");

    let (uninstall_ok, uninstall_stdout, uninstall_stderr) =
        run_in_home_without_test_mode(&["scoop", "uninstall", "demo"], &temp_home, &[]);
    assert!(
        uninstall_ok,
        "stdout: {uninstall_stdout}\nstderr: {uninstall_stderr}"
    );
    assert_contains(&uninstall_stdout, "Removed Scoop package 'demo'.");
    assert_path_missing(&current_exe);
    assert_path_missing(&shim);
    assert!(spoon::runtime::test_block_on(read_installed_state(&layout, "demo")).is_none());
}

#[test]
fn spoon_scoop_package_cli_handles_single_file_manifest_with_architecture_override() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    let bucket_source = temp_home.join("single-file-bucket");
    std::fs::create_dir_all(bucket_source.join("bucket")).unwrap();
    let exe_path = temp_home.join("solo.exe");
    std::fs::write(&exe_path, b"single-file-demo").unwrap();
    let bytes = std::fs::read(&exe_path).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = format!("{:x}", hasher.finalize());
    std::fs::write(
        bucket_source.join("bucket").join("solo.json"),
        format!(
            "{{\n  \"version\": \"1.0.0\",\n  \"architecture\": {{\n    \"64bit\": {{\n      \"url\": \"{}\",\n      \"hash\": \"{}\"\n    }}\n  }},\n  \"bin\": [[\"solo.exe\", \"solo\"]]\n}}",
            file_url(&exe_path),
            hash
        ),
    )
    .unwrap();

    register_local_bucket(&temp_home, "extras", &bucket_source);

    let (install_ok, install_stdout, install_stderr) =
        run_in_home_without_test_mode(&["scoop", "install", "solo"], &temp_home, &[]);
    assert!(
        install_ok,
        "stdout: {install_stdout}\nstderr: {install_stderr}"
    );
    assert!(
        install_stdout.contains("Installed Scoop package 'solo' into"),
        "stdout: {install_stdout}"
    );
    let installed = tool_root
        .join("scoop")
        .join("apps")
        .join("solo")
        .join("current")
        .join("solo.exe");
    let shim = config::shims_root_from(&tool_root).join("solo.cmd");
    assert_path_exists(&installed);
    assert_path_exists(&shim);
    let layout = RuntimeLayout::from_root(&tool_root);
    let state = spoon::runtime::test_block_on(read_installed_state(&layout, "solo")).unwrap();
    assert_eq!(state.package, "solo");
}

#[test]
fn spoon_scoop_package_cli_writes_shim_with_tuple_args() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    let bucket_source = temp_home.join("shim-args-bucket");
    std::fs::create_dir_all(bucket_source.join("bucket")).unwrap();
    let (archive, hash) = create_zip_archive(&temp_home, "demo.exe", b"shim-args");
    std::fs::write(
        bucket_source.join("bucket").join("demo.json"),
        format!(
            "{{\n  \"version\": \"1.0.0\",\n  \"url\": \"{}\",\n  \"hash\": \"{hash}\",\n  \"bin\": [[\"demo.exe\", \"demo\", \"--wrapped\"]]\n}}",
            file_url(&archive)
        ),
    )
    .unwrap();

    register_local_bucket(&temp_home, "extras", &bucket_source);

    let (install_ok, install_stdout, install_stderr) =
        run_in_home_without_test_mode(&["scoop", "install", "demo"], &temp_home, &[]);
    assert!(
        install_ok,
        "stdout: {install_stdout}\nstderr: {install_stderr}"
    );

    let shim =
        std::fs::read_to_string(config::shims_root_from(&tool_root).join("demo.cmd")).unwrap();
    assert!(shim.contains("--wrapped %*"), "shim: {shim}");
}

#[test]
fn spoon_scoop_package_cli_preserves_persisted_files_across_update() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    let bucket_source = temp_home.join("persist-bucket");
    std::fs::create_dir_all(bucket_source.join("bucket")).unwrap();
    let (archive_v1, hash_v1) =
        create_demo_archive_with_config(&temp_home, b"demo-v1", br#"{"theme":"dark"}"#);
    std::fs::write(
        bucket_source.join("bucket").join("demo.json"),
        format!(
            "{{\n  \"version\": \"1.0.0\",\n  \"url\": \"{}\",\n  \"hash\": \"{hash_v1}\",\n  \"bin\": \"bin/demo.exe\",\n  \"env_add_path\": [\"bin\"],\n  \"env_set\": {{\"DEMO_MODE\": \"1\"}},\n  \"persist\": [\"config\"]\n}}",
            file_url(&archive_v1)
        ),
    )
    .unwrap();

    register_local_bucket(&temp_home, "extras", &bucket_source);

    let (install_ok, install_stdout, install_stderr) =
        run_in_home_without_test_mode(&["scoop", "install", "demo"], &temp_home, &[]);
    assert_ok(install_ok, &install_stdout, &install_stderr);

    let persisted_file = tool_root
        .join("scoop")
        .join("apps")
        .join("demo")
        .join("current")
        .join("config")
        .join("settings.json");
    std::fs::create_dir_all(persisted_file.parent().unwrap()).unwrap();
    std::fs::write(&persisted_file, br#"{"theme":"light","user":"kept"}"#).unwrap();

    let (archive_v2, hash_v2) =
        create_demo_archive_with_config(&temp_home, b"demo-v2", br#"{"theme":"blue"}"#);
    std::fs::write(
        bucket_source.join("bucket").join("demo.json"),
        format!(
            "{{\n  \"version\": \"2.0.0\",\n  \"url\": \"{}\",\n  \"hash\": \"{hash_v2}\",\n  \"bin\": \"bin/demo.exe\",\n  \"env_add_path\": [\"bin\"],\n  \"env_set\": {{\"DEMO_MODE\": \"2\"}},\n  \"persist\": [\"config\"]\n}}",
            file_url(&archive_v2)
        ),
    )
    .unwrap();

    update_bucket(&temp_home, "extras");

    let (update_ok, update_stdout, update_stderr) =
        run_in_home_without_test_mode(&["scoop", "update", "demo"], &temp_home, &[]);
    assert_ok(update_ok, &update_stdout, &update_stderr);

    let persisted_text = std::fs::read_to_string(&persisted_file).unwrap();
    assert!(
        persisted_text.contains("\"light\"") && persisted_text.contains("\"kept\""),
        "persisted config should survive update: {persisted_text}"
    );

    let layout = RuntimeLayout::from_root(&tool_root);
    let state = spoon::runtime::test_block_on(read_installed_state(&layout, "demo")).unwrap();
    assert_eq!(state.env_add_path, vec!["bin".to_string()]);
    assert_eq!(
        state.env_set.get("DEMO_MODE").map(String::as_str),
        Some("2")
    );
    assert_eq!(state.persist.len(), 1);
    assert_eq!(state.persist[0].relative_path, "config");
}

#[test]
fn spoon_scoop_package_cli_installs_dependencies() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    let bucket_source = temp_home.join("depends-bucket");
    let (dep_archive, dep_hash) = create_zip_archive(&temp_home, "dep.exe", b"dep");
    let (app_archive, app_hash) = create_zip_archive(&temp_home, "app.exe", b"app");
    write_manifest_text(
        &bucket_source,
        "dep",
        &format!(
            "{{\n  \"version\": \"1.0.0\",\n  \"url\": \"{}\",\n  \"hash\": \"{dep_hash}\",\n  \"bin\": \"dep.exe\"\n}}",
            file_url(&dep_archive)
        ),
    );
    write_manifest_text(
        &bucket_source,
        "app",
        &format!(
            "{{\n  \"version\": \"1.0.0\",\n  \"depends\": [\"dep\"],\n  \"url\": \"{}\",\n  \"hash\": \"{app_hash}\",\n  \"bin\": \"app.exe\"\n}}",
            file_url(&app_archive)
        ),
    );

    register_local_bucket(&temp_home, "extras", &bucket_source);

    let (install_ok, install_stdout, install_stderr) =
        run_in_home_without_test_mode(&["scoop", "install", "app"], &temp_home, &[]);
    assert!(
        install_ok,
        "stdout: {install_stdout}\nstderr: {install_stderr}"
    );
    assert_path_exists(
        &tool_root
            .join("scoop")
            .join("apps")
            .join("dep")
            .join("current")
            .join("dep.exe"),
    );
    assert_path_exists(
        &tool_root
            .join("scoop")
            .join("apps")
            .join("app")
            .join("current")
            .join("app.exe"),
    );
}

#[test]
fn spoon_scoop_package_cli_handles_multiple_payload_archives() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    let bucket_source = temp_home.join("multi-payload-bucket");
    let (core_archive, core_hash) = create_zip_archive(&temp_home, "demo.exe", b"multi-core");
    let (docs_archive, docs_hash) =
        create_zip_archive_with_entries(&temp_home, "docs.zip", &[("README.txt", b"multi-docs")]);
    write_manifest_text(
        &bucket_source,
        "demo",
        &format!(
            "{{\n  \"version\": \"1.0.0\",\n  \"url\": [\"{}\", \"{}\"],\n  \"hash\": [\"{core_hash}\", \"{docs_hash}\"],\n  \"bin\": \"demo.exe\"\n}}",
            file_url(&core_archive),
            file_url(&docs_archive)
        ),
    );

    register_local_bucket(&temp_home, "extras", &bucket_source);

    let (install_ok, install_stdout, install_stderr) =
        run_in_home_without_test_mode(&["scoop", "install", "demo"], &temp_home, &[]);
    assert!(
        install_ok,
        "stdout: {install_stdout}\nstderr: {install_stderr}"
    );
    let current_root = tool_root
        .join("scoop")
        .join("apps")
        .join("demo")
        .join("current");
    assert_path_exists(&current_root.join("demo.exe"));
    assert_path_exists(&current_root.join("README.txt"));
}

#[test]
fn spoon_scoop_package_cli_creates_and_removes_shortcuts() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    let bucket_source = temp_home.join("shortcuts-bucket");
    let (archive, hash) = create_zip_archive(&temp_home, "demo.exe", b"shortcut-demo");
    write_manifest_text(
        &bucket_source,
        "demo",
        &format!(
            "{{\n  \"version\": \"1.0.0\",\n  \"url\": \"{}\",\n  \"hash\": \"{hash}\",\n  \"bin\": \"demo.exe\",\n  \"shortcuts\": [[\"demo.exe\", \"Demo Shortcut\", \"--gui\", \"demo.exe\"]]\n}}",
            file_url(&archive)
        ),
    );

    register_local_bucket(&temp_home, "extras", &bucket_source);

    let (install_ok, install_stdout, install_stderr) =
        run_in_home_without_test_mode(&["scoop", "install", "demo"], &temp_home, &[]);
    assert!(
        install_ok,
        "stdout: {install_stdout}\nstderr: {install_stderr}"
    );
    assert!(
        install_stdout.contains("Created shortcuts: Demo Shortcut"),
        "stdout: {install_stdout}"
    );

    let shortcut = temp_home
        .join(".spoon-test-startmenu")
        .join("Spoon Apps")
        .join("Demo Shortcut.lnk");
    assert_path_exists(&shortcut);

    let layout = RuntimeLayout::from_root(&tool_root);
    let state = spoon::runtime::test_block_on(read_installed_state(&layout, "demo")).unwrap();
    assert_eq!(state.shortcuts.len(), 1);
    assert_eq!(state.shortcuts[0].name, "Demo Shortcut");

    let (uninstall_ok, uninstall_stdout, uninstall_stderr) =
        run_in_home_without_test_mode(&["scoop", "uninstall", "demo"], &temp_home, &[]);
    assert!(
        uninstall_ok,
        "stdout: {uninstall_stdout}\nstderr: {uninstall_stderr}"
    );
    assert_path_missing(&shortcut);
}

#[test]
fn spoon_scoop_package_cli_honors_extract_dir_and_extract_to() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    let bucket_source = temp_home.join("extract-map-bucket");
    std::fs::create_dir_all(bucket_source.join("bucket")).unwrap();
    let (archive, hash) = create_zip_archive_with_entries(
        &temp_home,
        "extract-map.zip",
        &[("payload/tools/demo.exe", b"mapped-demo")],
    );
    std::fs::write(
        bucket_source.join("bucket").join("demo.json"),
        format!(
            "{{\n  \"version\": \"1.0.0\",\n  \"url\": \"{}\",\n  \"hash\": \"{}\",\n  \"extract_dir\": \"payload/tools\",\n  \"extract_to\": \"portable\",\n  \"bin\": \"portable/demo.exe\"\n}}",
            file_url(&archive),
            hash
        ),
    )
    .unwrap();

    register_local_bucket(&temp_home, "extras", &bucket_source);

    let (install_ok, install_stdout, install_stderr) =
        run_in_home_without_test_mode(&["scoop", "install", "demo"], &temp_home, &[]);
    assert!(
        install_ok,
        "stdout: {install_stdout}\nstderr: {install_stderr}"
    );

    let installed = tool_root
        .join("scoop")
        .join("apps")
        .join("demo")
        .join("current")
        .join("portable")
        .join("demo.exe");
    let shim = config::shims_root_from(&tool_root).join("demo.cmd");
    assert_path_exists(&installed);
    assert_path_exists(&shim);
}

#[test]
#[ignore = "best-effort real remote Scoop bucket smoke; requires working network/proxy/git IO"]
fn spoon_scoop_real_remote_bucket_cli_handles_add_list_remove() {
    let proxy = spoon::config::load_global_config().proxy;
    let env = create_configured_home_with_proxy(&proxy);
    let temp_home = env.home;
    let _tool_root = env.root;

    let (add_ok, add_stdout, add_stderr) = run_in_home_without_test_mode(
        &["scoop", "bucket", "add", "extras", "--branch", "master"],
        &temp_home,
        &[],
    );
    assert!(add_ok, "stdout: {add_stdout}\nstderr: {add_stderr}");

    let (list_ok, list_stdout, list_stderr) =
        run_in_home_without_test_mode(&["scoop", "bucket", "list"], &temp_home, &[]);
    assert!(list_ok, "stdout: {list_stdout}\nstderr: {list_stderr}");
    assert!(
        list_stdout.contains("extras | master | https://github.com/ScoopInstaller/Extras"),
        "stdout: {list_stdout}"
    );

    let (remove_ok, remove_stdout, remove_stderr) =
        run_in_home_without_test_mode(&["scoop", "bucket", "remove", "extras"], &temp_home, &[]);
    assert!(
        remove_ok,
        "stdout: {remove_stdout}\nstderr: {remove_stderr}"
    );
    assert!(
        remove_stdout.contains("Removed bucket 'extras'."),
        "stdout: {remove_stdout}"
    );
}

#[test]
#[ignore = "best-effort real remote Scoop package smoke; requires working network/proxy/git IO"]
fn spoon_scoop_real_remote_package_cli_handles_install_uninstall() {
    let proxy = spoon::config::load_global_config().proxy;
    let env = create_configured_home_with_proxy(&proxy);
    let temp_home = env.home;
    let tool_root = env.root;

    let (add_ok, add_stdout, add_stderr) = run_in_home_without_test_mode(
        &["scoop", "bucket", "add", "main", "--branch", "master"],
        &temp_home,
        &[],
    );
    assert!(add_ok, "stdout: {add_stdout}\nstderr: {add_stderr}");

    let (install_ok, install_stdout, install_stderr) =
        run_in_home_without_test_mode(&["scoop", "install", "jq"], &temp_home, &[]);
    assert!(
        install_ok,
        "stdout: {install_stdout}\nstderr: {install_stderr}"
    );
    assert!(
        install_stdout.contains("Installed Scoop package 'jq' into"),
        "stdout: {install_stdout}"
    );

    let jq_shim = config::shims_root_from(&tool_root).join("jq.cmd");
    assert!(jq_shim.exists(), "missing jq shim: {}", jq_shim.display());
    let layout = RuntimeLayout::from_root(&tool_root);
    assert!(
        spoon::runtime::test_block_on(read_installed_state(&layout, "jq")).is_some(),
        "missing jq installed state in control plane"
    );

    let (uninstall_ok, uninstall_stdout, uninstall_stderr) =
        run_in_home_without_test_mode(&["scoop", "uninstall", "jq"], &temp_home, &[]);
    assert!(
        uninstall_ok,
        "stdout: {uninstall_stdout}\nstderr: {uninstall_stderr}"
    );
    assert!(
        uninstall_stdout.contains("Removed Scoop package 'jq'."),
        "stdout: {uninstall_stdout}"
    );
}
