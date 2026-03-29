use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use spoon_backend::layout::RuntimeLayout;
use spoon_backend::status::BackendStatusSnapshot;

use crate::config;
use crate::service::msvc;
use crate::status::ToolStatus;
use crate::tool::{self, Tool};

#[derive(Debug, Clone)]
pub(super) struct ProbeResult {
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    pub available: bool,
}

#[derive(Debug, Clone)]
struct VersionProbe {
    version: Option<String>,
    available: bool,
}

pub fn command_path(command: &str) -> Option<PathBuf> {
    which::which(command).ok()
}

fn command_path_in_dirs<I>(command: &str, dirs: I) -> Option<PathBuf>
where
    I: IntoIterator<Item = PathBuf>,
{
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for dir in dirs {
        if let Ok(path) = which::which_in(command, Some(dir), &cwd) {
            return Some(path);
        }
    }
    None
}

fn normalize_output_lines(raw: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(raw)
        .replace('\r', "\n")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn extract_version_token(line: &str) -> Option<String> {
    let chars: Vec<char> = line.chars().collect();
    for start in 0..chars.len() {
        let ch = chars[start];
        let starts_version = ch.is_ascii_digit()
            || ((ch == 'v' || ch == 'V')
                && chars
                    .get(start + 1)
                    .copied()
                    .is_some_and(|next| next.is_ascii_digit()));
        if !starts_version {
            continue;
        }

        let mut end = start;
        while end < chars.len() {
            let current = chars[end];
            if current.is_ascii_alphanumeric() || matches!(current, '.' | '-' | '_' | '+') {
                end += 1;
            } else {
                break;
            }
        }

        let mut candidate: String = chars[start..end].iter().collect();
        while matches!(candidate.chars().last(), Some(ch) if !ch.is_ascii_alphanumeric()) {
            candidate.pop();
        }
        if candidate.len() >= 2
            && (candidate.starts_with('v') || candidate.starts_with('V'))
            && candidate
                .chars()
                .nth(1)
                .is_some_and(|next| next.is_ascii_digit())
        {
            candidate.remove(0);
        }
        if candidate.chars().any(|ch| ch.is_ascii_digit()) && candidate.contains('.') {
            return Some(candidate);
        }
    }
    None
}

fn parse_version_from_output(_tool: &Tool, _path: &Path, lines: &[String]) -> Option<String> {
    lines.iter().find_map(|line| extract_version_token(line))
}

fn probe_version(tool: &Tool, path: &Path) -> Option<VersionProbe> {
    let output = Command::new(path).args(tool.version_args).output().ok()?;
    let mut lines = normalize_output_lines(&output.stdout);
    lines.extend(normalize_output_lines(&output.stderr));
    Some(VersionProbe {
        version: parse_version_from_output(tool, path, &lines),
        available: output.status.success(),
    })
}

fn managed_scoop_state_version(
    tool: &'static Tool,
    _install_root: Option<&Path>,
    snapshot: Option<&BackendStatusSnapshot>,
) -> Option<String> {
    if tool.backend != crate::tool::Backend::Scoop {
        return None;
    }
    // D-07/D-08: consume backend snapshot, not app-side state file IO
    let snap = snapshot?;
    snap.installed_package_version(tool.package_name)
        .map(ToString::to_string)
}

fn is_windowsapps_alias_path(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|part| part.eq_ignore_ascii_case("WindowsApps"))
    })
}

fn configured_probe_path(tool: &'static Tool, install_root: Option<&Path>) -> Option<PathBuf> {
    let root = install_root
        .map(Path::to_path_buf)
        .or_else(config::configured_tool_root)?;
    match tool.backend {
        crate::tool::Backend::Scoop => {
            let layout = RuntimeLayout::from_root(&root);
            command_path_in_dirs(tool.command, std::iter::once(layout.shims))
        }
        _ => None,
    }
}

fn resolved_probe_path(tool: &'static Tool, install_root: Option<&Path>) -> Option<PathBuf> {
    if tool.prefers_configured_probe_path_only() {
        configured_probe_path(tool, install_root)
    } else {
        configured_probe_path(tool, install_root).or_else(|| command_path(tool.command))
    }
}

