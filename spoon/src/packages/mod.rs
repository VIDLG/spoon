use std::path::{Path, PathBuf};

use crate::config;
use anyhow::Result;
use serde::Serialize;
use serde_json::{Value, json};

pub(crate) mod claude;
pub(crate) mod codex;
pub(crate) mod git;
pub(crate) mod msvc;
pub(crate) mod python;
pub(crate) mod simple;
pub mod tool;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct PackageIdentity {
    pub key: &'static str,
    pub display_name: &'static str,
    pub order: u16,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct PackageDescriptor {
    pub key: &'static str,
    pub display_name: &'static str,
    pub order: u16,
    pub has_command_profile: bool,
    pub has_config_scope: bool,
    pub has_supplemental_shims: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct ConfigTargetDescriptor {
    pub package_key: &'static str,
    pub display_name: &'static str,
    pub menu_label: &'static str,
    pub detail_title: &'static str,
    pub order: u16,
    pub editable: bool,
    pub editor_opens_parent_dir: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigBadgeTone {
    Ready,
    Missing,
    Muted,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct ConfigTargetBadge {
    pub label: &'static str,
    pub tone: ConfigBadgeTone,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConfigEntryValue {
    Plain {
        value: String,
    },
    CommandProfile {
        value: String,
        additions: Vec<String>,
    },
}

impl Default for ConfigEntryValue {
    fn default() -> Self {
        Self::Plain { value: String::new() }
    }
}

impl ConfigEntryValue {
    pub fn display_value(&self) -> String {
        match self {
            Self::Plain { value } => value.clone(),
            Self::CommandProfile { value, additions } => {
                format!("{value} ({})", additions.join(", "))
            }
        }
    }

    pub fn json_value(&self) -> Value {
        match self {
            Self::Plain { value } => Value::String(value.clone()),
            Self::CommandProfile { value, additions } => json!({
                "value": value,
                "adds": additions,
            }),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: ConfigEntryValue,
}

impl ConfigEntry {
    pub fn plain(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: ConfigEntryValue::Plain {
                value: value.into(),
            },
        }
    }

    pub fn command_profile(
        key: impl Into<String>,
        value: impl Into<String>,
        additions: Vec<String>,
    ) -> Self {
        Self {
            key: key.into(),
            value: ConfigEntryValue::CommandProfile {
                value: value.into(),
                additions,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigScopeDetails {
    pub scope: &'static str,
    pub desired: Value,
    pub detected_native_config: Value,
    pub desired_entries: Vec<ConfigEntry>,
    pub detected_label: String,
    pub detected_entries: Vec<ConfigEntry>,
    pub config_files: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssistantConfigSummary {
    pub scope: &'static str,
    pub config_files: Vec<String>,
    pub detected: Value,
    pub detected_entries: Vec<ConfigEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PackageConfigDetails {
    Scope(ConfigScopeDetails),
    Summary(AssistantConfigSummary),
}

#[derive(Debug, Clone, Serialize)]
pub struct SupplementalShimSpec {
    pub alias: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageConfigReapply {
    None,
    ScoopIntegrations,
    ScoopCommandSurface,
    ManagedMsvcCommandSurface,
}

#[derive(Debug, Clone)]
pub struct PackageConfigMutation {
    pub changed_key: String,
    pub changed_value: String,
    pub reapply: PackageConfigReapply,
}

#[derive(Debug, Clone)]
pub enum PackageConfigSetResult {
    Changed(PackageConfigMutation),
    UnknownKey,
    InvalidValue { expected: &'static str },
}

#[derive(Debug, Clone)]
pub enum PackageConfigImportResult {
    Changed(PackageConfigMutation),
    Skipped { reason: String },
}

pub(crate) trait PackageSpec: Sync {
    fn identity(&self) -> PackageIdentity;

    fn descriptor_flags(&self) -> (bool, bool, bool) {
        (false, false, false)
    }

    fn descriptor(&self) -> PackageDescriptor {
        let identity = self.identity();
        let (has_command_profile, has_config_scope, has_supplemental_shims) =
            self.descriptor_flags();
        PackageDescriptor {
            key: identity.key,
            display_name: identity.display_name,
            order: identity.order,
            has_command_profile,
            has_config_scope,
            has_supplemental_shims,
        }
    }

    fn key(&self) -> &'static str {
        self.identity().key
    }

    fn command_profile_additions(&self, _command_profile: &str) -> Option<Vec<String>> {
        None
    }

    fn command_profile_display(&self, command_profile: &str) -> Option<String> {
        self.command_profile_additions(command_profile)
            .map(|adds| format!("{command_profile} ({})", adds.join(", ")))
    }

    fn desired_policy_entries(&self, _policy: &config::PolicyConfig) -> Vec<ConfigEntry> {
        Vec::new()
    }

    fn config_target(&self) -> Option<ConfigTargetDescriptor> {
        None
    }

    fn supported_config_keys(&self) -> &'static [&'static str] {
        &[]
    }

    fn tool_detail_config_path(&self) -> Option<PathBuf> {
        None
    }

    fn config_summary_entries(&self) -> Vec<ConfigEntry> {
        Vec::new()
    }

    fn config_menu_summary_lines(&self) -> Vec<String> {
        let summary = self
            .config_summary_entries()
            .into_iter()
            .map(|entry| format!("{}: {}", entry.key, entry.value.display_value()))
            .collect::<Vec<_>>()
            .join(" | ");
        if summary.is_empty() {
            Vec::new()
        } else {
            vec![summary]
        }
    }

    fn config_target_badge(&self) -> Option<ConfigTargetBadge> {
        self.config_target().map(|descriptor| {
            if descriptor.editable {
                ConfigTargetBadge {
                    label: "missing",
                    tone: ConfigBadgeTone::Missing,
                }
            } else {
                ConfigTargetBadge {
                    label: "policy",
                    tone: ConfigBadgeTone::Muted,
                }
            }
        })
    }

    fn config_scope_details(&self) -> Option<ConfigScopeDetails> {
        None
    }

    fn config_details(&self) -> Option<PackageConfigDetails> {
        self.config_scope_details().map(PackageConfigDetails::Scope)
    }

    fn supplemental_shims(&self, _current_root: &Path) -> Vec<SupplementalShimSpec> {
        Vec::new()
    }

    fn set_config_value(&self, _key: &str, _value: &str) -> Result<PackageConfigSetResult> {
        Ok(PackageConfigSetResult::UnknownKey)
    }

    fn import_config(&self) -> Result<Option<PackageConfigImportResult>> {
        Ok(None)
    }

    fn ensure_editable_config_exists(&self) -> Result<Option<PathBuf>> {
        Ok(None)
    }
}

pub(crate) fn config_target_from_identity(
    identity: PackageIdentity,
    menu_label: &'static str,
    editable: bool,
    editor_opens_parent_dir: bool,
) -> ConfigTargetDescriptor {
    ConfigTargetDescriptor {
        package_key: identity.key,
        display_name: identity.display_name,
        menu_label,
        detail_title: identity.display_name,
        order: identity.order,
        editable,
        editor_opens_parent_dir,
    }
}

macro_rules! register_packages {
    ($($name:ident => $ty:ty),+ $(,)?) => {
        $(static $name: $ty = <$ty>::new();)+
        static PACKAGE_SPECS: &[&dyn PackageSpec] = &[$(&$name),+];
    };
}

register_packages! {
    MSVC => msvc::MsvcPackage,
    GIT => git::GitPackage,
    CLAUDE => claude::ClaudePackage,
    CODEX => codex::CodexPackage,
    PYTHON => python::PythonPackage,
}

pub(crate) const TOOLS: &[tool::Tool] = &[
    msvc::TOOL,
    claude::TOOL,
    codex::TOOL,
    git::TOOL,
    simple::GH_TOOL,
    simple::ZED_TOOL,
    simple::VSCODE_TOOL,
    simple::NANO_TOOL,
    simple::RG_TOOL,
    simple::FD_TOOL,
    simple::JQ_TOOL,
    simple::BAT_TOOL,
    simple::CMAKE_TOOL,
    simple::SEVEN_ZIP_TOOL,
    simple::DELTA_TOOL,
    simple::NINJA_TOOL,
    simple::SG_TOOL,
    simple::YQ_TOOL,
    simple::UV_TOOL,
    python::TOOL,
    simple::WHICH_TOOL,
];

pub(crate) fn find_package(package_key: &str) -> Option<&'static dyn PackageSpec> {
    PACKAGE_SPECS
        .iter()
        .copied()
        .find(|spec| spec.key().eq_ignore_ascii_case(package_key))
}

pub fn all_package_descriptors() -> Vec<PackageDescriptor> {
    let mut descriptors: Vec<_> = PACKAGE_SPECS.iter().map(|spec| spec.descriptor()).collect();
    descriptors.sort_by_key(|descriptor| (descriptor.order, descriptor.display_name));
    descriptors
}

pub fn config_target_descriptors() -> Vec<ConfigTargetDescriptor> {
    let mut descriptors: Vec<_> = PACKAGE_SPECS
        .iter()
        .filter_map(|spec| spec.config_target())
        .collect();
    descriptors.sort_by_key(|descriptor| (descriptor.order, descriptor.display_name));
    descriptors
}

pub fn config_target_descriptor(package_key: &str) -> Option<ConfigTargetDescriptor> {
    config_target_descriptors()
        .into_iter()
        .find(|descriptor| descriptor.package_key.eq_ignore_ascii_case(package_key))
}

pub(crate) fn ensure_editable_config_exists(package_key: &str) -> Result<Option<PathBuf>> {
    match find_package(package_key) {
        Some(spec) => spec.ensure_editable_config_exists(),
        None => Ok(None),
    }
}

pub fn command_profile_additions(package_key: &str, command_profile: &str) -> Option<Vec<String>> {
    find_package(package_key)?.command_profile_additions(command_profile)
}

pub fn command_profile_display(package_key: &str, command_profile: &str) -> Option<String> {
    find_package(package_key)?.command_profile_display(command_profile)
}

pub fn desired_policy_entries(
    package_key: &str,
    policy: &config::PolicyConfig,
) -> Vec<ConfigEntry> {
    find_package(package_key)
        .map(|spec| spec.desired_policy_entries(policy))
        .unwrap_or_default()
}

pub fn config_scope_details(package_key: &str) -> Option<ConfigScopeDetails> {
    match find_package(package_key)?.config_details()? {
        PackageConfigDetails::Scope(data) => Some(data),
        PackageConfigDetails::Summary(_) => None,
    }
}

pub fn config_summary_entries(package_key: &str) -> Vec<ConfigEntry> {
    find_package(package_key)
        .map(|spec| spec.config_summary_entries())
        .unwrap_or_default()
}

pub fn config_menu_summary_lines(package_key: &str) -> Vec<String> {
    find_package(package_key)
        .map(|spec| spec.config_menu_summary_lines())
        .unwrap_or_default()
}

pub fn config_target_badge(package_key: &str) -> Option<ConfigTargetBadge> {
    find_package(package_key)?.config_target_badge()
}

pub fn supported_config_keys(package_key: &str) -> &'static [&'static str] {
    find_package(package_key)
        .map(|spec| spec.supported_config_keys())
        .unwrap_or(&[])
}

pub fn tool_detail_config_path(package_key: &str) -> Option<PathBuf> {
    find_package(package_key)?.tool_detail_config_path()
}

pub fn config_details(package_key: &str) -> Option<PackageConfigDetails> {
    find_package(package_key)?.config_details()
}

pub fn supplemental_shims(package_key: &str, current_root: &Path) -> Vec<SupplementalShimSpec> {
    find_package(package_key)
        .map(|spec| spec.supplemental_shims(current_root))
        .unwrap_or_default()
}

pub(crate) fn empty_to_none(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn simple_supplemental_shims(
    current_root: &Path,
    candidates: &[(&str, &str)],
) -> Vec<SupplementalShimSpec> {
    candidates
        .iter()
        .filter_map(|(alias, relative_path)| {
            current_root
                .join(relative_path)
                .exists()
                .then(|| SupplementalShimSpec {
                    alias: (*alias).to_string(),
                    relative_path: (*relative_path).to_string(),
                })
        })
        .collect()
}

pub(crate) fn set_config_value(
    package_key: &str,
    key: &str,
    value: &str,
) -> Result<PackageConfigSetResult> {
    match find_package(package_key) {
        Some(spec) => spec.set_config_value(key, value),
        None => Ok(PackageConfigSetResult::UnknownKey),
    }
}

pub(crate) fn import_config(package_key: &str) -> Result<Option<PackageConfigImportResult>> {
    match find_package(package_key) {
        Some(spec) => spec.import_config(),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        all_package_descriptors, command_profile_display, config_target_descriptors, find_package,
    };

    #[test]
    fn registry_exposes_self_describing_package_capabilities() {
        let descriptors = all_package_descriptors();
        assert!(
            descriptors
                .iter()
                .any(|item| item.key == "python" && item.has_supplemental_shims)
        );
        assert!(
            descriptors
                .iter()
                .any(|item| item.key == "git" && item.has_command_profile)
        );
        assert!(
            descriptors
                .iter()
                .any(|item| item.key == "msvc" && item.has_config_scope)
        );
        assert!(
            descriptors
                .iter()
                .any(|item| item.key == "claude" && item.display_name == "Claude Code")
        );
        assert!(
            descriptors
                .iter()
                .any(|item| item.key == "codex" && item.display_name == "Codex")
        );
    }

    #[test]
    fn registry_dispatches_command_profile_display() {
        assert_eq!(
            command_profile_display("git", "default").as_deref(),
            Some("default (bash)")
        );
        assert!(find_package("python").is_some());
        assert!(find_package("missing").is_none());
    }

    #[test]
    fn registry_exposes_explicit_package_and_config_target_order() {
        let package_keys: Vec<_> = all_package_descriptors()
            .into_iter()
            .map(|descriptor| descriptor.key)
            .collect();
        assert_eq!(
            package_keys,
            vec!["msvc", "git", "claude", "codex", "python"]
        );

        let config_target_keys: Vec<_> = config_target_descriptors()
            .into_iter()
            .map(|descriptor| descriptor.package_key)
            .collect();
        assert_eq!(
            config_target_keys,
            vec!["msvc", "git", "claude", "codex", "python"]
        );
    }
}
