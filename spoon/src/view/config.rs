use serde::Serialize;
use serde_json::Value;

use crate::config;
use crate::packages::{self, ConfigEntry, empty_to_none};
use spoon_core::RuntimeLayout;

#[derive(Debug, Clone, Serialize)]
pub struct ConfigModel {
    pub config_file: String,
    pub root_path: String,
    pub runtime_proxy: Option<String>,
    pub runtime_editor: Option<String>,
    pub runtime_msvc_arch: String,
    pub derived_scoop_root: String,
    pub derived_managed_msvc_root: String,
    pub derived_managed_msvc_toolchain: String,
    pub derived_official_msvc_root: String,
    pub derived_msvc_target_arch: String,
    pub packages: Vec<ConfigPackageSummary>,
}

impl ConfigModel {
    fn from_global(global: &config::GlobalConfig) -> Self {
        let root = std::path::Path::new(&global.root);
        let layout = RuntimeLayout::from_root(root);
        Self {
            config_file: config::global_config_path().display().to_string(),
            root_path: global.root.clone(),
            runtime_proxy: empty_to_none(&global.proxy),
            runtime_editor: empty_to_none(&global.editor),
            runtime_msvc_arch: global.msvc_arch.clone(),
            derived_scoop_root: layout.scoop.root.display().to_string(),
            derived_managed_msvc_root: layout.msvc.managed.root.display().to_string(),
            derived_managed_msvc_toolchain: layout.msvc.managed.toolchain_root.display().to_string(),
            derived_official_msvc_root: layout.msvc.official.root.display().to_string(),
            derived_msvc_target_arch: config::msvc_arch_from_config(global),
            packages: packages::config_target_descriptors()
                .into_iter()
                .map(ConfigPackageSummary::from_descriptor)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigScopeModel {
    pub scope: &'static str,
    pub desired: Value,
    pub detected_native_config: Value,
    pub desired_entries: Vec<ConfigEntry>,
    pub detected_label: String,
    pub detected_entries: Vec<ConfigEntry>,
    pub config_files: Vec<String>,
    pub conflicts: Vec<String>,
}

impl ConfigScopeModel {
    fn from_scope_data(data: packages::ConfigScopeDetails) -> Self {
        Self {
            scope: data.scope,
            desired: data.desired,
            detected_native_config: data.detected_native_config,
            desired_entries: data.desired_entries,
            detected_label: data.detected_label,
            detected_entries: data.detected_entries,
            config_files: data.config_files,
            conflicts: data.conflicts,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigPackageSummary {
    pub key: &'static str,
    pub display_name: &'static str,
    pub entries: Vec<ConfigEntry>,
}

impl ConfigPackageSummary {
    fn from_descriptor(descriptor: packages::ConfigTargetDescriptor) -> Self {
        Self {
            key: descriptor.package_key,
            display_name: descriptor.display_name,
            entries: packages::config_summary_entries(descriptor.package_key),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigDetailSection {
    pub title: String,
    pub entries: Vec<ConfigEntry>,
}

impl ConfigDetailSection {
    fn new(title: impl Into<String>, entries: Vec<ConfigEntry>) -> Self {
        Self {
            title: title.into(),
            entries,
        }
    }

    fn files(config_files: Vec<String>, secondary_key: &'static str) -> Option<Self> {
        if config_files.is_empty() {
            return None;
        }
        Some(Self::new(
            "Files",
            config_files
                .into_iter()
                .enumerate()
                .map(|(index, path)| {
                    let key = if index == 0 { "config" } else { secondary_key };
                    ConfigEntry::plain(key, path)
                })
                .collect(),
        ))
    }

    fn conflicts(conflicts: Vec<String>) -> Option<Self> {
        if conflicts.is_empty() {
            return None;
        }
        Some(Self::new(
            "Conflicts",
            conflicts
                .into_iter()
                .map(|conflict| ConfigEntry::plain("conflict", conflict))
                .collect(),
        ))
    }
}

pub fn build_config_model() -> ConfigModel {
    let global = config::load_global_config();
    ConfigModel::from_global(&global)
}

pub fn build_package_config_scope_model(package_key: &str) -> Option<ConfigScopeModel> {
    packages::config_scope_details(package_key).map(ConfigScopeModel::from_scope_data)
}

pub fn build_package_config_detail_sections(package_key: &str) -> Vec<ConfigDetailSection> {
    match packages::config_details(package_key) {
        Some(packages::PackageConfigDetails::Scope(scope)) => {
            let mut sections = Vec::new();
            if !scope.desired_entries.is_empty() {
                sections.push(ConfigDetailSection::new("Desired", scope.desired_entries));
            }
            if !scope.detected_entries.is_empty() {
                sections.push(ConfigDetailSection::new(
                    scope.detected_label,
                    scope.detected_entries,
                ));
            }
            if let Some(files) = ConfigDetailSection::files(scope.config_files, "path") {
                sections.push(files);
            }
            if let Some(conflicts) = ConfigDetailSection::conflicts(scope.conflicts) {
                sections.push(conflicts);
            }
            sections
        }
        Some(packages::PackageConfigDetails::Summary(summary)) => {
            let mut sections = Vec::new();
            if !summary.detected_entries.is_empty() {
                sections.push(ConfigDetailSection::new(
                    "Detected",
                    summary.detected_entries,
                ));
            }
            if let Some(files) = ConfigDetailSection::files(summary.config_files, "auth") {
                sections.push(files);
            }
            sections
        }
        None => Vec::new(),
    }
}


