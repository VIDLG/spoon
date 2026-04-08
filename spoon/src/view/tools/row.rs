use crate::actions::ToolAction;
use crate::formatting::format_bytes;
use crate::status::{self, ToolOwnership, ToolStatus};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ToolStatusRow {
    pub display_name: String,
    pub tag_label: String,
    pub status_label: String,
    pub backend_label: String,
    pub version: String,
    pub latest_version: String,
    pub installed_size: String,
    pub install_enabled: bool,
    pub update_enabled: bool,
    pub uninstall_enabled: bool,
}

impl ToolStatusRow {
    pub fn from_status(status: &ToolStatus, statuses: &[ToolStatus]) -> Self {
        let action_policy = status::action_policy(status, statuses);
        Self {
            display_name: status.tool.display_name.to_string(),
            tag_label: status.tool.tag.short_label().to_string(),
            status_label: Self::combined_status_label(status),
            backend_label: status.tool.backend.label().to_string(),
            version: status.version.clone().unwrap_or_else(|| "-".to_string()),
            latest_version: Self::display_latest_version(status),
            installed_size: status
                .installed_size_bytes
                .map(format_bytes)
                .unwrap_or_else(|| "-".to_string()),
            install_enabled: action_policy.allows(ToolAction::Install),
            update_enabled: action_policy.allows(ToolAction::Update),
            uninstall_enabled: action_policy.allows(ToolAction::Uninstall),
        }
    }

    fn display_latest_version(status: &ToolStatus) -> String {
        let latest = status
            .latest_version
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let current = status.version.as_deref().map(str::trim);
        match (latest, current, status.update_available) {
            (Some(latest), Some(current), false) if latest == current => "-".to_string(),
            (Some(latest), _, _) => latest.to_string(),
            _ => "-".to_string(),
        }
    }

    fn combined_status_label(status: &ToolStatus) -> String {
        match status.ownership() {
            ToolOwnership::Missing => "missing".to_string(),
            ToolOwnership::Managed => {
                if status.broken {
                    "managed!".to_string()
                } else if status.is_usable() {
                    "managed".to_string()
                } else if status.is_detected() {
                    "managed?".to_string()
                } else {
                    "missing".to_string()
                }
            }
            ToolOwnership::External => {
                if status.broken {
                    "external!".to_string()
                } else if status.is_usable() {
                    "external".to_string()
                } else if status.is_detected() {
                    "external?".to_string()
                } else {
                    "missing".to_string()
                }
            }
        }
    }
}

pub fn build_tool_status_row(status: &ToolStatus, statuses: &[ToolStatus]) -> ToolStatusRow {
    ToolStatusRow::from_status(status, statuses)
}
