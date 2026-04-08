use std::path::Path;

use crate::actions::ToolAction;
use crate::packages::tool::{Backend, EntityKind};
use crate::{config, packages::tool};
use spoon_core::RuntimeLayout;

use super::{ToolStatus, collect_statuses};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedReadiness {
    Missing,
    Broken,
    Detected,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionPolicy {
    pub install: bool,
    pub update: bool,
    pub uninstall: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolOwnership {
    Managed,
    External,
    Missing,
}

impl ManagedReadiness {
    pub fn label(self) -> &'static str {
        match self {
            ManagedReadiness::Missing => "missing",
            ManagedReadiness::Broken => "broken",
            ManagedReadiness::Detected => "detected",
            ManagedReadiness::Ready => "ready",
        }
    }

    pub fn is_detected(self) -> bool {
        !matches!(self, ManagedReadiness::Missing)
    }

    pub fn is_ready(self) -> bool {
        matches!(self, ManagedReadiness::Ready)
    }
}

impl ToolOwnership {
    pub fn label(self) -> &'static str {
        match self {
            ToolOwnership::Managed => "managed",
            ToolOwnership::External => "external",
            ToolOwnership::Missing => "-",
        }
    }
}

impl ActionPolicy {
    pub fn allows(self, action: ToolAction) -> bool {
        match action {
            ToolAction::Install => self.install,
            ToolAction::Update => self.update,
            ToolAction::Uninstall => self.uninstall,
        }
    }
}

impl ToolStatus {
    pub fn readiness(&self) -> ManagedReadiness {
        if self.broken {
            ManagedReadiness::Broken
        } else if self.is_usable() {
            ManagedReadiness::Ready
        } else if self.is_detected() {
            ManagedReadiness::Detected
        } else {
            ManagedReadiness::Missing
        }
    }

    pub fn ownership(&self) -> ToolOwnership {
        ownership_for_status(self)
    }
}

pub fn tool_detected(tool_key: &str, install_root: Option<&Path>) -> bool {
    tool_readiness(tool_key, install_root).is_detected()
}

pub fn tool_readiness(tool_key: &str, install_root: Option<&Path>) -> ManagedReadiness {
    collect_statuses(install_root)
        .iter()
        .find(|status| status.tool.key == tool_key)
        .map(ToolStatus::readiness)
        .unwrap_or(ManagedReadiness::Missing)
}

fn ownership_for_status(status: &ToolStatus) -> ToolOwnership {
    let Some(path) = status.path.as_deref() else {
        return ToolOwnership::Missing;
    };
    let Some(tool_root) = config::configured_tool_root() else {
        return ToolOwnership::External;
    };
    let layout = RuntimeLayout::from_root(&tool_root);

    let _scoop_root = layout.scoop.root;
    let msvc_root = layout.msvc.managed.root;
    let owned = match status.tool.backend {
        Backend::Native if status.tool.has_managed_toolchain_runtime() => {
            path.starts_with(&msvc_root)
        }
        Backend::Scoop => {
            let package_root = layout
                .scoop
                .apps_root
                .join(status.tool.package_name)
                .join("current");
            let shims_root = layout.shims;
            path.starts_with(&package_root) || path.starts_with(&shims_root)
        }
        _ => tool::expected_tool_dir(Some(&tool_root), status.tool)
            .as_deref()
            .is_some_and(|root| path.starts_with(root)),
    };

    if owned {
        ToolOwnership::Managed
    } else {
        ToolOwnership::External
    }
}

