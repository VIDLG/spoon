use std::fs;

use crate::scoop::cache::package_cache_size;
use crate::layout::RuntimeLayout;
use crate::tests::{block_on, temp_dir};

#[test]
fn package_cache_size_returns_zero_when_cache_root_is_missing() {
    let tool_root = temp_dir("scoop-cache-size-missing-root");
    fs::create_dir_all(&tool_root).expect("tool root should be created");

    let size = block_on(package_cache_size(&tool_root, "git"))
        .expect("missing cache root should be treated as empty");
    assert_eq!(size, 0);

    fs::remove_dir_all(&tool_root).expect("tool root should be removed");
}

#[test]
fn package_cache_size_keeps_zero_for_empty_matching_entry() {
    let tool_root = temp_dir("scoop-cache-size-empty-entry");
    let cache_entry = RuntimeLayout::from_root(&tool_root)
        .scoop
        .cache_root
        .join("git#1.0");
    fs::create_dir_all(&cache_entry).expect("cache entry should be created");

    let size = block_on(package_cache_size(&tool_root, "git"))
        .expect("empty matching cache entry should be readable");
    assert_eq!(size, 0);

    fs::remove_dir_all(&tool_root).expect("tool root should be removed");
}
