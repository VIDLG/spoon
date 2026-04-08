#[path = "../common/mod.rs"]
mod common;

use common::assertions::{assert_contains, assert_ok, assert_path_exists};
use common::cli::{create_test_home, run, run_in_home};
use common::setup::{create_configured_home, create_configured_home_with_proxy, write_test_config};
use spoon_core::RuntimeLayout;
use spoon_scoop::{InstalledPackageCommandSurface, InstalledPackageIdentity, InstalledPackageState, InstalledPackageUninstall, AppliedIntegration, write_installed_state};

fn empty_python_state() -> InstalledPackageState {
    InstalledPackageState {
        identity: InstalledPackageIdentity {
            package: "python".to_string(),
            version: "3.14.3".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
        },
        command_surface: InstalledPackageCommandSurface {
            bins: vec![
                "python".to_string(),
                "python3".to_string(),
                "pip".to_string(),
            ],
            shortcuts: vec![],
            env_add_path: vec!["Scripts".to_string(), ".".to_string()],
            env_set: std::collections::BTreeMap::new(),
            persist: vec![],
        },
        integrations: vec![],
        uninstall: InstalledPackageUninstall {
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec![],
        },
    }
}

fn empty_git_state() -> InstalledPackageState {
    InstalledPackageState {
        identity: InstalledPackageIdentity {
            package: "git".to_string(),
            version: "2.53.0.2".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
        },
        command_surface: InstalledPackageCommandSurface {
            bins: vec![
                "git".to_string(),
                "git-bash".to_string(),
                "bash".to_string(),
            ],
            shortcuts: vec![],
            env_add_path: vec!["cmd".to_string()],
            env_set: std::collections::BTreeMap::from([(
                "GIT_INSTALL_ROOT".to_string(),
                "$dir".to_string(),
            )]),
            persist: vec![],
        },
        integrations: vec![],
        uninstall: InstalledPackageUninstall {
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec![],
        },
    }
}

fn integration(key: &str, value: &str) -> AppliedIntegration {
    AppliedIntegration {
        key: key.to_string(),
        value: value.to_string(),
    }
}

fn get_integration<'a>(state: &'a InstalledPackageState, key: &str) -> Option<&'a str> {
    state.integrations.iter().find(|i| i.key == key).map(|i| i.value.as_str())
}

