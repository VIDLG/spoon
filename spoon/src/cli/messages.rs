use super::response::{CliEntry, CliKind, CliResponse};
use crate::view::{ConfigModel, ConfigScopeModel};

pub fn no_scoop_packages_selected() -> CliResponse {
    CliResponse::line(CliKind::Warning, "No Scoop packages selected.")
}

pub fn missing_scoop_root() -> CliResponse {
    CliResponse::new(vec![
        CliEntry::Line {
            kind: CliKind::Error,
            text: "Scoop package management requires a configured root.".to_string(),
        },
        CliEntry::Line {
            kind: CliKind::Plain,
            text: "Configure a root before managing Spoon-owned Scoop state.".to_string(),
        },
    ])
}

pub fn missing_msvc_root() -> CliResponse {
    CliResponse::new(vec![
        CliEntry::Line {
            kind: CliKind::Error,
            text: "MSVC Toolchain requires a configured root.".to_string(),
        },
        CliEntry::Line {
            kind: CliKind::Plain,
            text: "Set root in spoon config before managing the toolchain.".to_string(),
        },
    ])
}

pub fn no_installed_msvc_runtimes() -> CliResponse {
    CliResponse::new(vec![
        CliEntry::Line {
            kind: CliKind::Warning,
            text: "No installed MSVC runtimes were found.".to_string(),
        },
        CliEntry::Line {
            kind: CliKind::Plain,
            text: "Install a managed or official runtime first, or pass a runtime positionally such as `spoon msvc validate managed`.".to_string(),
        },
    ])
}

pub fn config_view(model: &ConfigModel) -> CliResponse {
    let mut entries = vec![
        CliResponse::section("config"),
        CliResponse::subsection(1, "Root"),
        CliEntry::KeyValue {
            key: "    config_file".to_string(),
            value: model.config_file.clone(),
        },
        CliEntry::KeyValue {
            key: "    path".to_string(),
            value: model.root_path.clone(),
        },
        CliResponse::subsection(1, "Runtime"),
        CliEntry::KeyValue {
            key: "    proxy".to_string(),
            value: model
                .runtime_proxy
                .clone()
                .unwrap_or_else(|| "unset".to_string()),
        },
        CliEntry::KeyValue {
            key: "    editor".to_string(),
            value: model
                .runtime_editor
                .clone()
                .unwrap_or_else(|| "unset".to_string()),
        },
        CliEntry::KeyValue {
            key: "    msvc_arch".to_string(),
            value: model.runtime_msvc_arch.clone(),
        },
        CliResponse::subsection(1, "Derived"),
        CliEntry::KeyValue {
            key: "    scoop_root".to_string(),
            value: model.derived_scoop_root.clone(),
        },
        CliEntry::KeyValue {
            key: "    managed_msvc_root".to_string(),
            value: model.derived_managed_msvc_root.clone(),
        },
        CliEntry::KeyValue {
            key: "    managed_msvc_toolchain".to_string(),
            value: model.derived_managed_msvc_toolchain.clone(),
        },
        CliEntry::KeyValue {
            key: "    official_msvc_root".to_string(),
            value: model.derived_official_msvc_root.clone(),
        },
        CliEntry::KeyValue {
            key: "    msvc_target_arch".to_string(),
            value: model.derived_msvc_target_arch.clone(),
        },
        CliResponse::subsection(1, "Packages"),
    ];
    for package in &model.packages {
        entries.push(CliResponse::subsection(2, package.display_name));
        entries.extend(package.entries.iter().map(|entry| CliEntry::KeyValue {
            key: format!("      {}", entry.key),
            value: entry.value.display_value(),
        }));
    }
    CliResponse::new(entries)
}

pub fn config_root_unset() -> CliResponse {
    CliResponse::new(vec![
        CliResponse::section("config"),
        CliEntry::KeyValue {
            key: "root".to_string(),
            value: "unset".to_string(),
        },
    ])
}

