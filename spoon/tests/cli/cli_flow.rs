#[path = "../common/mod.rs"]
mod common;

use common::assertions::{assert_contains, assert_ok, assert_path_exists, assert_path_missing};
use common::cli::{create_test_home, run, run_in_home, run_in_home_without_test_mode};
use common::setup::create_configured_home;
use spoon::config;
use spoon_core::RuntimeLayout;

#[test]
fn status_command_prints_core_sections() {
    let (ok, stdout, stderr) = run(&["status"]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Primary tools:");
    assert_contains(&stdout, "Runtime model:");
    assert_contains(&stdout, "Toolchains:");
}

#[test]
fn doctor_prepares_default_bucket_and_shims() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    let main_bucket = temp_home.join("main-bucket");
    std::fs::create_dir_all(main_bucket.join("bucket")).unwrap();

    let source = main_bucket.display().to_string();
    let (ok, stdout, stderr) = run_in_home_without_test_mode(
        &["doctor"],
        &temp_home,
        &[("SPOON_TEST_SCOOP_BUCKET_MAIN_SOURCE", &source)],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Ensured Scoop directory:");
    assert_contains(&stdout, "Registered bucket 'main'.");
    assert_contains(&stdout, "Ensured Spoon shims are available on PATH:");
    assert!(
        tool_root
            .join("scoop")
            .join("buckets")
            .join("main")
            .exists(),
        "main bucket should be prepared"
    );
    assert!(
        RuntimeLayout::from_root(&tool_root).shims.exists(),
        "shims root should be prepared"
    );
}

#[test]
fn cache_prune_requires_configured_root() {
    let temp_home = create_test_home();
    std::fs::create_dir_all(temp_home.join(".spoon")).unwrap();
    std::fs::write(
        temp_home.join(".spoon").join("config.toml"),
        "editor = \"\"\nproxy = \"\"\nroot = \"\"\n",
    )
    .unwrap();
    let (ok, stdout, stderr) = run_in_home(&["scoop", "cache", "prune"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(
        &stdout,
        "Scoop package management requires a configured root.",
    );
}

#[test]
fn cache_commands_clean_scoped_domain_cache() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;

    let scoop_cache = RuntimeLayout::from_root(&tool_root).scoop.cache_root;
    let msvc_cache = RuntimeLayout::from_root(&tool_root).msvc.managed.cache_root;
    let msvc_validate = msvc_cache.join("validate");
    let msvc_metadata = msvc_cache.join("metadata");
    let msvc_archives = msvc_cache.join("archives");
    let msvc_state = RuntimeLayout::from_root(&tool_root).msvc.managed.state_root;
    let msvc_toolchain = RuntimeLayout::from_root(&tool_root).msvc.managed.toolchain_root;

    std::fs::create_dir_all(&scoop_cache).unwrap();
    std::fs::create_dir_all(&msvc_validate).unwrap();
    std::fs::create_dir_all(&msvc_metadata).unwrap();
    std::fs::create_dir_all(&msvc_archives).unwrap();
    std::fs::create_dir_all(&msvc_state).unwrap();
    std::fs::create_dir_all(&msvc_toolchain).unwrap();

    std::fs::create_dir_all(&msvc_validate).unwrap();
    std::fs::create_dir_all(&msvc_metadata).unwrap();
    std::fs::create_dir_all(&msvc_archives).unwrap();
    std::fs::write(scoop_cache.join("demo.zip"), b"cache").unwrap();
    std::fs::write(msvc_validate.join("hello.exe"), b"cache").unwrap();
    std::fs::write(msvc_metadata.join("demo.txt"), b"cache").unwrap();
    std::fs::write(msvc_archives.join("payload.bin"), b"cache").unwrap();
    std::fs::write(msvc_state.join("runtime.json"), b"{}").unwrap();
    std::fs::write(msvc_toolchain.join("cl.exe"), b"").unwrap();

    let (ok, stdout, stderr) = run_in_home(&["msvc", "cache", "prune"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert!(
        stdout.contains("Pruned MSVC cache directory")
            || stdout.contains("No pruneable MSVC cache directories were present."),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert_path_missing(&msvc_validate);
    assert_path_missing(&msvc_metadata);
    assert_path_exists(&msvc_archives);
    assert_path_exists(&msvc_state);

    let (ok, stdout, stderr) = run_in_home(&["scoop", "cache", "clear"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Cleared Scoop cache under");
    assert_path_exists(&scoop_cache);
    assert_path_missing(&scoop_cache.join("demo.zip"));

    let (ok, stdout, stderr) = run_in_home(&["msvc", "cache", "clear"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Cleared MSVC cache under");
    assert_path_exists(&msvc_cache);
    assert_path_missing(&msvc_archives.join("payload.bin"));
    assert_path_exists(&msvc_state);
    assert_path_exists(&msvc_toolchain);
}

#[test]
fn verbose_cli_surfaces_internal_logs() {
    let (ok, stdout, stderr) = run(&["--verbose", "status"]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "session.start");
    assert_contains(&stdout, "app.start");
    assert_contains(&stdout, "Runtime model:");
}
