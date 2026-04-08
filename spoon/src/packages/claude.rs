use crate::config;
use crate::packages::tool::{
    Backend, DetailRuntimeKind, EntityKind, ProbePathPolicy, Tool, ToolCategory, ToolTag,
    UpdateStrategy,
};
use serde_json::json;

use super::{
    AssistantConfigSummary, ConfigBadgeTone, ConfigEntry, ConfigTargetBadge,
    ConfigTargetDescriptor, PackageConfigDetails, PackageIdentity, PackageSpec,
    config_target_from_identity,
};

pub(super) struct ClaudePackage;

const IDENTITY: PackageIdentity = PackageIdentity {
    key: "claude",
    display_name: "Claude Code",
    order: 20,
};

pub(crate) const TOOL: Tool = Tool {
    key: IDENTITY.key,
    display_name: IDENTITY.display_name,
    summary: "Anthropic's terminal coding assistant for interactive development work.",
    homepage: "https://code.claude.com",
    command: "claude",
    package_name: "claude-code",
    dir_name: "claude-code",
    category: ToolCategory::Core,
    tag: ToolTag::Core,
    kind: EntityKind::Tool,
    backend: Backend::Scoop,
    detail_runtime: DetailRuntimeKind::Standard,
    probe_path_policy: ProbePathPolicy::Default,
    owned_root_markers: None,
    depends_on: &["git"],
    version_args: &["--version"],
    update_strategy: UpdateStrategy::Backend,
};

impl ClaudePackage {
    pub const fn new() -> Self {
        Self
    }
}

impl PackageSpec for ClaudePackage {
    fn identity(&self) -> PackageIdentity {
        IDENTITY
    }

    fn config_target(&self) -> Option<ConfigTargetDescriptor> {
        Some(config_target_from_identity(
            IDENTITY,
            "Configure Claude Code",
            true,
            true,
        ))
    }

    fn config_summary_entries(&self) -> Vec<ConfigEntry> {
        let config = config::load_claude_config();
        vec![
            ConfigEntry::plain("base_url", empty_to_unset(&config.base_url)),
            ConfigEntry::plain(
                "auth_token",
                if config.auth_token.trim().is_empty() {
                    "missing".to_string()
                } else {
                    "present".to_string()
                },
            ),
        ]
    }

    fn config_menu_summary_lines(&self) -> Vec<String> {
        vec![format!(
            "status: {}",
            if is_configured() {
                "configured"
            } else {
                "missing"
            }
        )]
    }

    fn config_target_badge(&self) -> Option<ConfigTargetBadge> {
        Some(ConfigTargetBadge {
            label: if is_configured() {
                "configured"
            } else {
                "missing"
            },
            tone: if is_configured() {
                ConfigBadgeTone::Ready
            } else {
                ConfigBadgeTone::Missing
            },
        })
    }

    fn config_details(&self) -> Option<PackageConfigDetails> {
        let config = config::load_claude_config();
        Some(PackageConfigDetails::Summary(AssistantConfigSummary {
            scope: "claude",
            config_files: vec![config::claude_settings_path().display().to_string()],
            detected: json!({
                "base_url": maybe_trimmed(&config.base_url),
                "auth_token_present": !config.auth_token.trim().is_empty(),
            }),
            detected_entries: vec![
                ConfigEntry::plain("base_url", empty_to_unset(&config.base_url)),
                ConfigEntry::plain(
                    "auth_token",
                    if config.auth_token.trim().is_empty() {
                        "missing".to_string()
                    } else {
                        "present".to_string()
                    },
                ),
            ],
        }))
    }

    fn tool_detail_config_path(&self) -> Option<std::path::PathBuf> {
        Some(config::claude_settings_path())
    }

    fn ensure_editable_config_exists(&self) -> anyhow::Result<Option<std::path::PathBuf>> {
        Ok(Some(config::ensure_claude_settings_exists()?))
    }
}

fn maybe_trimmed(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn empty_to_unset(value: &str) -> String {
    maybe_trimmed(value).unwrap_or_else(|| "unset".to_string())
}

fn is_configured() -> bool {
    let cfg = config::load_claude_config();
    config::claude_settings_path().exists()
        || !cfg.base_url.trim().is_empty()
        || !cfg.auth_token.trim().is_empty()
}
