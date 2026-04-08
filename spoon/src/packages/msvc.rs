use crate::config;
use crate::packages::tool::{
    Backend, DetailRuntimeKind, EntityKind, OwnedRootMarkers, ProbePathPolicy, Tool, ToolCategory,
    ToolTag, UpdateStrategy,
};
use serde_json::json;
use spoon_core::RuntimeLayout;

use super::{
    ConfigEntry, ConfigScopeDetails, ConfigTargetDescriptor, PackageConfigDetails,
    PackageConfigMutation, PackageConfigReapply, PackageConfigSetResult, PackageIdentity,
    PackageSpec, config_target_from_identity,
};

pub(super) struct MsvcPackage;

const IDENTITY: PackageIdentity = PackageIdentity {
    key: "msvc",
    display_name: "MSVC",
    order: 0,
};

pub(crate) const TOOL: Tool = Tool {
    key: IDENTITY.key,
    display_name: "MSVC Toolchain",
    summary: "Managed MSVC build toolchain provisioned under the configured root.",
    homepage: "https://learn.microsoft.com/cpp/",
    command: "msvc",
    package_name: "msvc",
    dir_name: "msvc",
    category: ToolCategory::Helper,
    tag: ToolTag::Toolchain,
    kind: EntityKind::Toolchain,
    backend: Backend::Native,
    detail_runtime: DetailRuntimeKind::ManagedToolchain,
    probe_path_policy: ProbePathPolicy::ConfiguredOnly,
    owned_root_markers: Some(OwnedRootMarkers {
        domain_dir: "msvc",
        runtime_dir: "managed",
    }),
    depends_on: &[],
    version_args: &["--version"],
    update_strategy: UpdateStrategy::Backend,
};

impl MsvcPackage {
    pub const fn new() -> Self {
        Self
    }
}

impl PackageSpec for MsvcPackage {
    fn identity(&self) -> PackageIdentity {
        IDENTITY
    }

    fn descriptor_flags(&self) -> (bool, bool, bool) {
        (true, true, false)
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
            "Configure MSVC",
            false,
            false,
        ))
    }

    fn supported_config_keys(&self) -> &'static [&'static str] {
        &["command_profile"]
    }

    fn config_summary_entries(&self) -> Vec<ConfigEntry> {
        let policy = config::load_policy_config();
        vec![ConfigEntry::command_profile(
            "command_profile",
            &policy.msvc.command_profile,
            command_profile_additions(&policy.msvc.command_profile),
        )]
    }

    fn config_details(&self) -> Option<PackageConfigDetails> {
        Some(PackageConfigDetails::Scope(config_scope_details()))
    }

    fn set_config_value(&self, key: &str, value: &str) -> anyhow::Result<PackageConfigSetResult> {
        let mut policy = config::load_policy_config();
        match key {
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
                policy.msvc.command_profile = normalized.to_string();
                config::save_policy_config(&policy)?;
                Ok(PackageConfigSetResult::Changed(PackageConfigMutation {
                    changed_key: "msvc.command_profile".to_string(),
                    changed_value: normalized.to_string(),
                    reapply: PackageConfigReapply::ManagedMsvcCommandSurface,
                }))
            }
            _ => Ok(PackageConfigSetResult::UnknownKey),
        }
    }
}

fn command_profile_additions(command_profile: &str) -> Vec<String> {
    if command_profile.eq_ignore_ascii_case("extended") {
        vec![
            "spoon-cl".to_string(),
            "spoon-link".to_string(),
            "spoon-lib".to_string(),
            "spoon-rc".to_string(),
            "spoon-mt".to_string(),
            "spoon-nmake".to_string(),
            "spoon-dumpbin".to_string(),
        ]
    } else {
        vec![
            "spoon-cl".to_string(),
            "spoon-link".to_string(),
            "spoon-lib".to_string(),
        ]
    }
}

fn desired_policy_entries(policy: &config::PolicyConfig) -> Vec<ConfigEntry> {
    vec![ConfigEntry::command_profile(
        "command_profile",
        &policy.msvc.command_profile,
        command_profile_additions(&policy.msvc.command_profile),
    )]
}

fn config_scope_details() -> ConfigScopeDetails {
    let policy = config::load_policy_config();
    let root = config::configured_tool_root();
    let wrappers = root
        .as_deref()
        .map(detected_wrapper_names)
        .unwrap_or_default();
    let desired_wrappers = command_profile_additions(&policy.msvc.command_profile);
    let mut conflicts = Vec::new();
    let extras: Vec<String> = wrappers
        .iter()
        .filter(|name| !desired_wrappers.contains(name))
        .cloned()
        .collect();
    let missing: Vec<String> = desired_wrappers
        .iter()
        .filter(|name| !wrappers.contains(name))
        .cloned()
        .collect();
    if !extras.is_empty() {
        conflicts.push(format!(
            "managed wrapper set contains extra commands not covered by Spoon policy ({})",
            extras.join(", ")
        ));
    }
    if !missing.is_empty() {
        conflicts.push(format!(
            "managed wrapper set is missing policy-selected commands ({})",
            missing.join(", ")
        ));
    }

    ConfigScopeDetails {
        scope: "msvc",
        desired: json!({
            "command_profile": {
                "value": policy.msvc.command_profile,
                "adds": desired_wrappers,
            }
        }),
        detected_native_config: json!({
            "wrappers": wrappers,
        }),
        desired_entries: vec![ConfigEntry::command_profile(
            "command_profile",
            &policy.msvc.command_profile,
            desired_wrappers,
        )],
        detected_label: "Detected managed surface".to_string(),
        detected_entries: vec![ConfigEntry::plain(
            "wrappers",
            if wrappers.is_empty() {
                "none materialized".to_string()
            } else {
                wrappers.join(", ")
            },
        )],
        config_files: root
            .as_deref()
            .map(|root| vec![RuntimeLayout::from_root(root).shims.display().to_string()])
            .unwrap_or_default(),
        conflicts,
    }
}

fn detected_wrapper_names(root: &std::path::Path) -> Vec<String> {
    let shims = RuntimeLayout::from_root(root).shims;
    [
        "spoon-cl",
        "spoon-link",
        "spoon-lib",
        "spoon-rc",
        "spoon-mt",
        "spoon-nmake",
        "spoon-dumpbin",
    ]
    .into_iter()
    .filter(|name| shims.join(format!("{name}.cmd")).exists())
    .map(str::to_string)
    .collect()
}