#[test]
fn config_prints_current_typed_package_settings_view() {
    let env = create_configured_home();
    let temp_home = env.home;
    let (ok, stdout, stderr) =
        run_in_home(&["config", "python", "pip_mirror", "tuna"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let (ok, stdout, stderr) = run_in_home(
        &["config", "git", "follow_spoon_proxy", "true"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);

    let (ok, stdout, stderr) = run_in_home(&["config"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "config:");
    assert_contains(
        &stdout,
        &format!(
            "config_file: {}",
            temp_home.join(".spoon").join("config.toml").display()
        ),
    );
    assert_contains(&stdout, "pip_mirror: tuna");
    assert_contains(&stdout, "command_profile:");
    assert_contains(&stdout, "default (");
    assert_contains(&stdout, "follow_spoon_proxy: true");
    assert_contains(&stdout, "command_profile:");
}

#[test]
fn configure_roots_prints_unset_when_not_configured() {
    let (ok, stdout, stderr) = run(&["config"]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "config:");
    assert_contains(&stdout, "root:");
}

#[test]
fn config_prints_raw_and_derived_sections() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let root_text = tool_root.display().to_string().replace('\\', "\\\\");

    let (ok, stdout, stderr) = run_in_home(&["config"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "config:");
    assert_contains(&stdout, "Root:");
    assert_contains(
        &stdout,
        &format!(
            "config_file: {}",
            temp_home.join(".spoon").join("config.toml").display()
        ),
    );
    assert_contains(&stdout, &format!("path: {root_text}"));
    assert_contains(&stdout, "Runtime:");
    assert_contains(&stdout, "Derived:");
    assert_contains(&stdout, "scoop_root:");
    assert_contains(&stdout, "managed_msvc_root:");
    assert_contains(&stdout, "managed_msvc_toolchain:");
    assert_contains(&stdout, "official_msvc_root:");
    assert_contains(&stdout, "msvc_target_arch:");
    assert_contains(&stdout, "Packages:");
}

#[test]
fn config_prints_official_runtime_root_when_selected() {
    let temp_home = create_test_home();
    let tool_root = temp_home.join("tool-root");
    write_test_config(&temp_home, &tool_root, "");

    let (ok, stdout, stderr) = run_in_home(&["config"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "managed_msvc_root:");
    assert_contains(&stdout, "managed_msvc_toolchain:");
    assert_contains(&stdout, "official_msvc_root:");
    assert_contains(&stdout, "msvc\\official\\instance");
}

#[test]
fn config_path_prints_global_config_path() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) = run_in_home(&["config", "path"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_eq!(
        stdout.trim(),
        temp_home
            .join(".spoon")
            .join("config.toml")
            .display()
            .to_string()
    );
}

#[test]
fn config_cat_prints_raw_config_toml() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) =
        run_in_home(&["config", "python", "pip_mirror", "tuna"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);

    let (ok, stdout, stderr) = run_in_home(&["config", "cat"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "root =");
    assert_contains(&stdout, "[policy.python]");
    assert_contains(&stdout, "pip_mirror = \"tuna\"");
}

#[test]
fn config_python_set_updates_policy_in_config() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) =
        run_in_home(&["config", "python", "pip_mirror", "tuna"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Updated config 'python.pip_mirror'.");
    assert_contains(&stdout, "pip_mirror: tuna");

    let config_text =
        std::fs::read_to_string(temp_home.join(".spoon").join("config.toml")).unwrap();
    assert_contains(&config_text, "[policy.python]");
    assert_contains(&config_text, "pip_mirror = \"tuna\"");
}

#[test]
fn config_python_accepts_inline_key_value_assignment() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) = run_in_home(
        &["config", "python", "command_profile=default"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "command_profile: default (pip, py, pyw, pythonw)");
}

#[test]
fn config_git_set_updates_policy_in_config() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) = run_in_home(
        &["config", "git", "follow_spoon_proxy", "true"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Updated config 'git.follow_spoon_proxy'.");
    assert_contains(&stdout, "follow_spoon_proxy: true");

    let config_text =
        std::fs::read_to_string(temp_home.join(".spoon").join("config.toml")).unwrap();
    assert_contains(&config_text, "[policy.git]");
    assert_contains(&config_text, "follow_spoon_proxy = true");
}

#[test]
fn config_git_accepts_inline_key_value_assignment() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) = run_in_home(
        &["config", "git", "command_profile=extended"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(
        &stdout,
        "command_profile: extended (bash, git-gui, gitk, scalar, tig)",
    );
}

#[test]
fn config_msvc_accepts_inline_key_value_assignment() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) = run_in_home(
        &["config", "msvc", "command_profile=default"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(
        &stdout,
        "command_profile: default (spoon-cl, spoon-link, spoon-lib)",
    );
}

#[test]
fn config_msvc_set_updates_policy_in_config() {
    let env = create_configured_home();
    let temp_home = env.home;

    let (ok, stdout, stderr) = run_in_home(
        &["config", "msvc", "command_profile", "extended"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Updated config 'msvc.command_profile'.");
    assert_contains(
        &stdout,
        "command_profile: extended (spoon-cl, spoon-link, spoon-lib, spoon-rc, spoon-mt, spoon-nmake, spoon-dumpbin)",
    );

    let config_text =
        std::fs::read_to_string(temp_home.join(".spoon").join("config.toml")).unwrap();
    assert_contains(&config_text, "[policy.msvc]");
    assert_contains(&config_text, "command_profile = \"extended\"");
}

#[test]
fn config_git_command_profile_updates_policy_in_config() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let bucket_root = tool_root
        .join("scoop")
        .join("buckets")
        .join("main")
        .join("bucket");
    std::fs::create_dir_all(&bucket_root).unwrap();
    std::fs::write(
        bucket_root.join("git.json"),
        r#"{
            "version": "2.53.0.2",
            "description": "Git for Windows",
            "homepage": "https://gitforwindows.org",
            "bin": ["bin\\git.exe", "git-bash.exe"]
        }"#,
    )
    .unwrap();
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &tool_root,
        &spoon::service::scoop::BucketSpec {
            name: "main".to_string(),
            source: Some("https://github.com/ScoopInstaller/Main".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(
        &["config", "git", "command_profile", "extended"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Updated config 'git.command_profile'.");
    assert_contains(
        &stdout,
        "command_profile: extended (bash, git-gui, gitk, scalar, tig)",
    );

    let config_text =
        std::fs::read_to_string(temp_home.join(".spoon").join("config.toml")).unwrap();
    assert_contains(&config_text, "[policy.git]");
    assert_contains(&config_text, "command_profile = \"extended\"");
}

#[test]
fn config_python_set_reapplies_pip_mirror_for_installed_python() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let layout = RuntimeLayout::from_root(&tool_root);
    let current_root = layout.scoop.package_current_root("python");
    std::fs::create_dir_all(&current_root).unwrap();
    let mut state = empty_python_state();
    spoon::runtime::test_block_on(write_installed_state(&layout.scoop, &state)).unwrap();

    let (ok, stdout, stderr) =
        run_in_home(&["config", "python", "pip_mirror", "tuna"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Applied Python pip mirror policy to");
    assert_contains(&stdout, "pip_mirror: tuna");

    let pip_ini = temp_home
        .join("AppData")
        .join("Roaming")
        .join("pip")
        .join("pip.ini");
    assert_path_exists(&pip_ini);
    let pip_text = std::fs::read_to_string(pip_ini).unwrap();
    assert_contains(
        &pip_text,
        "index-url=https://pypi.tuna.tsinghua.edu.cn/simple",
    );

    let state = spoon::runtime::test_block_on(spoon_scoop::read_installed_state(&layout.scoop, "python"))
        .unwrap().unwrap();
    assert_eq!(get_integration(&state, "python.pip_mirror"), Some("tuna"));
}

#[test]
fn config_git_set_reapplies_proxy_policy_for_installed_git() {
    let env = create_configured_home_with_proxy("http://127.0.0.1:7897");
    let temp_home = env.home;
    let tool_root = env.root;
    let layout = RuntimeLayout::from_root(&tool_root);
    let current_root = layout.scoop.package_current_root("git");
    std::fs::create_dir_all(&current_root).unwrap();
    spoon::runtime::test_block_on(write_installed_state(&layout.scoop, &empty_git_state())).unwrap();

    let (ok, stdout, stderr) = run_in_home(
        &["config", "git", "follow_spoon_proxy", "true"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Applied Git proxy policy to");
    assert_contains(&stdout, "follow_spoon_proxy: true");

    let gitconfig = temp_home.join(".gitconfig");
    assert_path_exists(&gitconfig);
    let gitconfig_text = std::fs::read_to_string(gitconfig).unwrap();
    assert_contains(&gitconfig_text, "proxy=http://127.0.0.1:7897");

    let state = spoon::runtime::test_block_on(spoon_scoop::read_installed_state(&layout.scoop, "git"))
        .unwrap().unwrap();
    assert_eq!(get_integration(&state, "git.follow_spoon_proxy"), Some("true"));
}

#[test]
fn config_python_import_reads_native_pip_config_into_policy() {
    let env = create_configured_home();
    let temp_home = env.home;
    let pip_dir = temp_home.join("AppData").join("Roaming").join("pip");
    std::fs::create_dir_all(&pip_dir).unwrap();
    std::fs::write(
        pip_dir.join("pip.ini"),
        "[global]\nindex-url = https://pypi.tuna.tsinghua.edu.cn/simple\n",
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["config", "python", "import"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Imported native config into 'python.pip_mirror'.");
    assert_contains(&stdout, "pip_mirror: tuna");

    let config_text =
        std::fs::read_to_string(temp_home.join(".spoon").join("config.toml")).unwrap();
    assert_contains(&config_text, "[policy.python]");
    assert_contains(&config_text, "pip_mirror = \"tuna\"");
}

#[test]
fn config_git_import_reads_matching_native_proxy_into_policy() {
    let env = create_configured_home_with_proxy("http://127.0.0.1:7897");
    let temp_home = env.home;
    std::fs::write(
        temp_home.join(".gitconfig"),
        "[http]\n\tproxy = http://127.0.0.1:7897\n[https]\n\tproxy = http://127.0.0.1:7897\n",
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["config", "git", "import"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(
        &stdout,
        "Imported native config into 'git.follow_spoon_proxy'.",
    );
    assert_contains(&stdout, "follow_spoon_proxy: true");

    let config_text =
        std::fs::read_to_string(temp_home.join(".spoon").join("config.toml")).unwrap();
    assert_contains(&config_text, "[policy.git]");
    assert_contains(&config_text, "follow_spoon_proxy = true");
}

#[test]
fn config_python_reports_conflict_when_native_index_differs_from_policy() {
    let env = create_configured_home();
    let temp_home = env.home;
    let (ok, stdout, stderr) =
        run_in_home(&["config", "python", "pip_mirror", "tuna"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    let pip_dir = temp_home.join("AppData").join("Roaming").join("pip");
    std::fs::create_dir_all(&pip_dir).unwrap();
    std::fs::write(
        pip_dir.join("pip.ini"),
        "[global]\nindex-url = https://example.invalid/simple\n",
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["config", "python"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Conflicts:");
    assert_contains(&stdout, "native pip index-url differs from Spoon policy");
}

#[test]
fn config_git_reports_conflict_when_native_proxy_differs_from_spoon_proxy() {
    let env = create_configured_home_with_proxy("http://127.0.0.1:7897");
    let temp_home = env.home;
    std::fs::write(
        temp_home.join(".spoon").join("config.toml"),
        format!(
            "editor = \"\"\nproxy = \"http://127.0.0.1:7897\"\nroot = \"{}\"\nmsvc_arch = \"auto\"\n\n[policy.git]\nfollow_spoon_proxy = true\n",
            env.root.display().to_string().replace('\\', "\\\\")
        ),
    )
    .unwrap();
    std::fs::write(
        temp_home.join(".gitconfig"),
        "[http]\n\tproxy = http://127.0.0.1:9999\n",
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(&["config", "git"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Conflicts:");
    assert_contains(&stdout, "native Git proxy differs from Spoon proxy");
}

#[test]
fn config_python_set_reapplies_command_profile_for_installed_python() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let package_root = RuntimeLayout::from_root(&tool_root)
        .scoop
        .package_app_root("python");
    let current_root = package_root.join("current");
    let scripts_root = current_root.join("Scripts");
    let bucket_root = tool_root
        .join("scoop")
        .join("buckets")
        .join("main")
        .join("bucket");
    let _ = std::fs::remove_dir_all(&package_root);
    std::fs::create_dir_all(&package_root).unwrap();
    std::fs::create_dir_all(&scripts_root).unwrap();
    std::fs::create_dir_all(&bucket_root).unwrap();
    std::fs::write(current_root.join("python.exe"), b"fake").unwrap();
    std::fs::write(current_root.join("pythonw.exe"), b"fake").unwrap();
    std::fs::write(current_root.join("py.exe"), b"fake").unwrap();
    std::fs::write(current_root.join("pyw.exe"), b"fake").unwrap();
    std::fs::write(scripts_root.join("pip.exe"), b"fake").unwrap();
    std::fs::write(scripts_root.join("pip3.exe"), b"fake").unwrap();
    std::fs::write(scripts_root.join("pip3.14.exe"), b"fake").unwrap();
    std::fs::write(
        bucket_root.join("python.json"),
        r#"{
            "version": "3.14.3",
            "url": "https://example.com/python.exe",
            "hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "bin": [["python.exe", "python3"]],
            "env_add_path": ["Scripts", "."]
        }"#,
    )
    .unwrap();
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &tool_root,
        &spoon::service::scoop::BucketSpec {
            name: "main".to_string(),
            source: Some("https://github.com/ScoopInstaller/Main".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();
    let layout = RuntimeLayout::from_root(&tool_root);
    spoon::runtime::test_block_on(write_installed_state(&layout.scoop, &empty_python_state())).unwrap();

    let (ok, stdout, stderr) = run_in_home(
        &["config", "python", "command_profile", "extended"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "command_profile: extended");
    let layout = RuntimeLayout::from_root(&tool_root);
    assert_path_exists(&layout.shims.join("pip3.cmd"));
    assert_path_exists(&layout.shims.join("pip3.14.cmd"));

    let state = spoon::runtime::test_block_on(spoon_scoop::read_installed_state(&layout.scoop, "python"))
        .unwrap().unwrap();
    assert!(state.command_surface.bins.iter().any(|bin| bin == "pip3"));
    assert!(state.command_surface.bins.iter().any(|bin| bin == "pip3.14"));
    let config_text =
        std::fs::read_to_string(temp_home.join(".spoon").join("config.toml")).unwrap();
    assert_contains(&config_text, "command_profile = \"extended\"");
}

#[test]
fn config_msvc_set_reapplies_command_profile_for_installed_managed_msvc() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    let layout = RuntimeLayout::from_root(&tool_root);
    let toolchain_root = layout.msvc.managed.toolchain_root.clone();
    let state_root = layout.msvc.managed.root.join("state");
    let bin_root = toolchain_root
        .join("VC")
        .join("Tools")
        .join("MSVC")
        .join("14.44.35207")
        .join("bin")
        .join("Hostx64")
        .join("x64");

    std::fs::create_dir_all(&bin_root).unwrap();
    let vc_include_root = toolchain_root
        .join("VC")
        .join("Tools")
        .join("MSVC")
        .join("14.44.35207")
        .join("include");
    let vc_lib_root = toolchain_root
        .join("VC")
        .join("Tools")
        .join("MSVC")
        .join("14.44.35207")
        .join("lib")
        .join("x64");
    let sdk_include_root = toolchain_root
        .join("Windows Kits")
        .join("10")
        .join("Include")
        .join("10.0.26100.15");
    let sdk_lib_root = toolchain_root
        .join("Windows Kits")
        .join("10")
        .join("Lib")
        .join("10.0.26100.15");
    std::fs::create_dir_all(&vc_include_root).unwrap();
    std::fs::create_dir_all(&sdk_include_root.join("ucrt")).unwrap();
    std::fs::create_dir_all(&sdk_include_root.join("shared")).unwrap();
    std::fs::create_dir_all(&sdk_include_root.join("um")).unwrap();
    std::fs::create_dir_all(&sdk_lib_root.join("ucrt").join("x64")).unwrap();
    std::fs::create_dir_all(&sdk_lib_root.join("um").join("x64")).unwrap();
    std::fs::create_dir_all(&vc_lib_root).unwrap();
    std::fs::create_dir_all(&state_root).unwrap();

    for name in [
        "cl.exe",
        "link.exe",
        "lib.exe",
        "rc.exe",
        "mt.exe",
        "nmake.exe",
        "dumpbin.exe",
    ] {
        std::fs::write(bin_root.join(name), b"fake").unwrap();
    }
    std::fs::write(vc_include_root.join("vcruntime.h"), b"").unwrap();
    std::fs::write(sdk_include_root.join("ucrt").join("stdio.h"), b"").unwrap();
    std::fs::write(sdk_include_root.join("shared").join("winapifamily.h"), b"").unwrap();
    std::fs::write(sdk_include_root.join("um").join("Windows.h"), b"").unwrap();
    std::fs::write(sdk_lib_root.join("ucrt").join("x64").join("ucrt.lib"), b"").unwrap();
    std::fs::write(
        sdk_lib_root.join("um").join("x64").join("kernel32.lib"),
        b"",
    )
    .unwrap();
    std::fs::write(vc_lib_root.join("libcmt.lib"), b"").unwrap();
    std::fs::write(
        state_root.join("runtime.json"),
        serde_json::json!({
            "toolchain_root": toolchain_root,
            "wrappers_root": layout.shims.clone(),
            "runtime": "managed"
        })
        .to_string(),
    )
    .unwrap();
    std::fs::write(
        state_root.join("installed.json"),
        serde_json::json!({
            "msvc": "msvc-14.44.35207",
            "sdk": "sdk-10.0.26100.15"
        })
        .to_string(),
    )
    .unwrap();

    let (ok, stdout, stderr) = run_in_home(
        &["config", "msvc", "command_profile", "extended"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(
        &stdout,
        "command_profile: extended (spoon-cl, spoon-link, spoon-lib, spoon-rc, spoon-mt, spoon-nmake, spoon-dumpbin)",
    );
    assert_path_exists(&layout.shims.join("spoon-rc.cmd"));
    assert_path_exists(&layout.shims.join("spoon-mt.cmd"));
    assert_path_exists(&layout.shims.join("spoon-nmake.cmd"));
    assert_path_exists(&layout.shims.join("spoon-dumpbin.cmd"));

    let (ok, stdout, stderr) = run_in_home(
        &["config", "msvc", "command_profile", "default"],
        &temp_home,
        &[],
    );
    assert_ok(ok, &stdout, &stderr);
    assert_contains(
        &stdout,
        "command_profile: default (spoon-cl, spoon-link, spoon-lib)",
    );
    assert!(
        !layout.shims.join("spoon-rc.cmd").exists(),
        "optional wrapper spoon-rc.cmd should be removed"
    );
}

#[test]
fn scoop_info_shows_applied_policy_integrations() {
    let env = create_configured_home();
    let temp_home = env.home;
    let tool_root = env.root;
    std::fs::write(
        temp_home.join(".spoon").join("config.toml"),
        format!(
            "editor = \"\"\nproxy = \"\"\nroot = \"{}\"\nmsvc_arch = \"auto\"\n\n[policy.python]\npip_mirror = \"tuna\"\ncommand_profile = \"extended\"\n",
            tool_root.display().to_string().replace('\\', "\\\\")
        ),
    )
    .unwrap();
    let bucket_root = tool_root
        .join("scoop")
        .join("buckets")
        .join("main")
        .join("bucket");
    std::fs::create_dir_all(&bucket_root).unwrap();
    std::fs::write(
        bucket_root.join("python.json"),
        r#"{
            "version": "3.14.3",
            "description": "Python runtime",
            "homepage": "https://www.python.org",
            "license": "Python-2.0",
            "bin": [["python.exe", "python3"]]
        }"#,
    )
    .unwrap();
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &tool_root,
        &spoon::service::scoop::BucketSpec {
            name: "main".to_string(),
            source: Some("https://github.com/ScoopInstaller/Main".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();
    let layout = RuntimeLayout::from_root(&tool_root);
    let current_root = layout.scoop.package_current_root("python");
    std::fs::create_dir_all(&current_root).unwrap();
    let mut state = empty_python_state();
    state.integrations = vec![
        integration("python.pip_mirror", "tuna"),
        integration("python.pip_config", "C:\\Users\\vision\\AppData\\Roaming\\pip\\pip.ini"),
        integration("python.pip_index_url", "https://pypi.tuna.tsinghua.edu.cn/simple"),
    ];
    spoon::runtime::test_block_on(write_installed_state(&layout.scoop, &state)).unwrap();

    let (ok, stdout, stderr) = run_in_home(&["scoop", "info", "python"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "Integration:");
    assert_contains(&stdout, "Policy:");
    assert_contains(&stdout, "desired:");
    assert_contains(&stdout, "command_profile: extended");
    assert_contains(
        &stdout,
        "extended (pip, py, pyw, pythonw, pip3, versioned pipX.Y shims)",
    );
    assert_contains(&stdout, "applied value:");
    assert_contains(
        &stdout,
        "pip_index_url: https://pypi.tuna.tsinghua.edu.cn/simple",
    );
    assert_contains(&stdout, "config file:");
    assert_contains(&stdout, "C:\\Users\\vision\\AppData\\Roaming\\pip\\pip.ini");
}

#[test]
fn scoop_info_does_not_repeat_desired_policy_inside_applied_values() {
    let env = create_configured_home_with_proxy("http://127.0.0.1:7897");
    let temp_home = env.home;
    let tool_root = env.root;
    std::fs::write(
        temp_home.join(".spoon").join("config.toml"),
        format!(
            "editor = \"\"\nproxy = \"http://127.0.0.1:7897\"\nroot = \"{}\"\nmsvc_arch = \"auto\"\n\n[policy.git]\nfollow_spoon_proxy = true\n",
            tool_root.display().to_string().replace('\\', "\\\\")
        ),
    )
    .unwrap();
    let bucket_root = tool_root
        .join("scoop")
        .join("buckets")
        .join("main")
        .join("bucket");
    std::fs::create_dir_all(&bucket_root).unwrap();
    std::fs::write(
        bucket_root.join("git.json"),
        r#"{
            "version": "2.53.0.2",
            "description": "Git for Windows",
            "homepage": "https://gitforwindows.org",
            "bin": ["git.exe"]
        }"#,
    )
    .unwrap();
    spoon::runtime::test_block_on(spoon::service::scoop::upsert_bucket_to_registry(
        &tool_root,
        &spoon::service::scoop::BucketSpec {
            name: "main".to_string(),
            source: Some("https://github.com/ScoopInstaller/Main".to_string()),
            branch: Some("master".to_string()),
        },
    ))
    .unwrap();
    let layout = RuntimeLayout::from_root(&tool_root);
    let current_root = layout.scoop.package_current_root("git");
    std::fs::create_dir_all(&current_root).unwrap();
    let mut state = InstalledPackageState {
        identity: InstalledPackageIdentity {
            package: "git".to_string(),
            version: "2.53.0.2".to_string(),
            bucket: "main".to_string(),
            architecture: Some("x64".to_string()),
            cache_size_bytes: None,
        },
        command_surface: InstalledPackageCommandSurface {
            bins: vec!["git".to_string(), "bash".to_string()],
            shortcuts: vec![],
            env_add_path: vec![],
            env_set: std::collections::BTreeMap::new(),
            persist: vec![],
        },
        integrations: vec![
            integration("git.follow_spoon_proxy", "true"),
            integration("git.proxy", "http://127.0.0.1:7897"),
            integration("git.config", "C:\\Users\\vision\\.gitconfig"),
        ],
        uninstall: InstalledPackageUninstall {
            pre_uninstall: vec![],
            uninstaller_script: vec![],
            post_uninstall: vec![],
        },
    };
    spoon::runtime::test_block_on(write_installed_state(&layout.scoop, &state)).unwrap();

    let (ok, stdout, stderr) = run_in_home(&["scoop", "info", "git"], &temp_home, &[]);
    assert_ok(ok, &stdout, &stderr);
    assert_contains(&stdout, "desired:");
    assert_contains(&stdout, "follow_spoon_proxy: true");
    assert_contains(&stdout, "command_profile: default (bash)");
    assert_contains(&stdout, "applied value:");
    assert_contains(&stdout, "proxy: http://127.0.0.1:7897");
    assert!(!stdout.contains("applied value: follow_spoon_proxy: true"));
}
