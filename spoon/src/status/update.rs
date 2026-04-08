use std::path::{Path, PathBuf};

use crate::config;
use crate::service::msvc;
use crate::service::scoop as scoop_backend;
use crate::status::ToolStatus;
use crate::packages::tool::{Backend, UpdateStrategy};

fn managed_tool_root(install_root: Option<&Path>) -> Option<PathBuf> {
    install_root
        .map(Path::to_path_buf)
        .or_else(config::configured_tool_root)
}

fn scoop_latest_version(package_name: &str, install_root: Option<&Path>) -> Option<String> {
    let tool_root = managed_tool_root(install_root)?;
    scoop_backend::latest_version(&tool_root, package_name)
}

pub fn populate_update_info(statuses: &mut [ToolStatus], install_root: Option<&Path>) {
    for status in statuses.iter_mut() {
        status.latest_version = None;
        status.update_available = false;
        if !status.is_detected() {
            continue;
        }

        match status.tool.update_strategy {
            UpdateStrategy::Backend => match status.tool.backend {
                Backend::Scoop => {
                    if let Some(latest) =
                        scoop_latest_version(status.tool.package_name, install_root)
                    {
                        status.update_available = status
                            .version
                            .as_deref()
                            .is_some_and(|current| current != latest);
                        status.latest_version = Some(latest);
                    } else {
                        status.latest_version = status.version.clone();
                    }
                }
                Backend::Native => {
                    if status.tool.has_managed_toolchain_runtime() {
                        if let Some(latest) = install_root.and_then(msvc::latest_toolchain_version_label) {
                            status.update_available = status
                                .version
                                .as_deref()
                                .is_some_and(|current| current != latest);
                            status.latest_version = Some(latest);
                        } else {
                            status.latest_version = status.version.clone();
                        }
                    } else {
                        status.latest_version = status.version.clone();
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::status::ToolStatus;
    use crate::packages::tool;

    use super::populate_update_info;

    #[test]
    fn scoop_latest_version_comes_from_bucket_manifest() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();

        let base = std::env::temp_dir().join(format!(
            "spoon-update-info-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let tool_root = base.join("root");
        let manifest = tool_root
            .join("scoop")
            .join("buckets")
            .join("main")
            .join("bucket")
            .join("jq.json");
        fs::create_dir_all(manifest.parent().unwrap()).unwrap();
        fs::write(&manifest, r#"{ "version": "1.8.2" }"#).unwrap();

        let jq_tool = tool::all_tools()
            .into_iter()
            .find(|tool| tool.key == "jq")
            .expect("jq tool");
        let mut statuses = vec![ToolStatus {
            tool: jq_tool,
            path: Some(PathBuf::from("D:/spoon/shims/jq.exe")),
            version: Some("1.8.1".to_string()),
            latest_version: None,
            installed_size_bytes: None,
            update_available: false,
            expected_dir: None,
            available: true,
            broken: false,
        }];

        populate_update_info(&mut statuses, Some(&tool_root));

        assert_eq!(statuses[0].latest_version.as_deref(), Some("1.8.2"));
        assert!(statuses[0].update_available);

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn scoop_latest_version_still_populates_when_current_matches() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();

        let base = std::env::temp_dir().join(format!(
            "spoon-update-info-same-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let tool_root = base.join("root");
        let manifest = tool_root
            .join("scoop")
            .join("buckets")
            .join("main")
            .join("bucket")
            .join("jq.json");
        fs::create_dir_all(manifest.parent().unwrap()).unwrap();
        fs::write(&manifest, r#"{ "version": "1.8.1" }"#).unwrap();

        let jq_tool = tool::all_tools()
            .into_iter()
            .find(|tool| tool.key == "jq")
            .expect("jq tool");
        let mut statuses = vec![ToolStatus {
            tool: jq_tool,
            path: Some(PathBuf::from("D:/spoon/shims/jq.exe")),
            version: Some("1.8.1".to_string()),
            latest_version: None,
            installed_size_bytes: None,
            update_available: false,
            expected_dir: None,
            available: true,
            broken: false,
        }];

        populate_update_info(&mut statuses, Some(&tool_root));

        assert_eq!(statuses[0].latest_version.as_deref(), Some("1.8.1"));
        assert!(!statuses[0].update_available);

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn msvc_latest_version_comes_from_cached_manifest() {
        use std::fs;

        crate::config::enable_test_mode();
        let base = std::env::temp_dir().join(format!(
            "spoon-msvc-update-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let tool_root = base.join("root");
        let manifest_root = spoon_core::RuntimeLayout::from_root(&tool_root).msvc.managed.manifest_root.join("vs");
        fs::create_dir_all(&manifest_root).unwrap();
        fs::write(
            manifest_root.join("latest.json"),
            serde_json::json!({
                "packages": [
                    {
                        "id": "Microsoft.VC.14.44.17.14.Tools.HostX64.TargetX64.base",
                        "version": "14.44.17.14",
                        "language": "neutral",
                        "payloads": []
                    },
                    {
                        "id": "WindowsSdkPackageB",
                        "version": "10.0.22621.7",
                        "language": "en-US",
                        "payloads": [
                            {
                                "fileName": "Installers\\Windows SDK for Windows Store Apps Tools-x86_en-us.msi"
                            }
                        ]
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();

        let msvc_tool = tool::all_tools()
            .into_iter()
            .find(|tool| tool.key == "msvc")
            .expect("msvc tool");
        let mut statuses = vec![ToolStatus {
            tool: msvc_tool,
            path: Some(tool_root.join("msvc").join("managed").join("toolchain")),
            version: Some("14.44.17.14 + 10.0.22000.0".to_string()),
            latest_version: None,
            installed_size_bytes: None,
            update_available: false,
            expected_dir: None,
            available: true,
            broken: false,
        }];

        populate_update_info(&mut statuses, Some(&tool_root));

        assert_eq!(
            statuses[0].latest_version.as_deref(),
            Some("14.44.17.14 + 10.0.22621.7")
        );
        assert!(statuses[0].update_available);

        let _ = fs::remove_dir_all(base);
    }
}
