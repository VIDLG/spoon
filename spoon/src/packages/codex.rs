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

pub(super) struct CodexPackage;

const IDENTITY: PackageIdentity = PackageIdentity {
    key: "codex",
    display_name: "Codex",
    order: 30,
};

pub(crate) const TOOL: Tool = Tool {
    key: IDENTITY.key,
    display_name: IDENTITY.display_name,
    summary: "OpenAI coding assistant CLI for code generation and repository workflows.",
    homepage: "https://openai.com/chatgpt/codex/",
    command: "codex",
    package_name: "codex",
    dir_name: "codex",
    category: ToolCategory::Core,
    tag: ToolTag::Core,
    kind: EntityKind::Tool,
    backend: Backend::Scoop,
    detail_runtime: DetailRuntimeKind::Standard,
    probe_path_policy: ProbePathPolicy::Default,
    owned_root_markers: None,
    depends_on: &[],
    version_args: &["--version"],
    update_strategy: UpdateStrategy::Backend,
};

impl CodexPackage {
    pub const fn new() -> Self {
        Self
    }
}

impl PackageSpec for CodexPackage {
    fn identity(&self) -> PackageIdentity {
        IDENTITY
    }

    fn config_target(&self) -> Option<ConfigTargetDescriptor> {
        Some(config_target_from_identity(
            IDENTITY,
            "Configure Codex",
            true,
            true,
        ))
    }

    fn config_summary_entries(&self) -> Vec<ConfigEntry> {
        let config = config::load_codex_config("gpt-5.2-codex");
        vec![
            ConfigEntry::plain("model", empty_to_unset(&config.model)),
            ConfigEntry::plain("base_url", empty_to_unset(&config.base_url)),
            ConfigEntry::plain(
                "api_key",
                if config.api_key.trim().is_empty() {
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
        let config = config::load_codex_config("gpt-5.2-codex");
        Some(PackageConfigDetails::Summary(AssistantConfigSummary {
            scope: "codex",
            config_files: vec![
                config::codex_config_path().display().to_string(),
                config::codex_auth_path().display().to_string(),
            ],
            detected: json!({
                "base_url": maybe_trimmed(&config.base_url),
                "model": maybe_trimmed(&config.model),
                "api_key_present": !config.api_key.trim().is_empty(),
            }),
            detected_entries: vec![
                ConfigEntry::plain("model", empty_to_unset(&config.model)),
                ConfigEntry::plain("base_url", empty_to_unset(&config.base_url)),
                ConfigEntry::plain(
                    "api_key",
                    if config.api_key.trim().is_empty() {
                        "missing".to_string()
                    } else {
                        "present".to_string()
                    },
                ),
            ],
        }))
    }

    fn tool_detail_config_path(&self) -> Option<std::path::PathBuf> {
        Some(config::codex_config_path())
    }

    fn ensure_editable_config_exists(&self) -> anyhow::Result<Option<std::path::PathBuf>> {
        Ok(Some(config::ensure_codex_config_exists("gpt-5.2-codex")?))
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
    let cfg = config::load_codex_config("gpt-5.2-codex");
    config::codex_config_path().exists()
        || config::codex_auth_path().exists()
        || !cfg.base_url.trim().is_empty()
        || !cfg.api_key.trim().is_empty()
        || !cfg.model.trim().is_empty()
}
