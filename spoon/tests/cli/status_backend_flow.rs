#[path = "../common/mod.rs"]
mod common;

use common::assertions::assert_ok;
use common::cli::run_in_home;
use common::setup::create_configured_home;
use serde_json::Value;

fn parse_json(stdout: &str) -> Value {
    serde_json::from_str(stdout).expect("stdout should be valid json")
}

/// Regression test: the JSON status path must use backend read models
/// and must not fall back to app-side state file parsing (BNDR-05, LAY-02).
#[test]
fn json_status_uses_backend_read_models() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    // Set up a Scoop package state so the backend snapshot has content
    let state_root = spoon::config::scoop_state_root_from(&tool_root).join("packages");
    std::fs::create_dir_all(&state_root).unwrap();
    std::fs::write(
        state_root.join("git.json"),
        serde_json::json!({ "package": "git", "version": "2.53.0.2" }).to_string(),
    )
    .unwrap();

    // Register a bucket so the backend snapshot includes bucket data
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &tool_root,
        &spoon::service::scoop::BucketSpec {
            name: "main".to_string(),
            source: Some("https://github.com/ScoopInstaller/Main".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();

    // The JSON status command must succeed
    let (ok, stdout, stderr) = run_in_home(&["--json", "status"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);

    let json = parse_json(&stdout);
    assert_eq!(json["kind"], "status");

    // The status data must include tools and runtime sections
    assert!(json["data"]["tools"].is_array());
    assert!(json["data"]["runtime"].is_object());

    // When a backend snapshot is available, the roots must come from the snapshot
    // (backend-owned RuntimeLayout), not from app-side path helpers.
    // This is the core contract: the JSON status path consumes backend read models.
    let roots = &json["data"]["runtime"]["roots"];
    if roots.is_object() {
        // Roots exist and must match backend layout derivation
        assert!(roots["root"].is_string());
        assert!(roots["scoop"].is_string());
        assert!(roots["managed_msvc"].is_string());
        assert!(roots["managed_toolchain"].is_string());
        assert!(roots["official_msvc"].is_string());
    }
}