fn probe_msvc_toolchain(install_root: Option<&Path>) -> ProbeResult {
    let Some(root) = install_root
        .map(Path::to_path_buf)
        .or_else(config::configured_tool_root)
    else {
        return ProbeResult {
            path: None,
            version: None,
            available: false,
        };
    };

    let layout = RuntimeLayout::from_root(&root);
    let managed_root = layout.msvc.managed.toolchain_root.clone();
    let managed_runtime_state = msvc::runtime_state_path(&root);
    if managed_runtime_state.exists() {
        return ProbeResult {
            path: Some(managed_root),
            version: msvc::installed_toolchain_version_label(&root),
            available: true,
        };
    }
    if managed_root.exists() {
        return ProbeResult {
            path: Some(managed_root),
            version: msvc::installed_toolchain_version_label(&root),
            available: false,
        };
    }

    let (official_root, official_available, official_version) = msvc::official::probe(&root);
    if official_available || official_root.exists() {
        return ProbeResult {
            path: Some(official_root),
            version: official_version,
            available: official_available,
        };
    }
    ProbeResult {
        path: None,
        version: None,
        available: false,
    }
}

fn probe_tool(tool: &'static Tool, install_root: Option<&Path>) -> ProbeResult {
    probe_tool_with_snapshot(tool, install_root, None)
}

fn probe_tool_with_snapshot(
    tool: &'static Tool,
    install_root: Option<&Path>,
    snapshot: Option<&BackendStatusSnapshot>,
) -> ProbeResult {
    if tool.has_managed_toolchain_runtime() {
        return probe_msvc_toolchain(install_root);
    }
    let path = resolved_probe_path(tool, install_root);
    let Some(path) = path else {
        return ProbeResult {
            path: None,
            version: None,
            available: false,
        };
    };

    let version_probe = probe_version(tool, &path).unwrap_or(VersionProbe {
        version: None,
        available: false,
    });
    let version_probe_version = version_probe.version.clone();
    let version =
        managed_scoop_state_version(tool, install_root, snapshot).or(version_probe_version.clone());
    if is_windowsapps_alias_path(&path)
        && !version_probe.available
        && version_probe_version.is_none()
    {
        return ProbeResult {
            path: None,
            version: None,
            available: false,
        };
    }
    ProbeResult {
        path: Some(path),
        version,
        available: version_probe.available,
    }
}

pub fn collect_statuses_fast(install_root: Option<&Path>) -> Vec<ToolStatus> {
    collect_statuses_fast_with_snapshot(install_root, None)
}

pub fn collect_statuses_fast_with_snapshot(
    install_root: Option<&Path>,
    snapshot: Option<&BackendStatusSnapshot>,
) -> Vec<ToolStatus> {
    tool::all_tools()
        .into_iter()
        .map(|tool| {
            let probe = if tool.has_managed_toolchain_runtime() {
                probe_msvc_toolchain(install_root)
            } else {
                let path = resolved_probe_path(tool, install_root);
                let version = managed_scoop_state_version(tool, install_root, snapshot);
                ProbeResult {
                    available: path.is_some(),
                    path,
                    version,
                }
            };
            ToolStatus {
                tool,
                path: probe.path,
                version: probe.version,
                latest_version: None,
                installed_size_bytes: None,
                update_available: false,
                expected_dir: tool::expected_tool_dir(install_root, tool),
                available: probe.available,
                broken: false,
            }
        })
        .collect()
}

pub fn collect_statuses(install_root: Option<&Path>) -> Vec<ToolStatus> {
    collect_statuses_with_snapshot(install_root, None)
}

