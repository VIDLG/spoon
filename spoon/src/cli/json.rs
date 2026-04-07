use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Error;
use serde::Serialize;
use serde_json::{Value, json};

use crate::cli::response::CliResponse;
use crate::config;
use crate::service::{CommandResult, CommandStatus};
use crate::status;
use crate::view;

#[derive(Debug, Serialize)]
pub struct JsonEnvelope<T> {
    pub kind: &'static str,
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct CommandResultJson {
    pub title: String,
    pub status: &'static str,
    pub success: bool,
    pub streamed: bool,
    pub output: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorJson {
    pub message: String,
    pub chain: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ConfigScopeJson {
    pub scope: &'static str,
    pub desired: Value,
    pub detected_native_config: Value,
    pub config_files: Vec<String>,
    pub conflicts: Vec<String>,
}

pub fn command_result(result: &CommandResult) -> JsonEnvelope<CommandResultJson> {
    JsonEnvelope {
        kind: "command_result",
        data: CommandResultJson {
            title: result.title.clone(),
            status: result.status.as_str(),
            success: result.status == CommandStatus::Success,
            streamed: result.streamed,
            output: result.output.clone(),
        },
    }
}

pub fn cli_response(response: &CliResponse) -> JsonEnvelope<&CliResponse> {
    JsonEnvelope {
        kind: "cli_response",
        data: response,
    }
}

pub fn error(error: &Error) -> JsonEnvelope<ErrorJson> {
    JsonEnvelope {
        kind: "error",
        data: ErrorJson {
            message: error.to_string(),
            chain: error.chain().map(|item| item.to_string()).collect(),
        },
    }
}

pub fn config_view() -> JsonEnvelope<Value> {
    let model = view::build_config_model();
    let packages = model
        .packages
        .iter()
        .map(|package| {
            let entries = package
                .entries
                .iter()
                .map(|entry| (entry.key.clone(), entry.value.json_value()))
                .collect::<BTreeMap<_, _>>();
            (
                package.key.to_string(),
                serde_json::to_value(entries).expect("package entries should serialize"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    JsonEnvelope {
        kind: "config",
        data: json!({
            "config_file": model.config_file,
            "root": {
                "path": model.root_path,
            },
            "runtime": {
                "proxy": model.runtime_proxy,
                "editor": model.runtime_editor,
                "msvc_arch": model.runtime_msvc_arch,
            },
            "derived": {
                "scoop_root": model.derived_scoop_root,
                "managed_msvc_root": model.derived_managed_msvc_root,
                "managed_msvc_toolchain": model.derived_managed_msvc_toolchain,
                "official_msvc_root": model.derived_official_msvc_root,
                "msvc_target_arch": model.derived_msvc_target_arch,
            },
            "packages": packages,
        }),
    }
}

pub fn config_path() -> JsonEnvelope<Value> {
    JsonEnvelope {
        kind: "config_path",
        data: json!({
            "path": config::global_config_path().display().to_string()
        }),
    }
}

pub fn config_cat(path: &Path, content: &str) -> JsonEnvelope<Value> {
    JsonEnvelope {
        kind: "config_document",
        data: json!({
            "path": path.display().to_string(),
            "format": "toml",
            "content": content,
        }),
    }
}

pub fn config_scope_view(package_key: &str) -> Option<JsonEnvelope<ConfigScopeJson>> {
    let model = view::build_package_config_scope_model(package_key)?;
    Some(JsonEnvelope {
        kind: "config_scope",
        data: ConfigScopeJson {
            scope: model.scope,
            desired: model.desired,
            detected_native_config: model.detected_native_config,
            config_files: model.config_files,
            conflicts: model.conflicts,
        },
    })
}

pub fn status_view(install_root: Option<&Path>, include_update_info: bool) -> JsonEnvelope<Value> {
    let root = install_root
        .map(Path::to_path_buf)
        .or_else(config::configured_tool_root);
    let snapshot = root.as_deref().map(status::snapshot);
    JsonEnvelope {
        kind: "status",
        data: serde_json::to_value(status::build_status_details_with_snapshot(
            install_root,
            include_update_info,
            snapshot.as_ref(),
        ))
        .expect("status view should serialize"),
    }
}