pub fn config_scope_view(model: &ConfigScopeModel) -> CliResponse {
    let mut entries = vec![
        CliResponse::section(model.scope),
        CliResponse::subsection(1, "Desired"),
        CliResponse::subsection(1, &model.detected_label),
        CliResponse::subsection(1, "Config files"),
    ];
    entries.splice(
        2..2,
        model
            .desired_entries
            .iter()
            .map(|entry| CliEntry::KeyValue {
                key: format!("    {}", entry.key),
                value: entry.value.display_value(),
            }),
    );
    let detected_insert = 3 + model.desired_entries.len();
    entries.splice(
        detected_insert..detected_insert,
        model
            .detected_entries
            .iter()
            .map(|entry| CliEntry::KeyValue {
                key: format!("    {}", entry.key),
                value: entry.value.display_value(),
            }),
    );
    entries.extend(model.config_files.iter().map(|path| CliEntry::Line {
        kind: CliKind::Plain,
        text: format!("    - {path}"),
    }));
    if !model.conflicts.is_empty() {
        entries.push(CliResponse::subsection(1, "Conflicts"));
        entries.extend(model.conflicts.iter().map(|conflict| CliEntry::Line {
            kind: CliKind::Warning,
            text: format!("    - {conflict}"),
        }));
    }
    CliResponse::new(entries)
}

pub fn config_updated(key: &str, value: &str) -> CliResponse {
    CliResponse::new(vec![
        CliEntry::Line {
            kind: CliKind::Info,
            text: format!("Updated config '{key}'."),
        },
        CliEntry::KeyValue {
            key: key.to_string(),
            value: value.to_string(),
        },
    ])
}

pub fn config_imported(key: &str, value: &str) -> CliResponse {
    CliResponse::new(vec![
        CliEntry::Line {
            kind: CliKind::Info,
            text: format!("Imported native config into '{key}'."),
        },
        CliEntry::KeyValue {
            key: key.to_string(),
            value: value.to_string(),
        },
    ])
}

pub fn config_import_skipped(scope: &str, reason: &str) -> CliResponse {
    CliResponse::new(vec![
        CliEntry::Line {
            kind: CliKind::Warning,
            text: format!("No importable native config was applied for '{scope}'."),
        },
        CliEntry::Line {
            kind: CliKind::Plain,
            text: reason.to_string(),
        },
    ])
}

pub fn unknown_config_key(scope: &str, key: &str) -> CliResponse {
    let supported_keys = crate::packages::supported_config_keys(scope);
    let supported = if supported_keys.is_empty() {
        "No supported keys.".to_string()
    } else {
        format!("Supported keys: {}", supported_keys.join(", "))
    };
    CliResponse::new(vec![
        CliEntry::Line {
            kind: CliKind::Error,
            text: format!("Unknown config key for {scope}: {key}"),
        },
        CliEntry::Line {
            kind: CliKind::Plain,
            text: supported,
        },
    ])
}

pub fn invalid_config_value(key: &str, value: &str, expected: &str) -> CliResponse {
    CliResponse::new(vec![
        CliEntry::Line {
            kind: CliKind::Error,
            text: format!("Invalid value for config '{key}': {value}"),
        },
        CliEntry::Line {
            kind: CliKind::Plain,
            text: format!("Expected {expected}."),
        },
    ])
}

pub fn missing_config_value(scope: &str, key: Option<&str>) -> CliResponse {
    let text = match key {
        Some(key) => format!("Missing value for config '{scope}.{key}'."),
        None => format!("Missing config key for '{scope}'."),
    };
    CliResponse::new(vec![CliEntry::Line {
        kind: CliKind::Error,
        text,
    }])
}

pub fn config_root_updated(root: &str) -> CliResponse {
    CliResponse::new(vec![
        CliEntry::Line {
            kind: CliKind::Info,
            text: "Updated config 'root'.".to_string(),
        },
        CliEntry::KeyValue {
            key: "root".to_string(),
            value: root.to_string(),
        },
    ])
}
