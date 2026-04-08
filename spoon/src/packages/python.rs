use std::path::Path;

use crate::config;
use crate::packages::tool::{
    Backend, DetailRuntimeKind, EntityKind, ProbePathPolicy, Tool, ToolCategory, ToolTag,
    UpdateStrategy,
};
use serde_json::json;

use super::{
    ConfigEntry, ConfigScopeDetails, ConfigTargetDescriptor, PackageConfigDetails,
    PackageConfigImportResult, PackageConfigMutation, PackageConfigReapply, PackageConfigSetResult,
    PackageIdentity, PackageSpec, SupplementalShimSpec, config_target_from_identity,
    empty_to_none, simple_supplemental_shims,
};

pub(super) struct PythonPackage;

const IDENTITY: PackageIdentity = PackageIdentity {
    key: "python",
    display_name: "Python",
    order: 40,
};

pub(crate) const TOOL: Tool = Tool {
    key: IDENTITY.key,
    display_name: IDENTITY.display_name,
    summary: "Python runtime and scripting language for automation, tooling, and package workflows.",
    homepage: "https://www.python.org",
    command: "python3",
    package_name: "python",
    dir_name: "python",
    category: ToolCategory::Helper,
    tag: ToolTag::Helper,
    kind: EntityKind::Tool,
    backend: Backend::Scoop,
    detail_runtime: DetailRuntimeKind::Standard,
    probe_path_policy: ProbePathPolicy::Default,
    owned_root_markers: None,
    depends_on: &[],
    version_args: &["--version"],
    update_strategy: UpdateStrategy::Backend,
};

impl PythonPackage {
    pub const fn new() -> Self {
        Self
    }
}

