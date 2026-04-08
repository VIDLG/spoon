#![allow(dead_code)]

use std::path::Path;

pub fn assert_ok(ok: bool, stdout: &str, stderr: &str) {
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
}

pub fn assert_not_ok(ok: bool, stdout: &str, stderr: &str) {
    assert!(!ok, "stdout: {stdout}\nstderr: {stderr}");
}

pub fn assert_contains(text: &str, needle: &str) {
    assert!(
        text.contains(needle),
        "expected to find `{needle}` in:\n{text}"
    );
}

pub fn assert_not_contains(text: &str, needle: &str) {
    assert!(
        !text.contains(needle),
        "expected not to find `{needle}` in:\n{text}"
    );
}

pub fn assert_contains_any(text: &str, needles: &[&str]) {
    assert!(
        needles.iter().any(|needle| text.contains(needle)),
        "expected to find one of {:?} in:\n{}",
        needles,
        text
    );
}

pub fn assert_path_exists(path: &Path) {
    assert!(path.exists(), "missing path: {}", path.display());
}

pub fn assert_path_missing(path: &Path) {
    assert!(!path.exists(), "path should be absent: {}", path.display());
}
