use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use crate::config;
use crate::tool::{self, EntityKind, Tool, ToolCategory};
use crossterm::style::Stylize;
use serde::Serialize;
use spoon_backend::status::BackendStatusSnapshot;
use walkdir::WalkDir;

mod discovery;
mod policy;
mod update;

pub use discovery::{collect_statuses, collect_statuses_fast, command_path, refresh_process_env_from_registry};

// Snapshot-accepting variants for backend-driven status collection.
// These are the primary entry points for code that has a BackendStatusSnapshot.
pub fn collect_statuses_with_snapshot(
    install_root: Option<&Path>,
    snapshot: Option<&BackendStatusSnapshot>,
) -> Vec<ToolStatus> {
    discovery::collect_statuses_with_snapshot(install_root, snapshot)
}

pub fn collect_statuses_fast_with_snapshot(
    install_root: Option<&Path>,
    snapshot: Option<&BackendStatusSnapshot>,
) -> Vec<ToolStatus> {
    discovery::collect_statuses_fast_with_snapshot(install_root, snapshot)
}
pub use policy::{
    ActionPolicy, ManagedReadiness, ToolOwnership, action_policy, tool_detected, tool_readiness,
};
pub use update::populate_update_info;

const STATUS_SECTION_HEADERS: &[&str] = &["Overview", "Runtime", "Tools"];
const STATUS_LABEL_REPLACEMENTS: &[(&str, ColorTag)] = &[
    ("[Scoop]", ColorTag::Info),
    ("[External]", ColorTag::Muted),
    ("[Managed]", ColorTag::Primary),
];

#[derive(Clone, Copy)]
enum ColorTag {
    Info,
    Muted,
    Primary,
}

