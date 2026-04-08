#![allow(dead_code)]

use std::path::{Path, PathBuf};

use crate::common::cli::create_test_home;

pub struct TestInstallEnv {
    pub home: PathBuf,
    pub root: PathBuf,
}

pub fn create_configured_home() -> TestInstallEnv {
    create_configured_home_with_proxy("")
}

pub fn create_configured_home_with_proxy(proxy: &str) -> TestInstallEnv {
    let home = create_test_home();
    let root = home.join("tool-root");
    write_test_config(&home, &root, proxy);
    TestInstallEnv { home, root }
}

pub fn write_test_config(temp_home: &Path, tool_root: &Path, proxy: &str) {
    std::fs::create_dir_all(temp_home.join(".spoon")).unwrap();
    let config_text = format!(
        "editor = \"\"\nproxy = \"{}\"\nroot = '{}'\n",
        proxy.replace('\\', "\\\\"),
        tool_root.display().to_string().replace('\\', "\\\\")
    );
    std::fs::write(temp_home.join(".spoon").join("config.toml"), config_text).unwrap();
}
