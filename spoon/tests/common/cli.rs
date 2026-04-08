#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CLI_HOME_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn detect_repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|path| path.join("skills").is_dir())
        .expect("repo root")
        .to_path_buf()
}

pub fn create_test_home() -> PathBuf {
    let temp_home = std::env::temp_dir().join(format!(
        "spoon-cli-test-home-{}",
        CLI_HOME_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&temp_home);
    std::fs::create_dir_all(&temp_home).expect("create cli test home");
    temp_home
}

pub fn run_in_home(
    args: &[&str],
    temp_home: &Path,
    extra_env: &[(&str, &str)],
) -> (bool, String, String) {
    run_in_home_with_mode(args, temp_home, extra_env, true)
}

pub fn run_in_home_without_test_mode(
    args: &[&str],
    temp_home: &Path,
    extra_env: &[(&str, &str)],
) -> (bool, String, String) {
    run_in_home_with_mode(args, temp_home, extra_env, false)
}

fn run_in_home_with_mode(
    args: &[&str],
    temp_home: &Path,
    extra_env: &[(&str, &str)],
    test_mode: bool,
) -> (bool, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_spoon"));
    command
        .args(args)
        .current_dir(detect_repo_root())
        .env("SPOON_TEST_HOME", temp_home)
        .env("USERPROFILE", temp_home)
        .env("HOME", temp_home);
    if test_mode {
        command.env("SPOON_TEST_MODE", "1");
    } else {
        command.env_remove("SPOON_TEST_MODE");
    }
    for (key, value) in extra_env {
        command.env(key, value);
    }

    let output = command.output().expect("run spoon");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

pub fn run(args: &[&str]) -> (bool, String, String) {
    let temp_home = create_test_home();
    run_in_home(args, &temp_home, &[])
}