pub fn action_policy(status: &ToolStatus, statuses: &[ToolStatus]) -> ActionPolicy {
    let readiness = status.readiness();
    let ownership = status.ownership();
    let _ = statuses;

    match status.tool.kind {
        EntityKind::Toolchain => ActionPolicy {
            install: matches!(
                readiness,
                ManagedReadiness::Missing | ManagedReadiness::Broken
            ),
            update: readiness.is_ready() && status.update_available,
            uninstall: readiness.is_detected(),
        },
        EntityKind::Tool => {
            if matches!(ownership, ToolOwnership::External) {
                return ActionPolicy {
                    install: false,
                    update: false,
                    uninstall: false,
                };
            }
            ActionPolicy {
                install: matches!(
                    readiness,
                    ManagedReadiness::Missing | ManagedReadiness::Broken
                ),
                update: readiness.is_detected() && status.update_available,
                uninstall: readiness.is_detected(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::packages::tool;
    use spoon_core::RuntimeLayout;

    use super::{ToolOwnership, ToolStatus, action_policy};

    #[test]
    fn ownership_distinguishes_managed_scoop_package_from_external_provider() {
        crate::config::enable_test_mode();
        let temp_home = std::env::temp_dir().join(format!(
            "spoon-status-ownership-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&temp_home);
        std::fs::create_dir_all(&temp_home).unwrap();
        crate::config::set_home_override(temp_home.clone());

        let tool_root = temp_home.join("root");
        crate::config::save_global_config(&crate::config::GlobalConfig {
            editor: String::new(),
            proxy: String::new(),
            root: tool_root.display().to_string(),
            msvc_arch: crate::config::native_msvc_arch().to_string(),
        })
        .unwrap();

        let jq_tool = tool::find_tool("jq").unwrap();
        let managed = ToolStatus {
            tool: jq_tool,
            path: Some(
                RuntimeLayout::from_root(&tool_root)
                    .shims
                    .join("jq.exe"),
            ),
            version: None,
            latest_version: None,
            installed_size_bytes: None,
            update_available: false,
            expected_dir: None,
            available: true,
            broken: false,
        };
        let external = ToolStatus {
            tool: jq_tool,
            path: Some(std::path::PathBuf::from("C:/external/jq.exe")),
            version: None,
            latest_version: None,
            installed_size_bytes: None,
            update_available: false,
            expected_dir: None,
            available: true,
            broken: false,
        };

        assert_eq!(managed.ownership(), ToolOwnership::Managed);
        assert_eq!(external.ownership(), ToolOwnership::External);
    }

    #[test]
    fn msvc_update_requires_real_update_signal() {
        let msvc_tool = tool::find_tool("msvc").unwrap();
        let status = ToolStatus {
            tool: msvc_tool,
            path: Some(std::path::PathBuf::from("D:/spoon/msvc/managed/toolchain")),
            version: Some("14.44.17.14 + 10.0.26100.15".to_string()),
            latest_version: Some("14.44.17.14 + 10.0.26100.15".to_string()),
            installed_size_bytes: None,
            update_available: false,
            expected_dir: None,
            available: true,
            broken: false,
        };

        let policy = action_policy(&status, &[]);
        assert!(!policy.update);
        assert!(policy.uninstall);
    }

    #[test]
    fn external_tool_is_not_actionable() {
        crate::config::enable_test_mode();
        let temp_home = std::env::temp_dir().join(format!(
            "spoon-status-policy-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&temp_home);
        std::fs::create_dir_all(&temp_home).unwrap();
        crate::config::set_home_override(temp_home.clone());

        let tool_root = temp_home.join("root");
        crate::config::save_global_config(&crate::config::GlobalConfig {
            editor: String::new(),
            proxy: String::new(),
            root: tool_root.display().to_string(),
            msvc_arch: crate::config::native_msvc_arch().to_string(),
        })
        .unwrap();

        let jq_tool = tool::find_tool("jq").unwrap();
        let external = ToolStatus {
            tool: jq_tool,
            path: Some(std::path::PathBuf::from("C:/external/jq.exe")),
            version: Some("1.8.1".to_string()),
            latest_version: Some("1.8.2".to_string()),
            installed_size_bytes: None,
            update_available: true,
            expected_dir: None,
            available: true,
            broken: false,
        };

        let policy = action_policy(&external, &[]);
        assert!(!policy.install);
        assert!(!policy.update);
        assert!(!policy.uninstall);
    }
}
