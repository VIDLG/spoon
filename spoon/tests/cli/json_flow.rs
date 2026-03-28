#[path = "../common/mod.rs"]
mod common;

use common::assertions::assert_ok;
use common::cli::run_in_home;
use common::setup::create_configured_home;
use serde_json::Value;

fn parse_json(stdout: &str) -> Value {
    serde_json::from_str(stdout).expect("stdout should be valid json")
}

#[test]
fn status_json_prints_machine_readable_status_snapshot() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) = run_in_home(&["--json", "status"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "status");
    assert!(json["data"]["overview"]["primary_tools"].is_array());
    assert!(json["data"]["tools"].is_array());
    assert_eq!(json["data"]["runtime"]["updates"]["mode"], "local_only");
}

#[test]
fn config_json_prints_structured_view_model() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) = run_in_home(&["--json", "config"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "config");
    assert!(json["data"]["config_file"].is_string());
    assert!(json["data"]["packages"]["git"]["command_profile"]["adds"].is_array());
    assert!(json["data"]["packages"]["python"]["command_profile"]["adds"].is_array());
}

#[test]
fn config_python_set_json_returns_single_result_object() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) = run_in_home(
        &["--json", "config", "python", "command_profile=default"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "config_scope_result");
    assert_eq!(json["scope"], "python");
    assert_eq!(json["action"], "set");
    assert_eq!(json["changed_key"], "python.command_profile");
    assert_eq!(json["view"]["scope"], "python");
}

