use crate::scoop::add_bucket_to_registry;
use crate::scoop::{
    BucketSpec, load_buckets_from_registry, remove_bucket_from_registry, resolve_manifest,
    sync_main_bucket_registry, upsert_bucket_to_registry,
};
use crate::tests::{block_on, temp_dir};

#[test]
fn registry_tracks_main_bucket() {
    let root = temp_dir("registry-main");
    std::fs::create_dir_all(root.join("scoop").join("buckets").join("main")).unwrap();
    block_on(sync_main_bucket_registry(&root)).unwrap();
    let buckets = block_on(load_buckets_from_registry(&root));
    assert!(buckets.iter().any(|bucket| bucket.name == "main"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn sqlite_bucket_registry_roundtrips_and_preserves_repo_fs() {
    let root = temp_dir("sqlite-bucket-registry");
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
            .join("jq.json"),
        "{}",
    )
    .unwrap();

    block_on(upsert_bucket_to_registry(
        &root,
        &BucketSpec {
            name: "extras".to_string(),
            source: Some("https://example.com/extras.git".to_string()),
            branch: Some("main".to_string()),
        },
    ))
    .unwrap();

    let buckets = block_on(load_buckets_from_registry(&root));
    assert_eq!(buckets.len(), 1);
    assert_eq!(buckets[0].name, "extras");
    assert_eq!(buckets[0].branch, "main");
    assert!(
        root.join("scoop")
            .join("buckets")
            .join("extras")
            .join("bucket")
            .join("jq.json")
            .exists()
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn resolve_manifest_prefers_registered_bucket_order() {
    let root = temp_dir("resolve-prefers-order");
    std::fs::create_dir_all(
        root.join("scoop")
            .join("buckets")
            .join("extras")
            .join("bucket"),
    )
    .unwrap();
    std::fs::create_dir_all(
        root.join("scoop")
            .join("buckets")
            .join("main")
            .join("bucket"),
    )
    .unwrap();
    std::fs::write(
        root.join("scoop")
            .join("buckets")
            .join("extras")
            .join("bucket")
            .join("jq.json"),
        "{}",
    )
    .unwrap();
    std::fs::write(
        root.join("scoop")
            .join("buckets")
            .join("main")
            .join("bucket")
            .join("jq.json"),
        "{}",
    )
    .unwrap();
    block_on(upsert_bucket_to_registry(
        &root,
        &BucketSpec {
            name: "extras".to_string(),
            source: Some("https://example.com/extras.git".to_string()),
            branch: Some("main".to_string()),
        },
    ))
    .unwrap();
    block_on(sync_main_bucket_registry(&root)).unwrap();
    let resolved = block_on(resolve_manifest(&root, "jq")).unwrap();
    assert_eq!(resolved.bucket.name, "extras");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn remove_bucket_from_registry_refuses_main() {
    let root = temp_dir("remove-main");
    let error = block_on(remove_bucket_from_registry(&root, "main")).unwrap_err();
    assert!(error.to_string().contains("cannot be removed"));
}

#[test]
fn add_bucket_to_registry_supports_non_git_local_sources() {
    let root = temp_dir("add-local");
    let source = root.join("local-source");
    std::fs::create_dir_all(source.join("bucket")).unwrap();
    std::fs::write(source.join("bucket").join("demo.json"), "{}").unwrap();

    block_on(add_bucket_to_registry(
        &root,
        &BucketSpec {
            name: "local".to_string(),
            source: Some(source.display().to_string()),
            branch: Some("main".to_string()),
        },
        "",
    ))
    .unwrap();

    assert!(
        root.join("scoop")
            .join("buckets")
            .join("local")
            .join("bucket")
            .join("demo.json")
            .exists()
    );
    let _ = std::fs::remove_dir_all(root);
}
