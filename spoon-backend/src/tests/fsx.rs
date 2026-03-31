use std::fs;

use crate::{BackendError, directory_size};

use super::{block_on, temp_dir};

#[test]
fn directory_size_returns_zero_for_empty_directory() {
    let root = temp_dir("fsx-empty-directory");
    fs::create_dir_all(&root).expect("temp directory should be created");

    let size = block_on(directory_size(&root)).expect("empty directory should be readable");
    assert_eq!(size, 0);

    fs::remove_dir_all(&root).expect("temp directory should be removed");
}

#[test]
fn directory_size_returns_error_for_missing_path() {
    let path = temp_dir("fsx-missing-directory");

    let err = block_on(directory_size(&path)).expect_err("missing path should not look empty");
    match err {
        BackendError::Fs {
            action,
            path: err_path,
            ..
        } => {
            assert_eq!(action, "metadata");
            assert_eq!(err_path, path);
        }
        other => panic!("expected filesystem metadata error, got {other:?}"),
    }
}

#[test]
fn directory_size_sums_nested_files() {
    let root = temp_dir("fsx-nested-directory");
    let nested = root.join("nested");
    fs::create_dir_all(&nested).expect("nested temp directory should be created");
    fs::write(root.join("a.bin"), vec![0_u8; 3]).expect("root file should be written");
    fs::write(nested.join("b.bin"), vec![0_u8; 5]).expect("nested file should be written");

    let size = block_on(directory_size(&root)).expect("directory should be readable");
    assert_eq!(size, 8);

    fs::remove_dir_all(&root).expect("temp directory should be removed");
}