impl PackageSpec for PythonPackage {
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
            "Configure Python",
            false,
            false,
        ))
    }

    fn supported_config_keys(&self) -> &'static [&'static str] {
        &["pip_mirror", "command_profile"]
    }

    fn config_summary_entries(&self) -> Vec<ConfigEntry> {
        let policy = config::load_policy_config();
        vec![
            ConfigEntry::plain(
                "pip_mirror",
                empty_to_none(&policy.python.pip_mirror).unwrap_or_else(|| "unset".to_string()),
            ),
            ConfigEntry::command_profile(
                "command_profile",
                &policy.python.command_profile,
                command_profile_additions(&policy.python.command_profile),
            ),
        ]
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
            "pip_mirror" => {
                policy.python.pip_mirror = value.to_string();
                config::save_policy_config(&policy)?;
                Ok(PackageConfigSetResult::Changed(PackageConfigMutation {
                    changed_key: "python.pip_mirror".to_string(),
                    changed_value: value.to_string(),
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
                policy.python.command_profile = normalized.to_string();
                config::save_policy_config(&policy)?;
                Ok(PackageConfigSetResult::Changed(PackageConfigMutation {
                    changed_key: "python.command_profile".to_string(),
                    changed_value: normalized.to_string(),
                    reapply: PackageConfigReapply::ScoopCommandSurface,
                }))
            }
            _ => Ok(PackageConfigSetResult::UnknownKey),
        }
    }

    fn import_config(&self) -> anyhow::Result<Option<PackageConfigImportResult>> {
        let index_url = config::load_pip_index_url();
        if index_url.trim().is_empty() {
            return Ok(Some(PackageConfigImportResult::Skipped {
                reason: "No native pip index-url was detected.".to_string(),
            }));
        }
        let imported = imported_pip_mirror_policy_value(&index_url);
        let mut policy = config::load_policy_config();
        policy.python.pip_mirror = imported.clone();
        config::save_policy_config(&policy)?;
        Ok(Some(PackageConfigImportResult::Changed(
            PackageConfigMutation {
                changed_key: "python.pip_mirror".to_string(),
                changed_value: imported,
                reapply: PackageConfigReapply::ScoopIntegrations,
            },
        )))
    }
}

fn command_profile_additions(command_profile: &str) -> Vec<String> {
    if command_profile.eq_ignore_ascii_case("extended") {
        vec![
            "pip".to_string(),
            "py".to_string(),
            "pyw".to_string(),
            "pythonw".to_string(),
            "pip3".to_string(),
            "versioned pipX.Y shims".to_string(),
        ]
    } else {
        vec![
            "pip".to_string(),
            "py".to_string(),
            "pyw".to_string(),
            "pythonw".to_string(),
        ]
    }
}

fn desired_policy_entries(policy: &config::PolicyConfig) -> Vec<ConfigEntry> {
    let mut entries = Vec::new();
    if !policy.python.pip_mirror.trim().is_empty() {
        entries.push(ConfigEntry::plain("pip_mirror", &policy.python.pip_mirror));
    }
    entries.push(ConfigEntry::command_profile(
        "command_profile",
        &policy.python.command_profile,
        command_profile_additions(&policy.python.command_profile),
    ));
    entries
}

fn config_scope_details() -> ConfigScopeDetails {
    let policy = config::load_policy_config();
    let detected_index_url = config::load_pip_index_url();
    let mut conflicts = Vec::new();
    if !policy.python.pip_mirror.trim().is_empty() {
        let desired_index_url = crate::service::scoop::runtime::resolved_pip_mirror_url_for_display(
            &policy.python.pip_mirror,
        );
        if !detected_index_url.trim().is_empty() && detected_index_url.trim() != desired_index_url {
            conflicts.push(format!(
                "native pip index-url differs from Spoon policy ({detected_index_url} != {desired_index_url})"
            ));
        }
    }
    ConfigScopeDetails {
        scope: "python",
        desired: json!({
            "pip_mirror": empty_to_none(&policy.python.pip_mirror),
            "command_profile": {
                "value": policy.python.command_profile,
                "adds": command_profile_additions(&policy.python.command_profile),
            }
        }),
        detected_native_config: json!({
            "index_url": empty_to_none(&detected_index_url),
        }),
        desired_entries: vec![
            ConfigEntry::plain(
                "pip_mirror",
                empty_to_none(&policy.python.pip_mirror).unwrap_or_else(|| "unset".to_string()),
            ),
            ConfigEntry::command_profile(
                "command_profile",
                &policy.python.command_profile,
                command_profile_additions(&policy.python.command_profile),
            ),
        ],
        detected_label: "Detected native config".to_string(),
        detected_entries: vec![ConfigEntry::plain(
            "index_url",
            empty_to_none(&detected_index_url).unwrap_or_else(|| "unset".to_string()),
        )],
        config_files: vec![config::pip_config_path().display().to_string()],
        conflicts,
    }
}

fn supplemental_shims(current_root: &Path) -> Vec<SupplementalShimSpec> {
    let policy = config::load_policy_config();
    let mut shims = simple_supplemental_shims(
        current_root,
        &[
            ("pythonw", "pythonw.exe"),
            ("py", "py.exe"),
            ("pyw", "pyw.exe"),
            ("pip", "Scripts\\pip.exe"),
        ],
    );
    if policy
        .python
        .command_profile
        .eq_ignore_ascii_case("extended")
    {
        shims.extend(simple_supplemental_shims(
            current_root,
            &[("pip3", "Scripts\\pip3.exe")],
        ));
        let scripts_root = current_root.join("Scripts");
        if let Ok(entries) = std::fs::read_dir(&scripts_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                    continue;
                };
                let lower = file_name.to_ascii_lowercase();
                if !lower.starts_with("pip") || !lower.ends_with(".exe") {
                    continue;
                }
                let alias = file_name.trim_end_matches(".exe");
                if alias.eq_ignore_ascii_case("pip") || alias.eq_ignore_ascii_case("pip3") {
                    continue;
                }
                if !alias.strip_prefix("pip").is_some_and(|suffix| {
                    !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit() || c == '.')
                }) {
                    continue;
                }
                shims.push(SupplementalShimSpec {
                    alias: alias.to_string(),
                    relative_path: format!("Scripts\\{file_name}"),
                });
            }
        }
    }
    shims
}

pub fn resolved_pip_mirror_url_for_display(policy_value: &str) -> String {
    match policy_value.trim().to_ascii_lowercase().as_str() {
        "tuna" => "https://pypi.tuna.tsinghua.edu.cn/simple".to_string(),
        "ustc" => "https://pypi.mirrors.ustc.edu.cn/simple".to_string(),
        "sjtug" | "sjtu" => "https://mirror.sjtu.edu.cn/pypi/web/simple".to_string(),
        other if other.contains("://") => policy_value.trim().to_string(),
        _ => policy_value.trim().to_string(),
    }
}

pub fn imported_pip_mirror_policy_value(index_url: &str) -> String {
    match index_url.trim() {
        "https://pypi.tuna.tsinghua.edu.cn/simple" => "tuna".to_string(),
        "https://pypi.mirrors.ustc.edu.cn/simple" => "ustc".to_string(),
        "https://mirror.sjtu.edu.cn/pypi/web/simple" => "sjtug".to_string(),
        other => other.to_string(),
    }
}
