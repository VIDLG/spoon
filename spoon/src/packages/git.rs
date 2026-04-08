use std::path::Path;

use crate::config;
use crate::packages::tool::{
    Backend, DetailRuntimeKind, EntityKind, ProbePathPolicy, Tool, ToolCategory, ToolTag,
    UpdateStrategy,
};
use serde_json::json;

use super::{
    ConfigBadgeTone, ConfigEntry, ConfigScopeDetails, ConfigTargetBadge, ConfigTargetDescriptor,
    PackageConfigDetails, PackageConfigImportResult, PackageConfigMutation, PackageConfigReapply,
    PackageConfigSetResult, PackageIdentity, PackageSpec, SupplementalShimSpec,
    config_target_from_identity, empty_to_none, simple_supplemental_shims,
};

pub(super) struct GitPackage;

const IDENTITY: PackageIdentity = PackageIdentity {
    key: "git",
    display_name: "Git",
    order: 10,
};

pub(crate) const TOOL: Tool = Tool {
    key: IDENTITY.key,
    display_name: IDENTITY.display_name,
    summary: "Distributed version control for repository history, cloning, and everyday source collaboration.",
    homepage: "https://git-scm.com",
    command: "git",
    package_name: "git",
    dir_name: "git",
    category: ToolCategory::Helper,
    tag: ToolTag::Helper,
    kind: EntityKind::Tool,
    backend: Backend::Scoop,
    detail_runtime: DetailRuntimeKind::Standard,
    probe_path_policy: ProbePathPolicy::ConfiguredOnly,
    owned_root_markers: None,
    depends_on: &[],
    version_args: &["--version"],
    update_strategy: UpdateStrategy::Backend,
};

impl GitPackage {
    pub const fn new() -> Self {
        Self
    }
}

impl PackageSpec for GitPackage {
    fn identity(&self) -> PackageIdentity {
        IDENTITY
    }

    fn descriptor_flags(&self) -> (bool, bool, bool) {
        (true, true, true)
    }

    fn command_profile_additions(&self, command_profile: &str) -> Option<Vec<String>> {
        Some(command_profile_additions(command_profile))
    }

    fn desired_policy_entries(&self, policy: &config::PolicyConfig) -> Vec<ConfigEntry> {
        desired_policy_entries(policy)
    }

    fn config_target(&self) -> Option<ConfigTargetDescriptor> {
        Some(config_target_from_identity(
            IDENTITY,
            "Configure Git",
            true,
            false,
        ))
    }

    fn supported_config_keys(&self) -> &'static [&'static str] {
        &["follow_spoon_proxy", "command_profile"]
    }

    fn config_summary_entries(&self) -> Vec<ConfigEntry> {
        let policy = config::load_policy_config();
        vec![
            ConfigEntry::plain(
                "follow_spoon_proxy",
                if policy.git.follow_spoon_proxy {
                    "true"
                } else {
                    "false"
                },
            ),
            ConfigEntry::command_profile(
                "command_profile",
                &policy.git.command_profile,
                command_profile_additions(&policy.git.command_profile),
            ),
        ]
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
        Some(PackageConfigDetails::Scope(config_scope_details()))
    }

    fn supplemental_shims(&self, current_root: &Path) -> Vec<SupplementalShimSpec> {
        supplemental_shims(current_root)
    }

    fn set_config_value(&self, key: &str, value: &str) -> anyhow::Result<PackageConfigSetResult> {
        let mut policy = config::load_policy_config();
        match key {
            "follow_spoon_proxy" => {
                let parsed = match value.trim().to_ascii_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => true,
                    "false" | "0" | "no" | "off" => false,
                    _ => {
                        return Ok(PackageConfigSetResult::InvalidValue {
                            expected: "a boolean such as true/false",
                        });
                    }
                };
                policy.git.follow_spoon_proxy = parsed;
                config::save_policy_config(&policy)?;
                Ok(PackageConfigSetResult::Changed(PackageConfigMutation {
                    changed_key: "git.follow_spoon_proxy".to_string(),
                    changed_value: if parsed { "true" } else { "false" }.to_string(),
                    reapply: PackageConfigReapply::ScoopIntegrations,
                }))
            }
            "command_profile" => {
                let normalized = match value.trim().to_ascii_lowercase().as_str() {
                    "default" => "default",
                    "extended" => "extended",
                    _ => {
                        return Ok(PackageConfigSetResult::InvalidValue {
                            expected: "`default` or `extended`",
                        });
                    }
                };
                policy.git.command_profile = normalized.to_string();
                config::save_policy_config(&policy)?;
                Ok(PackageConfigSetResult::Changed(PackageConfigMutation {
                    changed_key: "git.command_profile".to_string(),
                    changed_value: normalized.to_string(),
                    reapply: PackageConfigReapply::ScoopCommandSurface,
                }))
            }
            _ => Ok(PackageConfigSetResult::UnknownKey),
        }
    }

    fn import_config(&self) -> anyhow::Result<Option<PackageConfigImportResult>> {
        let git = config::load_git_config();
        if git.proxy.trim().is_empty() {
            let mut policy = config::load_policy_config();
            policy.git.follow_spoon_proxy = false;
            config::save_policy_config(&policy)?;
            return Ok(Some(PackageConfigImportResult::Changed(
                PackageConfigMutation {
                    changed_key: "git.follow_spoon_proxy".to_string(),
                    changed_value: "false".to_string(),
                    reapply: PackageConfigReapply::ScoopIntegrations,
                },
            )));
        }

        let mut global = config::load_global_config();
        if global.proxy.trim().is_empty() {
            global.proxy = git.proxy.trim().to_string();
            config::save_global_config(&global)?;
        }
        if git.proxy.trim() == global.proxy.trim() {
            let mut policy = config::load_policy_config();
            policy.git.follow_spoon_proxy = true;
            config::save_policy_config(&policy)?;
            return Ok(Some(PackageConfigImportResult::Changed(
                PackageConfigMutation {
                    changed_key: "git.follow_spoon_proxy".to_string(),
                    changed_value: "true".to_string(),
                    reapply: PackageConfigReapply::ScoopIntegrations,
                },
            )));
        }

        Ok(Some(PackageConfigImportResult::Skipped {
            reason: "Detected native Git proxy does not match Spoon proxy, so it cannot be represented by the current boolean follow_spoon_proxy policy.".to_string(),
        }))
    }

    fn ensure_editable_config_exists(&self) -> anyhow::Result<Option<std::path::PathBuf>> {
        Ok(Some(config::ensure_git_config_parent_exists()?))
    }
}