#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub tool: &'static Tool,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    pub latest_version: Option<String>,
    pub installed_size_bytes: Option<u64>,
    pub update_available: bool,
    pub expected_dir: Option<PathBuf>,
    pub available: bool,
    pub broken: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusDetails {
    pub include_update_info: bool,
    pub overview: StatusSummary,
    pub runtime: RuntimeStatus,
    pub tools: Vec<ToolStatusDetails>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusSummary {
    pub primary_tools: Vec<String>,
    pub additional_tools: Vec<String>,
    pub toolchains: Vec<String>,
    pub tool_dependencies: Vec<String>,
    pub runtime_model: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeStatus {
    pub health: ToolchainHealth,
    pub updates: UpdateSummary,
    pub roots: Option<StatusRoots>,
    pub path_mismatches: Vec<PathMismatch>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolchainHealth {
    pub toolchains: Vec<ToolchainHealthEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolchainHealthEntry {
    pub key: String,
    pub display_name: String,
    pub readiness: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateSummary {
    pub mode: String,
    pub count: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusRoots {
    pub root: String,
    pub scoop: String,
    pub managed_msvc: String,
    pub managed_toolchain: String,
    pub official_msvc: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PathMismatch {
    pub tool: String,
    pub display_name: String,
    pub current: String,
    pub expected: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolStatusDetails {
    pub key: String,
    pub display_name: String,
    pub category: String,
    pub kind: String,
    pub backend: String,
    pub ownership: String,
    pub readiness: String,
    pub state: String,
    pub version: Option<String>,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub path: Option<String>,
    pub expected_dir: Option<String>,
}

impl ToolStatus {
    pub fn is_detected(&self) -> bool {
        self.path.is_some()
    }
    pub fn is_usable(&self) -> bool {
        self.available
    }
}

pub fn populate_installed_size_info(statuses: &mut [ToolStatus]) {
    for status in statuses.iter_mut() {
        status.installed_size_bytes = managed_install_size_bytes(status);
    }
}

fn managed_install_size_bytes(status: &ToolStatus) -> Option<u64> {
    if !matches!(status.ownership(), ToolOwnership::Managed) {
        return None;
    }
    let path = status.expected_dir.as_deref()?;
    directory_size_bytes(path)
}

fn directory_size_bytes(root: &Path) -> Option<u64> {
    if root.is_file() {
        return root.metadata().ok().map(|metadata| metadata.len());
    }
    if !root.exists() {
        return None;
    }

    let mut total = 0u64;
    for entry in WalkDir::new(root).follow_links(true) {
        let entry = entry.ok()?;
        if !entry.file_type().is_file() {
            continue;
        }
        total = total.checked_add(entry.metadata().ok()?.len())?;
    }
    Some(total)
}

fn tool_keys_for_category(category: ToolCategory) -> Vec<String> {
    tool::all_tools()
        .into_iter()
        .filter(|item| item.kind == EntityKind::Tool && item.category == category)
        .map(|item| item.key.to_string())
        .collect()
}

fn tool_keys_for_kind(kind: EntityKind) -> Vec<String> {
    tool::all_tools()
        .into_iter()
        .filter(|item| item.kind == kind)
        .map(|item| item.key.to_string())
        .collect()
}

fn tool_dependency_edges() -> Vec<String> {
    tool::all_tools()
        .into_iter()
        .flat_map(|tool| {
            tool.depends_on
                .iter()
                .map(move |dependency| format!("{} -> {}", tool.key, dependency))
        })
        .collect()
}

fn should_surface_update(status: &ToolStatus) -> bool {
    match status.tool.kind {
        EntityKind::Toolchain => status.update_available,
        EntityKind::Tool => {
            matches!(status.ownership(), ToolOwnership::Managed) && status.update_available
        }
    }
}

fn has_managed_scoop_state(
    status: &ToolStatus,
    _install_root: Option<&Path>,
    snapshot: Option<&BackendStatusSnapshot>,
) -> bool {
    if status.tool.backend != tool::Backend::Scoop {
        return false;
    }
    if !matches!(status.ownership(), ToolOwnership::Managed) {
        return false;
    }
    // D-07/D-08: check via backend snapshot, not app-side state file IO
    snapshot
        .is_some_and(|snap| snap.has_installed_package(status.tool.package_name))
}

pub fn status_lines(install_root: Option<&Path>, include_update_info: bool) -> Vec<String> {
    render_status_details(&build_status_details(install_root, include_update_info))
}

/// Build status details using a backend snapshot for roots and state checks.
pub fn build_status_details_with_snapshot(
    install_root: Option<&Path>,
    include_update_info: bool,
    snapshot: Option<&BackendStatusSnapshot>,
) -> StatusDetails {
    let mut statuses = if include_update_info {
        if let Some(snap) = snapshot {
            collect_statuses_with_snapshot(install_root, Some(snap))
        } else {
            collect_statuses(install_root)
        }
    } else if let Some(snap) = snapshot {
        collect_statuses_fast_with_snapshot(install_root, Some(snap))
    } else {
        collect_statuses_fast(install_root)
    };
    let snapshot_ref = snapshot;
    let toolchain_health = tool::all_tools()
        .into_iter()
        .filter(|tool| tool.kind == EntityKind::Toolchain)
        .map(|tool| {
            let readiness = statuses
                .iter()
                .find(|status| status.tool.key == tool.key)
                .map(ToolStatus::readiness)
                .unwrap_or_else(|| tool_readiness(tool.key, install_root));
            ToolchainHealthEntry {
                key: tool.key.to_string(),
                display_name: tool.display_name.to_string(),
                readiness: readiness.label().to_string(),
            }
        })
        .collect();
    let updates = if include_update_info {
        populate_update_info(&mut statuses, install_root);
        UpdateSummary {
            mode: "refreshed".to_string(),
            count: Some(
                statuses
                    .iter()
                    .filter(|status| should_surface_update(status))
                    .count(),
            ),
        }
    } else {
        UpdateSummary {
            mode: "local_only".to_string(),
            count: None,
        }
    };
    let roots = snapshot_ref.map(|snap| StatusRoots {
        root: snap.runtime_roots.root.clone(),
        scoop: snap.runtime_roots.scoop.clone(),
        managed_msvc: snap.runtime_roots.managed_msvc.clone(),
        managed_toolchain: snap.runtime_roots.managed_toolchain.clone(),
        official_msvc: snap.runtime_roots.official_msvc.clone(),
    });
    let path_mismatches = install_root
        .map(|root| status_path_mismatches(root, &statuses))
        .unwrap_or_default();
    let tools = statuses
        .iter()
        .map(|s| status_tool_entry_with_snapshot(s, snapshot_ref))
        .collect();
    StatusDetails {
        include_update_info,
        overview: StatusSummary {
            primary_tools: tool_keys_for_category(ToolCategory::Core),
            additional_tools: tool_keys_for_category(ToolCategory::Helper),
            toolchains: tool_keys_for_kind(EntityKind::Toolchain),
            tool_dependencies: tool_dependency_edges(),
            runtime_model: "Spoon-owned Scoop package flows plus managed/native toolchain flows."
                .to_string(),
        },
        runtime: RuntimeStatus {
            health: ToolchainHealth {
                toolchains: toolchain_health,
            },
            updates,
            roots,
            path_mismatches,
        },
        tools,
    }
}

pub fn build_status_details(
    install_root: Option<&Path>,
    include_update_info: bool,
) -> StatusDetails {
    build_status_details_with_snapshot(install_root, include_update_info, None)
}

fn render_status_details(view: &StatusDetails) -> Vec<String> {
    let mut lines = vec![
        "Overview".to_string(),
        format!(
            "  Primary tools: {}",
            view.overview.primary_tools.join(", ")
        ),
        format!(
            "  Additional tools: {}",
            view.overview.additional_tools.join(", ")
        ),
        format!("  Toolchains: {}", view.overview.toolchains.join(", ")),
        format!(
            "  Tool dependencies: {}",
            view.overview.tool_dependencies.join(", ")
        ),
        format!("  Runtime model: {}", view.overview.runtime_model),
        String::new(),
        "Runtime".to_string(),
    ];
    if view.runtime.health.toolchains.is_empty() {
        lines.push("  Health: no toolchains registered".to_string());
    } else {
        lines.push("  Health:".to_string());
        lines.extend(
            view.runtime
                .health
                .toolchains
                .iter()
                .map(|entry| format!("    {} {}", entry.key, entry.readiness)),
        );
    }
    if let Some(count) = view.runtime.updates.count {
        lines.push(format!("  Updates: {}", count));
    } else {
        lines.push("  Updates: use --refresh to check".to_string());
    }
    if let Some(roots) = &view.runtime.roots {
        lines.push("  Roots".to_string());
        lines.push(format!("    root: {}", roots.root));
        lines.push(format!("    scoop: {}", roots.scoop));
        lines.push(format!("    managed_msvc: {}", roots.managed_msvc));
        lines.push(format!(
            "    managed_toolchain: {}",
            roots.managed_toolchain
        ));
        lines.push(format!("    official_msvc: {}", roots.official_msvc));
        for mismatch in &view.runtime.path_mismatches {
            lines.push(format!(
                "  Path mismatch: {} -> current={} expected={}",
                mismatch.display_name, mismatch.current, mismatch.expected
            ));
        }
    }
    lines.push(String::new());
    lines.push("Tools".to_string());
    for tool in &view.tools {
        match (
            &tool.path,
            &tool.version,
            &tool.latest_version,
            tool.update_available,
            tool.state.as_str(),
        ) {
            (Some(path), Some(version), Some(latest), true, _) => {
                lines.push(format!(
                    "  {} [{}]: {} ({}) -> Latest ({}) [update available]",
                    tool.display_name,
                    status_label_from_strings(&tool.kind, &tool.backend, &tool.ownership),
                    state_verb_for_tool(tool),
                    version,
                    latest
                ));
                lines.push(format!("    path: {path}"));
            }
            (Some(path), Some(_version), _, _, "broken") => {
                lines.push(format!(
                    "  {} [{}]: Broken command entry [repair required]",
                    tool.display_name,
                    status_label_from_strings(&tool.kind, &tool.backend, &tool.ownership)
                ));
                lines.push(format!("    path: {path}"));
            }
            (Some(path), Some(version), _, _, _) => {
                lines.push(format!(
                    "  {} [{}]: {} ({})",
                    tool.display_name,
                    status_label_from_strings(&tool.kind, &tool.backend, &tool.ownership),
                    state_verb_for_tool(tool),
                    version
                ));
                lines.push(format!("    path: {path}"));
            }
            (Some(path), None, Some(latest), true, _) => {
                lines.push(format!(
                    "  {} [{}]: {} -> Latest ({}) [update available]",
                    tool.display_name,
                    status_label_from_strings(&tool.kind, &tool.backend, &tool.ownership),
                    state_verb_for_tool(tool),
                    latest
                ));
                lines.push(format!("    path: {path}"));
            }
            (Some(path), None, _, _, "broken") => {
                lines.push(format!(
                    "  {} [{}]: Broken command entry [repair required]",
                    tool.display_name,
                    status_label_from_strings(&tool.kind, &tool.backend, &tool.ownership)
                ));
                lines.push(format!("    path: {path}"));
            }
            (Some(path), None, _, _, state) => {
                let state_verb = match state {
                    "managed_state_missing" => "Detected [state missing]",
                    "installed" => "Installed",
                    _ => "Detected",
                };
                lines.push(format!(
                    "  {} [{}]: {}",
                    tool.display_name,
                    status_label_from_strings(&tool.kind, &tool.backend, &tool.ownership),
                    state_verb
                ));
                lines.push(format!("    path: {path}"));
            }
            (None, _, _, _, _) => {
                lines.push(format!(
                    "  {} [{}]: Not installed",
                    tool.display_name,
                    status_label_from_strings(&tool.kind, &tool.backend, &tool.ownership)
                ));
            }
        }
    }
    lines
}

pub fn print_status(install_root: Option<&Path>, include_update_info: bool) {
    let use_color = std::io::stdout().is_terminal();
    for line in render_status_details(&build_status_details(install_root, include_update_info)) {
        if use_color {
            println!("{}", colorize_status_line(&line));
        } else {
            println!("{line}");
        }
    }
}

fn colorize_status_line(line: &str) -> String {
    if STATUS_SECTION_HEADERS.contains(&line) {
        return line.bold().to_string();
    }
    if line.starts_with("  Path mismatch:") {
        return format!("{}", line.yellow());
    }
    let line = replace_colored_tokens(line, STATUS_LABEL_REPLACEMENTS);
    if line.contains("Broken command entry") || line.ends_with(": Not installed") {
        return line
            .replace(
                "Broken command entry [repair required]",
                &format!("{}", "Broken command entry [repair required]".red().bold()),
            )
            .replace(
                ": Not installed",
                &format!(": {}", "Not installed".dark_red().bold()),
            );
    }
    if line.contains("Detected [state missing]") {
        return line.replace(
            "Detected [state missing]",
            &format!("{}", "Detected [state missing]".yellow().bold()),
        );
    }
    if line.contains("[update available]") {
        return line
            .replace(
                "[update available]",
                &format!("{}", "[update available]".yellow().bold()),
            )
            .replace("Latest", &format!("{}", "Latest".yellow()));
    }
    if line.contains("Readiness:") {
        return line
            .replace(" ready", &format!(" {}", "ready".green().bold()))
            .replace(" broken", &format!(" {}", "broken".red().bold()))
            .replace(" missing", &format!(" {}", "missing".dark_red().bold()))
            .replace(" detected", &format!(" {}", "detected".yellow().bold()));
    }
    if line.contains("Updates available: 0") {
        return line.replace("0", &format!("{}", "0".green()));
    }
    if let Some((prefix, count)) = line.split_once("Updates available: ") {
        if count != "0" {
            return format!("{prefix}Updates available: {}", count.yellow().bold());
        }
    }
    if line.contains(": Installed") {
        return line.replace(": Installed", &format!(": {}", "Installed".green().bold()));
    }
    if line.contains(": Detected") {
        return line.replace(": Detected", &format!(": {}", "Detected".yellow().bold()));
    }
    line.to_string()
}

fn replace_colored_tokens(line: &str, tokens: &[(&str, ColorTag)]) -> String {
    let mut rendered = line.to_string();
    for (token, color) in tokens {
        rendered = rendered.replace(token, &colorize_status_token(token, *color));
    }
    rendered
}

fn status_tool_entry(status: &ToolStatus) -> ToolStatusDetails {
    status_tool_entry_with_snapshot(status, None)
}

fn status_tool_entry_with_snapshot(
    status: &ToolStatus,
    snapshot: Option<&BackendStatusSnapshot>,
) -> ToolStatusDetails {
    let state = match (&status.path, status.broken, status.available) {
        (None, _, _) => "missing".to_string(),
        (Some(_), true, _) => "broken".to_string(),
        (Some(_), false, true) => "installed".to_string(),
        (Some(_), false, false) => {
            if matches!(status.ownership(), ToolOwnership::Managed)
                && status.tool.backend == tool::Backend::Scoop
                && !has_managed_scoop_state(status, None, snapshot)
            {
                "managed_state_missing".to_string()
            } else {
                "detected".to_string()
            }
        }
    };
    ToolStatusDetails {
        key: status.tool.key.to_string(),
        display_name: status.tool.display_name.to_string(),
        category: match status.tool.category {
            ToolCategory::Core => "core".to_string(),
            ToolCategory::Helper => "helper".to_string(),
        },
        kind: match status.tool.kind {
            EntityKind::Tool => "tool".to_string(),
            EntityKind::Toolchain => "toolchain".to_string(),
        },
        backend: status.tool.backend.label().to_string(),
        ownership: status.ownership().label().to_string(),
        readiness: status.readiness().label().to_string(),
        state,
        version: status.version.clone(),
        latest_version: status.latest_version.clone(),
        update_available: status.update_available,
        path: status.path.as_ref().map(|path| path.display().to_string()),
        expected_dir: status
            .expected_dir
            .as_ref()
            .map(|path| path.display().to_string()),
    }
}

fn status_path_mismatches(root: &Path, statuses: &[ToolStatus]) -> Vec<PathMismatch> {
    let shims_root = spoon_backend::layout::RuntimeLayout::from_root(root).shims;
    statuses
        .iter()
        .filter_map(|status| {
            if !matches!(status.ownership(), ToolOwnership::Managed) {
                return None;
            }
            let (Some(current), Some(expected)) = (&status.path, &status.expected_dir) else {
                return None;
            };
            if current.starts_with(&shims_root) {
                return None;
            }
            let current_dir = current
                .parent()
                .and_then(|p| p.parent())
                .unwrap_or(current.as_path());
            if current_dir.starts_with(expected) {
                return None;
            }
            Some(PathMismatch {
                tool: status.tool.key.to_string(),
                display_name: status.tool.display_name.to_string(),
                current: current.display().to_string(),
                expected: expected.display().to_string(),
            })
        })
        .collect()
}

fn status_label_from_strings(kind: &str, backend: &str, ownership: &str) -> &'static str {
    match kind {
        "toolchain" => "Managed",
        _ => match ownership {
            "managed" => match backend {
                "scoop" => "Scoop",
                "native" => "Native",
                _ => "Managed",
            },
            "external" => "External",
            _ => match backend {
                "scoop" => "Scoop",
                "native" => "Native",
                _ => "Managed",
            },
        },
    }
}

fn state_verb_for_tool(tool: &ToolStatusDetails) -> &'static str {
    if tool.ownership == "external" {
        "Detected"
    } else {
        "Installed"
    }
}

fn colorize_status_token(token: &str, color: ColorTag) -> String {
    match color {
        ColorTag::Info => format!("{}", token.cyan().bold()),
        ColorTag::Muted => format!("{}", token.dark_grey().bold()),
        ColorTag::Primary => format!("{}", token.blue().bold()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        ToolOwnership, ToolStatus, colorize_status_line, should_surface_update,
        status_label_from_strings, tool_keys_for_category, tool_keys_for_kind,
    };
    use crate::tool::{EntityKind, ToolCategory};

    #[test]
    fn status_summary_lists_keep_kinds_separate() {
        assert_eq!(
            tool_keys_for_category(ToolCategory::Core).join(", "),
            "claude, codex"
        );
        assert_eq!(
            tool_keys_for_category(ToolCategory::Helper).join(", "),
            "git, gh, zed, vscode, nano, rg, fd, jq, bat, cmake, 7zip, delta, ninja, sg, yq, uv, python, which"
        );
        assert_eq!(tool_keys_for_kind(EntityKind::Toolchain).join(", "), "msvc");
    }

    #[test]
    fn colorize_status_line_marks_update_signal() {
        let rendered = colorize_status_line(
            "  Codex [Scoop]: Installed (0.1.0) -> Latest (0.2.0) [update available]",
        );
        assert!(rendered.contains("\u{1b}["));
        assert!(rendered.contains("[update available]"));
    }

    #[test]
    fn colorize_status_line_marks_broken_signal() {
        let rendered =
            colorize_status_line("  Python [Scoop]: Broken command entry [repair required]");
        assert!(rendered.contains("\u{1b}["));
        assert!(rendered.contains("Broken command entry [repair required]"));
    }

    #[test]
    fn colorize_status_line_marks_ownership_labels() {
        let rendered = colorize_status_line("  Claude Code [External]: Detected (2.1.76)");
        assert!(rendered.contains("\u{1b}["));
        assert!(rendered.contains("[External]"));
    }

    #[test]
    fn status_label_uses_external_ownership_for_scoop_tools() {
        crate::config::enable_test_mode();
        let temp_home = std::env::temp_dir().join(format!(
            "spoon-status-label-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&temp_home);
        std::fs::create_dir_all(&temp_home).unwrap();
        crate::config::set_home_override(temp_home.clone());
        let tool_root = temp_home.join("root");
        crate::config::save_global_config(&crate::config::GlobalConfig {
            editor: String::new(),
            proxy: String::new(),
            root: tool_root.display().to_string(),
            msvc_arch: crate::config::native_msvc_arch().to_string(),
        })
        .unwrap();

        let claude = crate::tool::find_tool("claude").unwrap();
        let external = ToolStatus {
            tool: claude,
            path: Some(PathBuf::from("C:/Users/vision/.local/bin/claude.exe")),
            version: Some("2.1.76".to_string()),
            latest_version: None,
            installed_size_bytes: None,
            update_available: false,
            expected_dir: None,
            available: true,
            broken: false,
        };

        assert_eq!(external.ownership(), ToolOwnership::External);
        assert_eq!(
            status_label_from_strings("tool", "scoop", external.ownership().label()),
            "External"
        );
        assert!(!should_surface_update(&external));
    }
}