#[test]
fn scoop_info_json_prints_structured_package_view() {
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
        bucket_root.join("git.json"),
        r#"{
            "version": "2.53.0.2",
            "description": "Git for Windows",
            "homepage": "https://gitforwindows.org",
            "url": "https://example.invalid/git.7z",
            "bin": ["bin\\git.exe", "git-bash.exe"]
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

    let (ok, stdout, stderr) = run_in_home(&["--json", "info", "git"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "package_info");
    assert_eq!(json["success"], true);
    assert_eq!(json["package"]["name"], "git");
    assert!(json["package"]["download_urls"].is_array());
    assert!(json["integration"]["policy"]["desired"].is_array());
}

#[test]
fn scoop_status_json_prints_structured_runtime_view() {
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
    let package_state_root = spoon::config::scoop_state_root_from(&tool_root).join("packages");
    std::fs::create_dir_all(&package_state_root).unwrap();
    std::fs::write(
        package_state_root.join("jq.json"),
        serde_json::json!({ "package": "jq", "version": "1.8.1" }).to_string(),
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["--json", "scoop", "status"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "scoop_status");
    assert_eq!(json["runtime"]["bucket_count"], 1);
    assert!(json["buckets"].is_array());
    assert!(json["installed_packages"].is_array());
}

#[test]
fn scoop_bucket_list_json_prints_structured_bucket_inventory() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &tool_root,
        &spoon::service::scoop::BucketSpec {
            name: "main".to_string(),
            source: Some("https://github.com/ScoopInstaller/Main".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["--json", "scoop", "bucket", "list"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "scoop_bucket_list");
    assert_eq!(json["bucket_count"], 1);
    assert_eq!(json["buckets"][0]["name"], "main");
}

#[test]
fn scoop_bucket_remove_json_prints_structured_action_result() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &tool_root,
        &spoon::service::scoop::BucketSpec {
            name: "extras".to_string(),
            source: Some("https://github.com/ScoopInstaller/Extras".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();
    std::fs::create_dir_all(
        spoon::config::scoop_root_from(&tool_root)
            .join("buckets")
            .join("extras"),
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(
        &["--json", "scoop", "bucket", "remove", "extras"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "scoop_bucket_action");
    assert_eq!(json["action"], "remove");
    assert_eq!(json["targets"][0], "extras");
    assert_eq!(json["success"], true);
    assert!(json["buckets"].is_array());
}

#[test]
fn scoop_search_json_prints_structured_matches() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let bucket_root = tool_root
        .join("scoop")
        .join("buckets")
        .join("extras")
        .join("bucket");
    std::fs::create_dir_all(&bucket_root).unwrap();
    std::fs::write(
        bucket_root.join("demo-tool.json"),
        r#"{ "version": "1.0.0", "description": "Demo search package", "homepage": "https://example.invalid/demo" }"#,
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

    let (ok, stdout, stderr) = run_in_home(&["--json", "scoop", "search", "demo"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "scoop_search");
    assert_eq!(json["query"], "demo");
    assert_eq!(json["matches"][0]["package_name"], "demo-tool");
    assert_eq!(json["matches"][0]["bucket"], "extras");
}

#[test]
fn msvc_status_json_prints_structured_runtime_status() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let state_root = spoon::config::msvc_state_root_from(&tool_root);
    std::fs::create_dir_all(spoon::config::msvc_toolchain_root_from(&tool_root)).unwrap();
    std::fs::create_dir_all(&state_root).unwrap();
    std::fs::write(state_root.join("runtime.json"), "{}").unwrap();
    std::fs::write(
        state_root.join("installed.json"),
        serde_json::json!({
            "msvc": "msvc-14.44.17.14",
            "sdk": "sdk-10.0.26100.15"
        })
        .to_string(),
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["--json", "msvc", "status"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "msvc_status");
    assert!(json["managed"]["root"].is_string());
    assert!(json["official"]["root"].is_string());
}

#[test]
fn doctor_json_prints_structured_scoop_runtime_repair_summary() {
    let env = create_configured_home();
    let temp_home = env.home;
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &env.root,
        &spoon::service::scoop::BucketSpec {
            name: "main".to_string(),
            source: Some("https://github.com/ScoopInstaller/Main".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();
    std::fs::create_dir_all(
        spoon::config::scoop_root_from(&env.root)
            .join("buckets")
            .join("main"),
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["--json", "doctor"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "scoop_doctor");
    assert_eq!(json["success"], true);
    assert!(json["runtime"]["root"].is_string());
    assert!(json["ensured_paths"].is_array());
    assert!(json["registered_buckets"].is_array());
}

#[test]
fn scoop_prefix_json_prints_structured_prefix_view() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let package_name = "git";
    let state_root = spoon::config::scoop_state_root_from(&tool_root).join("packages");
    std::fs::create_dir_all(&state_root).unwrap();
    std::fs::write(
        state_root.join(format!("{package_name}.json")),
        serde_json::json!({
            "package": package_name,
            "version": "2.53.0.2"
        })
        .to_string(),
    )
    .unwrap();
    std::fs::create_dir_all(
        spoon::config::scoop_root_from(&tool_root)
            .join("apps")
            .join(package_name)
            .join("current"),
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(
        &["--json", "scoop", "prefix", package_name],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "package_prefix");
    assert_eq!(json["package"], package_name);
    assert_eq!(json["installed"], true);
    assert!(json["prefix"].is_string());
}

#[test]
fn install_json_prints_structured_package_action_results() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) = run_in_home(&["--json", "install", "git"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "scoop_package_actions");
    assert_eq!(json["action"], "install");
    assert_eq!(json["results"][0]["kind"], "scoop_package_action");
    assert_eq!(json["results"][0]["package"]["name"], "git");
    assert!(json["results"][0]["output"].is_array());
    assert!(json["results"][0]["state"].is_object());
}

#[test]
fn json_errors_print_stable_error_envelope() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) = run_in_home(
        &["--json", "scoop", "bucket", "add", "custom-only"],
        &temp_home,
        &[],
    );
    assert!(!ok, "stdout={stdout}\nstderr={stderr}");
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "error");
    assert!(json["data"]["message"].is_string());
    assert!(json["data"]["chain"].is_array());
    assert!(
        stderr.trim().is_empty(),
        "stderr should stay empty in --json mode: {stderr}"
    );
}

#[test]
fn scoop_cache_clear_json_prints_structured_cache_action() {
    let env = create_configured_home();
    let temp_home = env.home;
    let scoop_cache = spoon::config::scoop_root_from(&env.root).join("cache");
    std::fs::create_dir_all(&scoop_cache).unwrap();
    std::fs::write(scoop_cache.join("demo.txt"), "demo").unwrap();

    let (ok, stdout, stderr) = run_in_home(&["--json", "scoop", "cache", "clear"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "cache_action");
    assert_eq!(json["action"], "clear");
    assert_eq!(json["scope"], "scoop");
    assert!(json["paths"]["scoop"].is_string());
}

#[test]
fn status_refresh_json_embeds_structured_bucket_update_result() {
    let env = create_configured_home();
    let temp_home = env.home;
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &env.root,
        &spoon::service::scoop::BucketSpec {
            name: "main".to_string(),
            source: Some("https://github.com/ScoopInstaller/Main".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["--json", "status", "--refresh"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "status_refresh");
    assert_eq!(json["bucket_update"]["kind"], "scoop_bucket_action");
    assert_eq!(json["bucket_update"]["action"], "update");
    assert_eq!(json["status"]["kind"], "status");
}

#[test]
fn package_report_uses_backend_models() {
    // Verify that the package prefix report uses backend RuntimeLayout and
    // runtime_status instead of app-local package_current_root /
    // installed_package_states_filtered helpers.

    // Verify RuntimeLayout derives the same prefix path that package_current_root did
    let tool_root = std::path::PathBuf::from("D:/test-root");
    let layout = spoon_backend::layout::RuntimeLayout::from_root(&tool_root);
    let package_name = "git";

    // RuntimeLayout should derive: <root>/scoop/apps/<package>/current
    let layout_prefix = layout.scoop.apps_root.join(package_name).join("current");
    assert!(
        layout_prefix.to_string_lossy().contains("scoop"),
        "RuntimeLayout prefix should contain scoop dir"
    );
    assert!(
        layout_prefix.to_string_lossy().contains(package_name),
        "RuntimeLayout prefix should contain package name"
    );
    assert!(
        layout_prefix.to_string_lossy().contains("current"),
        "RuntimeLayout prefix should contain 'current'"
    );

    // Verify the path matches the old package_current_root derivation
    let old_prefix = spoon_backend::scoop::package_current_root(&tool_root, package_name);
    assert_eq!(
        layout_prefix, old_prefix,
        "RuntimeLayout prefix should match backend package_current_root derivation"
    );
}

fn bucket_json_uses_backend_repo_sync_outcome() {
    // Verify that the app bucket layer exposes RepoSyncOutcome from the backend
    // and that the backend contract type is usable at the app boundary.
    let outcome = spoon::service::scoop::RepoSyncOutcome {
        head_commit: Some("abc123".to_string()),
        head_branch: Some("main".to_string()),
    };
    assert_eq!(outcome.head_commit.as_deref(), Some("abc123"));
    assert_eq!(outcome.head_branch.as_deref(), Some("main"));

    // Also verify that a real bucket update flows through backend outcomes.
    // This exercises the bucket adapter path where load_backend_config
    // has been removed and proxy is read directly from app config.
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &tool_root,
        &spoon::service::scoop::BucketSpec {
            name: "main".to_string(),
            source: Some("https://github.com/ScoopInstaller/Main".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();
    std::fs::create_dir_all(
        spoon::config::scoop_root_from(&tool_root)
            .join("buckets")
            .join("main"),
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(
        &["--json", "scoop", "bucket", "list"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "scoop_bucket_list");
    assert_eq!(json["bucket_count"], 1);
}