fn command_profile_additions(command_profile: &str) -> Vec<String> {
    if command_profile.eq_ignore_ascii_case("extended") {
        vec![
            "bash".to_string(),
            "git-gui".to_string(),
            "gitk".to_string(),
            "scalar".to_string(),
            "tig".to_string(),
        ]
    } else {
        vec!["bash".to_string()]
    }
}

fn desired_policy_entries(policy: &config::PolicyConfig) -> Vec<ConfigEntry> {
    vec![
        ConfigEntry::plain(
            "follow_spoon_proxy".to_string(),
            if policy.git.follow_spoon_proxy {
                "true".to_string()
            } else {
                "false".to_string()
            },
        ),
        ConfigEntry::command_profile(
            "command_profile",
            &policy.git.command_profile,
            command_profile_additions(&policy.git.command_profile),
        ),
    ]
}

fn config_scope_details() -> ConfigScopeDetails {
    let policy = config::load_policy_config();
    let git = config::load_git_config();
    let global = config::load_global_config();
    let mut conflicts = Vec::new();
    if policy.git.follow_spoon_proxy {
        if global.proxy.trim().is_empty() {
            conflicts
                .push("follow_spoon_proxy is enabled but Spoon global proxy is unset".to_string());
        } else if !git.proxy.trim().is_empty() && git.proxy.trim() != global.proxy.trim() {
            conflicts.push(format!(
                "native Git proxy differs from Spoon proxy ({} != {})",
                git.proxy.trim(),
                global.proxy.trim()
            ));
        }
    } else if !git.proxy.trim().is_empty() {
        conflicts.push(format!(
            "native Git proxy is set ({}) while Spoon policy does not manage one",
            git.proxy.trim()
        ));
    }
    ConfigScopeDetails {
        scope: "git",
        desired: json!({
            "follow_spoon_proxy": policy.git.follow_spoon_proxy,
            "command_profile": {
                "value": policy.git.command_profile,
                "adds": command_profile_additions(&policy.git.command_profile),
            }
        }),
        detected_native_config: json!({
            "proxy": empty_to_none(&git.proxy),
        }),
        desired_entries: vec![
            ConfigEntry::plain(
                "follow_spoon_proxy",
                if policy.git.follow_spoon_proxy {
                    "true"
                } else {
                    "false"
                },
            ),
            ConfigEntry::command_profile(
                "command_profile",
                &policy.git.command_profile,
                command_profile_additions(&policy.git.command_profile),
            ),
        ],
        detected_label: "Detected native config".to_string(),
        detected_entries: vec![ConfigEntry::plain(
            "proxy",
            empty_to_none(&git.proxy).unwrap_or_else(|| "unset".to_string()),
        )],
        config_files: vec![config::git_config_path().display().to_string()],
        conflicts,
    }
}

fn supplemental_shims(current_root: &Path) -> Vec<SupplementalShimSpec> {
    let policy = config::load_policy_config();
    let mut shims = simple_supplemental_shims(current_root, &[("bash", "bin\\bash.exe")]);
    if policy.git.command_profile.eq_ignore_ascii_case("extended") {
        shims.extend(simple_supplemental_shims(
            current_root,
            &[
                ("git-gui", "cmd\\git-gui.exe"),
                ("gitk", "cmd\\gitk.exe"),
                ("scalar", "cmd\\scalar.exe"),
                ("tig", "cmd\\tig.exe"),
            ],
        ));
    }
    shims
}

fn is_configured() -> bool {
    let cfg = config::load_git_config();
    config::git_config_path().exists()
        || !cfg.user_name.trim().is_empty()
        || !cfg.user_email.trim().is_empty()
        || !cfg.proxy.trim().is_empty()
}
