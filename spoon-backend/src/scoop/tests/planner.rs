use crate::scoop::{
    BucketSpec, ScoopPackageAction, infer_tool_root, plan_package_action, upsert_bucket_to_registry,
};
use crate::tests::{block_on, temp_dir};

#[test]
fn install_plan_adds_no_update_scoop_flag() {
    let plan = plan_package_action("install", "uv", "uv", None);
    assert_eq!(plan.action, ScoopPackageAction::Install);
    assert_eq!(plan.args, vec!["install", "uv", "--no-update-scoop"]);
}

#[test]
fn uninstall_plan_uses_plain_uninstall_args() {
    let plan = plan_package_action("uninstall", "uv", "uv", None);
    assert_eq!(plan.action, ScoopPackageAction::Uninstall);
    assert_eq!(plan.args, vec!["uninstall", "uv"]);
}

#[test]
fn install_plan_resolves_registered_bucket_manifest() {
    let root = temp_dir("plan-resolve-manifest");
    std::fs::create_dir_all(
        root.join("scoop")
            .join("buckets")
            .join("extras")
            .join("bucket"),
    )
    .unwrap();
    std::fs::write(
        root.join("scoop")
            .join("buckets")
            .join("extras")
            .join("bucket")
            .join("uv.json"),
        "{}",
    )
    .unwrap();
    block_on(upsert_bucket_to_registry(
        &root,
        &BucketSpec {
            name: "extras".to_string(),
            source: Some("https://example.com/extras".to_string()),
            branch: Some("main".to_string()),
        },
    ))
    .unwrap();
    let plan = plan_package_action("install", "uv", "uv", Some(&root));
    let line = plan.resolution_line().expect("resolution line");
    assert!(line.contains("bucket 'extras'"), "line: {line}");
    assert!(line.contains("https://example.com/extras"), "line: {line}");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn infer_tool_root_uses_explicit_root_first() {
    let root = std::env::temp_dir().join("spoon-planner-root");
    let inferred = infer_tool_root(Some(&root), None).expect("explicit root");
    assert_eq!(inferred, root);
}
