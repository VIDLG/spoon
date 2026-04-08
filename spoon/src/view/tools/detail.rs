use std::path::{Path, PathBuf};

use crate::actions::ToolAction;
use crate::formatting::format_bytes;
use crate::packages;
use crate::service::scoop::resolve_manifest;
use crate::status::{self, ToolOwnership, ToolStatus};
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct ToolDetailModel {
    pub title: String,
    pub rows: Vec<ToolDetailRow>,
}

impl ToolDetailModel {
    fn new(title: impl Into<String>, rows: Vec<ToolDetailRow>) -> Self {
        Self {
            title: title.into(),
            rows,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ToolDetailRow {
    Title {
        text: String,
    },
    ActionGroup {
        install: bool,
        update: bool,
        uninstall: bool,
    },
    Field {
        label: String,
        value: String,
        value_kind: ToolDetailValueKind,
    },
}

impl ToolDetailRow {
    fn title(text: impl Into<String>) -> Self {
        Self::Title { text: text.into() }
    }

    fn action_group(install: bool, update: bool, uninstall: bool) -> Self {
        Self::ActionGroup {
            install,
            update,
            uninstall,
        }
    }

    fn field(
        label: impl Into<String>,
        value: impl Into<String>,
        value_kind: ToolDetailValueKind,
    ) -> Self {
        Self::Field {
            label: label.into(),
            value: value.into(),
            value_kind,
        }
    }

    fn plain_text(&self) -> String {
        match self {
            Self::Title { text } => text.clone(),
            Self::ActionGroup {
                install,
                update,
                uninstall,
            } => format!(
                "available operations: i={} u={} x={}",
                on_off(*install),
                on_off(*update),
                on_off(*uninstall)
            ),
            Self::Field { label, value, .. } => format!("{label}: {value}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolDetailValueKind {
    Default,
    Package,
    Backend,
    Path,
    Version,
    State,
}

pub fn build_tool_detail_model(status: &ToolStatus, statuses: &[ToolStatus]) -> ToolDetailModel {
    let version = status
        .version
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let latest_version = status
        .latest_version
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let path = status
        .path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "not found in PATH".to_string());
    let kind_label = match status.tool.kind {
        crate::packages::tool::EntityKind::Tool => "tool",
        crate::packages::tool::EntityKind::Toolchain => "toolchain",
    };
    let behavior_note = status.tool.detail_behavior_note();
    let source = match status.tool.kind {
        crate::packages::tool::EntityKind::Toolchain => "managed toolchain object",
        crate::packages::tool::EntityKind::Tool => match status.tool.backend {
            crate::packages::tool::Backend::Scoop => "Scoop package (main bucket)",
            crate::packages::tool::Backend::Native => "native managed tool",
        },
    };
    let config_path = detail_config_path(status.tool);
    let managed_path = detail_managed_path(status);
    let cache_path = detail_cache_path(status, state_tool_root(statuses));
    let cache_size = detail_cache_size(status, state_tool_root(statuses));
    let install_root = detail_install_root(status, state_tool_root(statuses));
    let installed_size = detail_installed_size(status, state_tool_root(statuses));
    let payload_plan = detail_payload_plan(status, state_tool_root(statuses));
    let cached_payloads = detail_cached_payloads(status, state_tool_root(statuses));
    let staged_msis = detail_staged_msis(status, state_tool_root(statuses));
    let extracted_msis = detail_extracted_msis(status, state_tool_root(statuses));
    let image_files = detail_install_image_files(status, state_tool_root(statuses));
    let package_label = detail_package_label(status.tool);
    let bucket_name = detail_bucket_name(status, state_tool_root(statuses));
    let package_link = detail_package_link(status, state_tool_root(statuses));
    let user_env_effect = detail_user_env_effect(status, statuses);
    let process_env_effect = detail_process_env_effect(status, statuses);
    let action_policy = status::action_policy(status, statuses);

    let mut rows = vec![
        ToolDetailRow::title(status.tool.display_name),
        ToolDetailRow::field("summary", status.tool.summary, ToolDetailValueKind::Default),
        ToolDetailRow::action_group(
            action_policy.allows(ToolAction::Install),
            action_policy.allows(ToolAction::Update),
            action_policy.allows(ToolAction::Uninstall),
        ),
        ToolDetailRow::field(
            "installed",
            installed_state_label(status),
            ToolDetailValueKind::State,
        ),
        ToolDetailRow::field("version", version.clone(), ToolDetailValueKind::Version),
        ToolDetailRow::field(
            "update available",
            yes_no(status.update_available),
            ToolDetailValueKind::State,
        ),
        ToolDetailRow::field("homepage", status.tool.homepage, ToolDetailValueKind::Path),
        ToolDetailRow::field("source", source, ToolDetailValueKind::Backend),
        ToolDetailRow::field("path", path, ToolDetailValueKind::Path),
        ToolDetailRow::field("key", status.tool.key, ToolDetailValueKind::Default),
        ToolDetailRow::field("command", status.tool.command, ToolDetailValueKind::Default),
        ToolDetailRow::field(
            "backend",
            status.tool.backend.label(),
            ToolDetailValueKind::Backend,
        ),
        ToolDetailRow::field("type", kind_label, ToolDetailValueKind::Backend),
        ToolDetailRow::field("note", behavior_note, ToolDetailValueKind::Default),
    ];

    if let Some(latest_version) = latest_version.filter(|latest| latest != &version) {
        rows.insert(
            5,
            ToolDetailRow::field("latest", latest_version, ToolDetailValueKind::Version),
        );
    }

    if config_path != "-" {
        rows.insert(
            7,
            ToolDetailRow::field("config path", config_path, ToolDetailValueKind::Path),
        );
    }

    let path_insert_index = rows
        .iter()
        .position(|row| matches!(row, ToolDetailRow::Field { label, .. } if label == "path"))
        .map(|index| index + 1)
        .unwrap_or(rows.len());
    let mut footprint_rows = Vec::new();
    if let Some(install_root) = install_root {
        footprint_rows.push(ToolDetailRow::field(
            "install root",
            install_root,
            ToolDetailValueKind::Path,
        ));
    }
    if let Some(installed_size) = installed_size {
        footprint_rows.push(ToolDetailRow::field(
            "installed size",
            installed_size,
            ToolDetailValueKind::Default,
        ));
    }
    if let Some(cache_path) = cache_path {
        footprint_rows.push(ToolDetailRow::field(
            "cache path",
            cache_path,
            ToolDetailValueKind::Path,
        ));
    }
    if let Some(cache_size) = cache_size {
        footprint_rows.push(ToolDetailRow::field(
            "cache size",
            cache_size,
            ToolDetailValueKind::Default,
        ));
    }
    if !footprint_rows.is_empty() {
        rows.splice(path_insert_index..path_insert_index, footprint_rows);
    }

    if managed_path != "-" {
        rows.push(ToolDetailRow::field(
            "managed path",
            managed_path,
            ToolDetailValueKind::Path,
        ));
    }
    if let Some(payload_plan) = payload_plan {
        rows.push(ToolDetailRow::field(
            "payload plan",
            payload_plan,
            ToolDetailValueKind::Default,
        ));
    }
    if let Some(cached_payloads) = cached_payloads {
        rows.push(ToolDetailRow::field(
            "cached payloads",
            cached_payloads,
            ToolDetailValueKind::Default,
        ));
    }
    if let Some(staged_msis) = staged_msis {
        rows.push(ToolDetailRow::field(
            "staged MSIs",
            staged_msis,
            ToolDetailValueKind::Default,
        ));
    }
    if let Some(extracted_msis) = extracted_msis {
        rows.push(ToolDetailRow::field(
            "expanded MSIs",
            extracted_msis,
            ToolDetailValueKind::Default,
        ));
    }
    if let Some(image_files) = image_files {
        rows.push(ToolDetailRow::field(
            "image files",
            image_files,
            ToolDetailValueKind::Default,
        ));
    }
    if let Some(package_label) = package_label {
        rows.push(ToolDetailRow::field(
            "package",
            package_label,
            ToolDetailValueKind::Package,
        ));
    }
    if let Some(bucket_name) = bucket_name {
        rows.push(ToolDetailRow::field(
            "bucket",
            bucket_name,
            ToolDetailValueKind::Package,
        ));
    }
    if let Some(package_link) = package_link {
        rows.push(ToolDetailRow::field(
            "package link",
            package_link,
            ToolDetailValueKind::Path,
        ));
    }
    if let Some(user_env_effect) = user_env_effect {
        rows.push(ToolDetailRow::field(
            "user environment",
            user_env_effect,
            ToolDetailValueKind::Default,
        ));
    }
    if let Some(process_env_effect) = process_env_effect {
        rows.push(ToolDetailRow::field(
            "process environment",
            process_env_effect,
            ToolDetailValueKind::Default,
        ));
    }

    ToolDetailModel::new(status.tool.display_name, rows)
}

pub fn tool_detail_plain_lines(model: &ToolDetailModel) -> Vec<String> {
    model.rows.iter().map(ToolDetailRow::plain_text).collect()
}

fn installed_state_label(status: &ToolStatus) -> &'static str {
    if status.broken {
        "broken"
    } else if status.is_usable() {
        "yes"
    } else if status.is_detected() {
        "detected but unusable"
    } else {
        "no"
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

fn state_tool_root(statuses: &[ToolStatus]) -> Option<&Path> {
    statuses
        .iter()
        .find(|status| status.tool.has_managed_toolchain_runtime())
        .and_then(|status| status.expected_dir.as_deref())
        .and_then(Path::parent)
}

fn configured_tool_root() -> Option<PathBuf> {
    crate::config::configured_tool_root()
}

fn inferred_tool_root_from_status(status: &ToolStatus, fallback: Option<&Path>) -> Option<PathBuf> {
    let path = status.path.as_ref()?;
    let configured_root = configured_tool_root();
    status
        .tool
        .infer_owned_root_from_path(fallback.or(configured_root.as_deref()), path)
}

fn detail_config_path(tool: &'static crate::packages::tool::Tool) -> String {
    packages::tool_detail_config_path(tool.key)
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn detail_managed_path(status: &ToolStatus) -> String {
    status
        .tool
        .detail_managed_path(
            configured_tool_root().as_deref(),
            status.expected_dir.as_deref(),
        )
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn detail_cache_path(status: &ToolStatus, tool_root: Option<&Path>) -> Option<String> {
    let owned_root = inferred_tool_root_from_status(status, tool_root);
    let root = owned_root.as_deref()?;
    status
        .tool
        .detail_cache_path(root)
        .map(|path| path.display().to_string())
}

fn detail_cache_size(status: &ToolStatus, tool_root: Option<&Path>) -> Option<String> {
    let cache_path = detail_cache_path(status, tool_root)?;
    let size = directory_size_bytes(Path::new(&cache_path))?;
    Some(format_bytes(size))
}

fn detail_install_root(status: &ToolStatus, tool_root: Option<&Path>) -> Option<String> {
    let owned_root = inferred_tool_root_from_status(status, tool_root);
    let root = owned_root.as_deref()?;
    status
        .tool
        .detail_install_root(
            root,
            status.expected_dir.as_deref(),
            status.ownership() == ToolOwnership::Managed,
        )
        .map(|path| path.display().to_string())
}

fn detail_installed_size(status: &ToolStatus, tool_root: Option<&Path>) -> Option<String> {
    let install_root = detail_install_root(status, tool_root)?;
    let size = directory_size_bytes(Path::new(&install_root))?;
    Some(format_bytes(size))
}

fn detail_payload_plan(status: &ToolStatus, tool_root: Option<&Path>) -> Option<String> {
    let owned_root = inferred_tool_root_from_status(status, tool_root);
    let root = owned_root.as_deref()?;
    status.tool.detail_payload_plan(root)
}

fn detail_cached_payloads(status: &ToolStatus, tool_root: Option<&Path>) -> Option<String> {
    let owned_root = inferred_tool_root_from_status(status, tool_root);
    let root = owned_root.as_deref()?;
    status
        .tool
        .detail_cached_payloads(root)
        .map(|count| count.to_string())
}

fn detail_staged_msis(status: &ToolStatus, tool_root: Option<&Path>) -> Option<String> {
    let owned_root = inferred_tool_root_from_status(status, tool_root);
    let root = owned_root.as_deref()?;
    status
        .tool
        .detail_staged_msis(root)
        .map(|count| count.to_string())
}

fn detail_extracted_msis(status: &ToolStatus, tool_root: Option<&Path>) -> Option<String> {
    let owned_root = inferred_tool_root_from_status(status, tool_root);
    let root = owned_root.as_deref()?;
    status
        .tool
        .detail_extracted_msis(root)
        .map(|count| count.to_string())
}

fn detail_install_image_files(status: &ToolStatus, tool_root: Option<&Path>) -> Option<String> {
    let owned_root = inferred_tool_root_from_status(status, tool_root);
    let root = owned_root.as_deref()?;
    status
        .tool
        .detail_install_image_files(root)
        .map(|count| count.to_string())
}

fn directory_size_bytes(root: &Path) -> Option<u64> {
    if !root.exists() {
        return None;
    }
    let mut total = 0_u64;
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            total = total.saturating_add(entry.metadata().ok()?.len());
        }
    }
    Some(total)
}

fn detail_package_label(tool: &'static crate::packages::tool::Tool) -> Option<&'static str> {
    tool.detail_package_label()
}

fn detail_bucket_name(status: &ToolStatus, tool_root: Option<&Path>) -> Option<String> {
    if status.tool.backend != crate::packages::tool::Backend::Scoop
        || status.tool.kind != crate::packages::tool::EntityKind::Tool
    {
        return None;
    }
    let owned_root = inferred_tool_root_from_status(status, tool_root);
    let root = owned_root.as_deref()?;
    let package_name = status.tool.package_name;

    tokio::runtime::Handle::current()
        .block_on(resolve_manifest(root, package_name))
        .map(|resolved| resolved.bucket.name.clone())
        .or(Some("main".to_string()))
}

fn detail_package_link(status: &ToolStatus, tool_root: Option<&Path>) -> Option<String> {
    if status.tool.backend != crate::packages::tool::Backend::Scoop
        || status.tool.kind != crate::packages::tool::EntityKind::Tool
    {
        return None;
    }
    let owned_root = inferred_tool_root_from_status(status, tool_root);
    let root = owned_root.as_deref()?;
    let package_name = status.tool.package_name;

    let Some(resolved) =
        tokio::runtime::Handle::current().block_on(resolve_manifest(root, package_name))
    else {
        return Some(format!(
            "https://github.com/ScoopInstaller/Main/blob/master/bucket/{}.json",
            package_name
        ));
    };

    {
        let normalized = resolved
            .bucket
            .source
            .trim_end_matches(".git")
            .trim_end_matches('/');
        Some(format!(
            "{normalized}/blob/{}/bucket/{}.json",
            resolved.bucket.branch, package_name
        ))
    }
}

fn detail_user_env_effect(status: &ToolStatus, statuses: &[ToolStatus]) -> Option<String> {
    let owned_root = configured_tool_root();
    let root = state_tool_root(statuses).or(owned_root.as_deref())?;
    status.tool.detail_user_env_effect(root)
}

fn detail_process_env_effect(status: &ToolStatus, _statuses: &[ToolStatus]) -> Option<String> {
    status.tool.detail_process_env_effect().map(str::to_string)
}