pub fn collect_statuses_with_snapshot(
    install_root: Option<&Path>,
    snapshot: Option<&BackendStatusSnapshot>,
) -> Vec<ToolStatus> {
    tool::all_tools()
        .into_iter()
        .map(|tool| {
            let probe = probe_tool_with_snapshot(tool, install_root, snapshot);
            ToolStatus {
                tool,
                broken: probe.path.is_some() && !probe.available,
                path: probe.path,
                version: probe.version,
                available: probe.available,
                latest_version: None,
                installed_size_bytes: None,
                update_available: false,
                expected_dir: tool::expected_tool_dir(install_root, tool),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use crate::tool;
    use spoon_backend::RuntimeLayout;
    use spoon_backend::status::BackendStatusSnapshot;

    use super::{
        configured_probe_path, extract_version_token, is_windowsapps_alias_path,
        managed_scoop_state_version, probe_msvc_toolchain, resolved_probe_path,
    };

    #[test]
    fn extract_version_token_handles_common_cli_formats() {
        assert_eq!(
            extract_version_token("gh version 2.88.0 (2026-03-10)"),
            Some("2.88.0".to_string())
        );
        assert_eq!(
            extract_version_token("yq (https://github.com/mikefarah/yq/) version v4.52.4"),
            Some("4.52.4".to_string())
        );
        assert_eq!(
            extract_version_token("codex-cli 0.114.0"),
            Some("0.114.0".to_string())
        );
        assert_eq!(
            extract_version_token("2.1.74 (Claude Code)"),
            Some("2.1.74".to_string())
        );
        assert_eq!(extract_version_token("jq-1.8.1"), Some("1.8.1".to_string()));
    }

    #[test]
    fn msvc_toolchain_reads_managed_toolchain_root() {
        let base = std::env::temp_dir().join(format!(
            "spoon-msvc-probe-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let tool_root = base.join("root");
        let msvc_root = RuntimeLayout::from_root(&tool_root).msvc.managed.toolchain_root;
        let state_root = RuntimeLayout::from_root(&tool_root).msvc.managed.state_root;
        fs::create_dir_all(&state_root).unwrap();
        fs::write(crate::service::msvc::runtime_state_path(&tool_root), "{}").unwrap();

        let installed = state_root.join("installed.json");
        fs::write(
            &installed,
            serde_json::json!({
                "msvc": "msvc-14.44.17.14",
                "sdk": "sdk-10.0.22621.7"
            })
            .to_string(),
        )
        .unwrap();
        let probe = probe_msvc_toolchain(Some(&tool_root));
        assert_eq!(probe.path.as_deref(), Some(msvc_root.as_path()));
        assert!(probe.available);
        assert_eq!(probe.version.as_deref(), Some("14.44.17.14 + 10.0.22621.7"));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn git_probe_does_not_fallback_to_external_path() {
        crate::config::enable_test_mode();
        let temp_home = std::env::temp_dir().join(format!(
            "spoon-git-probe-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&temp_home);
        fs::create_dir_all(&temp_home).unwrap();
        crate::config::set_home_override(temp_home.clone());

        let tool_root = temp_home.join("root");
        crate::config::save_global_config(&crate::config::GlobalConfig {
            editor: String::new(),
            proxy: String::new(),
            root: tool_root.display().to_string(),
            msvc_arch: crate::config::native_msvc_arch().to_string(),
        })
        .unwrap();

        let git = tool::find_tool("git").expect("git tool");
        let status = super::probe_tool(git, Some(&tool_root));
        assert!(status.path.is_none(), "status: {:?}", status.path);
        assert!(!status.available);
    }

    #[test]
    fn msvc_toolchain_is_broken_when_root_exists_without_runtime_state() {
        let base = std::env::temp_dir().join(format!(
            "spoon-msvc-probe-broken-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let tool_root = base.join("root");
        let msvc_root = RuntimeLayout::from_root(&tool_root).msvc.managed.toolchain_root;
        fs::create_dir_all(&msvc_root).unwrap();

        let probe = probe_msvc_toolchain(Some(&tool_root));
        assert_eq!(probe.path.as_deref(), Some(msvc_root.as_path()));
        assert!(!probe.available);
        assert!(probe.version.is_none());

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn scoop_managed_tools_prefer_configured_shims_over_process_path() {
        let base = std::env::temp_dir().join(format!(
            "spoon-scoop-probe-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let root = base.join("root");
        let shims = RuntimeLayout::from_root(&root).shims;
        fs::create_dir_all(&shims).unwrap();
        let managed = shims.join("gh.exe");
        fs::write(&managed, "").unwrap();

        let gh = tool::all_tools()
            .into_iter()
            .find(|tool| tool.key == "gh")
            .expect("gh tool");

        assert_eq!(
            configured_probe_path(gh, Some(&root)),
            Some(PathBuf::from(&managed))
        );

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn configured_probe_path_prefers_managed_python3_shim() {
        let base = std::env::temp_dir().join(format!(
            "spoon-python-probe-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let tool_root = base.join("root");
        let shims = RuntimeLayout::from_root(&tool_root).shims;
        fs::create_dir_all(&shims).unwrap();
        fs::write(shims.join("python3.exe"), "fake").unwrap();

        let tool = tool::find_tool("python").expect("python tool");
        let path = configured_probe_path(tool, Some(&tool_root));
        assert_eq!(path.as_deref(), Some(shims.join("python3.exe").as_path()));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn managed_scoop_state_version_reads_installed_package_version() {
        let snap = BackendStatusSnapshot {
            kind: "test",
            scoop: spoon_backend::status::BackendScoopStatus {
                installed: true,
                root: String::new(),
                shims: String::new(),
                bucket_count: 0,
                installed_package_count: 1,
                buckets: vec![],
                installed_packages: vec![
                    spoon_backend::status::BackendInstalledPackageEntry {
                        name: "git".into(),
                        version: "2.53.0.2".into(),
                    },
                ],
            },
            msvc: spoon_backend::status::BackendMsvcSummary {
                managed_status: String::new(),
                managed_version: None,
                managed_root: String::new(),
                official_status: String::new(),
                official_version: None,
                official_root: String::new(),
            },
            runtime_roots: spoon_backend::status::BackendRuntimeRoots::default(),
        };

        let git = tool::find_tool("git").expect("git tool");
        assert_eq!(
            managed_scoop_state_version(git, None, Some(&snap)).as_deref(),
            Some("2.53.0.2")
        );
    }

    #[test]
    fn managed_7zip_probe_uses_cli_compatible_version_args() {
        let base = std::env::temp_dir().join(format!(
            "spoon-7zip-probe-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let tool_root = base.join("root");
        let shims = RuntimeLayout::from_root(&tool_root).shims;
        let state_root = RuntimeLayout::from_root(&tool_root).scoop.package_state_root;
        fs::create_dir_all(&shims).unwrap();
        fs::create_dir_all(&state_root).unwrap();
        fs::write(
            shims.join("7z.cmd"),
            "@echo off\r\nif \"%~1\"==\"i\" (\r\n  echo 7-Zip 26.00 ^(x64^)\r\n  exit /b 0\r\n)\r\n>&2 echo Unknown switch: %~1\r\nexit /b 2\r\n",
        )
        .unwrap();
        fs::write(
            state_root.join("7zip.json"),
            serde_json::json!({
                "package": "7zip",
                "version": "26.00"
            })
            .to_string(),
        )
        .unwrap();

        let tool = tool::find_tool("7zip").expect("7zip tool");
        let probe = super::probe_tool(tool, Some(&tool_root));
        assert_eq!(probe.path.as_deref(), Some(shims.join("7z.cmd").as_path()));
        assert!(probe.available, "probe should use 7-Zip compatible args");
        assert_eq!(probe.version.as_deref(), Some("26.00"));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn fast_and_full_probe_share_managed_path_resolution_for_scoop_tools() {
        let base = std::env::temp_dir().join(format!(
            "spoon-probe-consistency-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let tool_root = base.join("root");
        let shims = RuntimeLayout::from_root(&tool_root).shims;
        fs::create_dir_all(&shims).unwrap();
        fs::write(
            shims.join("gh.cmd"),
            "@echo off\r\necho gh version 2.88.1\r\nexit /b 0\r\n",
        )
        .unwrap();
        fs::write(
            shims.join("7z.cmd"),
            "@echo off\r\nif \"%~1\"==\"i\" (\r\n  echo 7-Zip 26.00 ^(x64^)\r\n  exit /b 0\r\n)\r\nexit /b 2\r\n",
        )
        .unwrap();

        let snap = BackendStatusSnapshot {
            kind: "test",
            scoop: spoon_backend::status::BackendScoopStatus {
                installed: true,
                root: String::new(),
                shims: String::new(),
                bucket_count: 0,
                installed_package_count: 2,
                buckets: vec![],
                installed_packages: vec![
                    spoon_backend::status::BackendInstalledPackageEntry {
                        name: "gh".into(),
                        version: "2.88.1".into(),
                    },
                    spoon_backend::status::BackendInstalledPackageEntry {
                        name: "7zip".into(),
                        version: "26.00".into(),
                    },
                ],
            },
            msvc: spoon_backend::status::BackendMsvcSummary {
                managed_status: String::new(),
                managed_version: None,
                managed_root: String::new(),
                official_status: String::new(),
                official_version: None,
                official_root: String::new(),
            },
            runtime_roots: spoon_backend::status::BackendRuntimeRoots::default(),
        };

        for key in ["gh", "7zip"] {
            let tool = tool::find_tool(key).expect("tool");
            let fast = super::collect_statuses_fast_with_snapshot(Some(&tool_root), Some(&snap))
                .into_iter()
                .find(|status| status.tool.key == key)
                .expect("fast status");
            let full = super::collect_statuses_with_snapshot(Some(&tool_root), Some(&snap))
                .into_iter()
                .find(|status| status.tool.key == key)
                .expect("full status");

            assert_eq!(resolved_probe_path(tool, Some(&tool_root)), fast.path);
            assert_eq!(fast.path, full.path, "path drift for {key}");
            assert_eq!(fast.version, full.version, "version drift for {key}");
            assert!(full.available, "full probe should stay usable for {key}");
            assert!(!full.broken, "full probe should not mark {key} broken");
        }

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn windowsapps_alias_path_is_recognized() {
        assert!(is_windowsapps_alias_path(Path::new(
            "C:/Users/vision/AppData/Local/Microsoft/WindowsApps/python3.exe"
        )));
        assert!(!is_windowsapps_alias_path(Path::new(
            "C:/Users/vision/.local/bin/python3.exe"
        )));
    }
}
