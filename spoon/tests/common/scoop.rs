#![allow(dead_code)]

use crate::common::assertions::assert_ok;
use crate::common::cli::run_in_home;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn file_url(path: &Path) -> String {
    format!("file:///{}", path.display().to_string().replace('\\', "/"))
}

pub fn create_zip_archive_with_entries(
    base: &Path,
    archive_name: &str,
    entries: &[(&str, &[u8])],
) -> (PathBuf, String) {
    let archive = base.join(archive_name);
    let file = std::fs::File::create(&archive).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    for (entry_name, contents) in entries {
        zip.start_file(entry_name, options).unwrap();
        zip.write_all(contents).unwrap();
    }
    zip.finish().unwrap();
    let bytes = std::fs::read(&archive).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    (archive, format!("{:x}", hasher.finalize()))
}

pub fn create_zip_archive(base: &Path, file_name: &str, contents: &[u8]) -> (PathBuf, String) {
    create_zip_archive_with_entries(base, &format!("{file_name}.zip"), &[(file_name, contents)])
}

pub fn create_demo_archive_with_config(
    base: &Path,
    exe_contents: &[u8],
    config_contents: &[u8],
) -> (PathBuf, String) {
    create_zip_archive_with_entries(
        base,
        "demo-with-config.zip",
        &[
            ("bin/demo.exe", exe_contents),
            ("config/settings.json", config_contents),
        ],
    )
}

pub fn write_manifest_text(bucket_source: &Path, package_name: &str, manifest_text: &str) {
    std::fs::create_dir_all(bucket_source.join("bucket")).unwrap();
    std::fs::write(
        bucket_source
            .join("bucket")
            .join(format!("{package_name}.json")),
        manifest_text,
    )
    .unwrap();
}

pub fn write_demo_manifest(bucket_source: &Path, version: &str, archive: &Path, hash: &str) {
    write_manifest_text(
        bucket_source,
        "demo",
        &format!(
            "{{\n  \"version\": \"{version}\",\n  \"url\": \"{}\",\n  \"hash\": \"{hash}\",\n  \"bin\": \"demo.exe\"\n}}",
            file_url(archive)
        ),
    );
}

pub fn register_local_bucket(temp_home: &Path, bucket_name: &str, bucket_source: &Path) {
    let source = bucket_source.display().to_string();
    let (ok, stdout, stderr) = run_in_home(
        &[
            "scoop",
            "bucket",
            "add",
            bucket_name,
            &source,
            "--branch",
            "main",
        ],
        temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
}

pub fn update_bucket(temp_home: &Path, bucket_name: &str) {
    let (ok, stdout, stderr) =
        run_in_home(&["scoop", "bucket", "update", bucket_name], temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
}
