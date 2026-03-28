use std::path::{Path, PathBuf};

use spoon_backend::layout::RuntimeLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCategory {
    Core,
    Helper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToolTag {
    Core,
    Editor,
    Helper,
    Toolchain,
}

impl ToolTag {
    pub fn short_label(self) -> &'static str {
        match self {
            ToolTag::Core => "CORE",
            ToolTag::Editor => "EDITOR",
            ToolTag::Helper => "HELPER",
            ToolTag::Toolchain => "TOOLCHAIN",
        }
    }

    pub fn sort_rank(self) -> u8 {
        match self {
            ToolTag::Toolchain => 0,
            ToolTag::Core => 1,
            ToolTag::Editor => 2,
            ToolTag::Helper => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Tool,
    Toolchain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Scoop,
    Native,
}

impl Backend {
    pub fn short_tag(self) -> &'static str {
        match self {
            Backend::Scoop => "SC",
            Backend::Native => "NV",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Backend::Scoop => "scoop",
            Backend::Native => "native",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStrategy {
    Backend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailRuntimeKind {
    Standard,
    ManagedToolchain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbePathPolicy {
    Default,
    ConfiguredOnly,
}

#[derive(Debug, Clone, Copy)]
pub struct OwnedRootMarkers {
    pub domain_dir: &'static str,
    pub runtime_dir: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct Tool {
    pub key: &'static str,
    pub display_name: &'static str,
    pub summary: &'static str,
    pub homepage: &'static str,
    pub command: &'static str,
    pub package_name: &'static str,
    pub dir_name: &'static str,
    pub category: ToolCategory,
    pub tag: ToolTag,
    pub kind: EntityKind,
    pub backend: Backend,
    pub detail_runtime: DetailRuntimeKind,
    pub probe_path_policy: ProbePathPolicy,
    pub owned_root_markers: Option<OwnedRootMarkers>,
    pub depends_on: &'static [&'static str],
    pub version_args: &'static [&'static str],
    pub update_strategy: UpdateStrategy,
}

pub const TOOLS: &[Tool] = super::TOOLS;

impl Tool {
    pub fn has_managed_toolchain_runtime(self) -> bool {
        self.detail_runtime == DetailRuntimeKind::ManagedToolchain
    }

    pub fn managed_root_ancestor_name(self) -> Option<&'static str> {
        self.owned_root_markers.map(|markers| markers.domain_dir)
    }

    pub fn prefers_configured_probe_path_only(self) -> bool {
        self.probe_path_policy == ProbePathPolicy::ConfiguredOnly
    }

    pub fn infer_owned_root_from_path(
        self,
        configured_root: Option<&Path>,
        path: &Path,
    ) -> Option<PathBuf> {
        if let Some(root) = configured_root {
            return Some(root.to_path_buf());
        }
        if self.backend == Backend::Scoop {
            for ancestor in path.ancestors() {
                if ancestor
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("scoop"))
                {
                    return ancestor.parent().map(Path::to_path_buf);
                }
            }
        }
        if self.has_managed_toolchain_runtime() {
            for ancestor in path.ancestors() {
                if ancestor.file_name().is_some_and(|name| {
                    self.owned_root_markers.is_some_and(|markers| {
                        name.to_string_lossy()
                            .eq_ignore_ascii_case(markers.runtime_dir)
                    })
                }) && ancestor.parent().is_some_and(|parent| {
                    parent.file_name().is_some_and(|name| {
                        self.owned_root_markers.is_some_and(|markers| {
                            name.to_string_lossy()
                                .eq_ignore_ascii_case(markers.domain_dir)
                        })
                    })
                }) {
                    return ancestor
                        .parent()
                        .and_then(Path::parent)
                        .map(Path::to_path_buf);
                }
            }
        }
        None
    }

    pub fn detail_managed_path(
        self,
        configured_root: Option<&Path>,
        expected_dir: Option<&Path>,
    ) -> Option<PathBuf> {
        match self.kind {
            EntityKind::Toolchain => match self.detail_runtime {
                DetailRuntimeKind::ManagedToolchain => configured_root
                    .map(|r| RuntimeLayout::from_root(r).msvc.managed.root)
                    .or_else(|| expected_dir.map(Path::to_path_buf)),
                DetailRuntimeKind::Standard => expected_dir.map(Path::to_path_buf),
            },
            EntityKind::Tool => None,
        }
    }

    pub fn detail_cache_path(self, tool_root: &Path) -> Option<PathBuf> {
        match self.detail_runtime {
            DetailRuntimeKind::ManagedToolchain => {
                let layout = RuntimeLayout::from_root(tool_root);
                Some(layout.msvc.managed.cache_root)
            }
            DetailRuntimeKind::Standard => None,
        }
    }

    pub fn detail_install_root(
        self,
        tool_root: &Path,
        expected_dir: Option<&Path>,
        is_managed_owned: bool,
    ) -> Option<PathBuf> {
        let layout = RuntimeLayout::from_root(tool_root);
        match self.kind {
            EntityKind::Toolchain => match self.detail_runtime {
                DetailRuntimeKind::ManagedToolchain => Some(layout.msvc.managed.toolchain_root),
                DetailRuntimeKind::Standard => expected_dir.map(Path::to_path_buf),
            },
            EntityKind::Tool => match self.backend {
                Backend::Scoop => {
                    let install_root = layout
                        .scoop
                        .apps_root
                        .join(self.package_name)
                        .join("current");
                    if is_managed_owned || install_root.exists() {
                        Some(install_root)
                    } else {
                        None
                    }
                }
                Backend::Native => expected_dir.map(Path::to_path_buf),
            },
        }
    }

    pub fn detail_behavior_note(self) -> &'static str {
        match self.detail_runtime {
            DetailRuntimeKind::ManagedToolchain => {
                "Build toolchain managed directly by spoon under the configured root."
            }
            DetailRuntimeKind::Standard => match self.backend {
                Backend::Scoop => {
                    "Managed by Spoon-owned Scoop package flows under the configured root."
                }
                Backend::Native => "Managed as a regular tool entry.",
            },
        }
    }

    pub fn detail_process_env_effect(self) -> Option<&'static str> {
        match self.detail_runtime {
            DetailRuntimeKind::ManagedToolchain => Some(
                "spoon computes compiler PATH, INCLUDE, and LIB per action, and exposes stable wrapper entrypoints under the shared shims root for downstream use",
            ),
            DetailRuntimeKind::Standard => None,
        }
    }

    pub fn detail_user_env_effect(self, tool_root: &Path) -> Option<String> {
        match self.detail_runtime {
            DetailRuntimeKind::ManagedToolchain => {
                let layout = RuntimeLayout::from_root(tool_root);
                Some(format!(
                    "persists {} into user PATH",
                    layout.shims.display()
                ))
            }
            DetailRuntimeKind::Standard => None,
        }
    }

    pub fn detail_payload_plan(self, tool_root: &Path) -> Option<String> {
        match self.detail_runtime {
            DetailRuntimeKind::ManagedToolchain => {
                let layout = RuntimeLayout::from_root(tool_root);
                let stage = layout.msvc.managed.cache_root.join("stage");
                Some(format!(
                    "cached MSVC packages are staged under {} and expanded into an admin-image style toolchain layout",
                    stage.display()
                ))
            }
            DetailRuntimeKind::Standard => None,
        }
    }

    pub fn detail_cached_payloads(self, tool_root: &Path) -> Option<usize> {
        match self.detail_runtime {
            DetailRuntimeKind::ManagedToolchain => {
                let layout = RuntimeLayout::from_root(tool_root);
                let cache_dir = layout.msvc.managed.cache_root;
                std::fs::read_dir(cache_dir)
                    .ok()
                    .map(|entries| entries.flatten().count())
            }
            DetailRuntimeKind::Standard => None,
        }
    }

    pub fn detail_staged_msis(self, tool_root: &Path) -> Option<usize> {
        match self.detail_runtime {
            DetailRuntimeKind::ManagedToolchain => {
                let layout = RuntimeLayout::from_root(tool_root);
                let stage_dir = layout.msvc.managed.cache_root.join("stage");
                std::fs::read_dir(stage_dir)
                    .ok()
                    .map(|entries| entries.flatten().count())
            }
            DetailRuntimeKind::Standard => None,
        }
    }

    pub fn detail_extracted_msis(self, tool_root: &Path) -> Option<usize> {
        match self.detail_runtime {
            DetailRuntimeKind::ManagedToolchain => {
                let layout = RuntimeLayout::from_root(tool_root);
                let expanded_dir = layout.msvc.managed.cache_root.join("expanded");
                std::fs::read_dir(expanded_dir)
                    .ok()
                    .map(|entries| entries.flatten().count())
            }
            DetailRuntimeKind::Standard => None,
        }
    }

    pub fn detail_install_image_files(self, tool_root: &Path) -> Option<usize> {
        match self.detail_runtime {
            DetailRuntimeKind::ManagedToolchain => {
                let layout = RuntimeLayout::from_root(tool_root);
                let image_dir = layout.msvc.managed.cache_root.join("image");
                std::fs::read_dir(image_dir)
                    .ok()
                    .map(|entries| entries.flatten().count())
            }
            DetailRuntimeKind::Standard => None,
        }
    }

    pub fn detail_package_label(self) -> Option<&'static str> {
        if self.backend == Backend::Scoop && self.kind == EntityKind::Tool {
            Some(self.package_name)
        } else {
            None
        }
    }
}

pub fn find_tool(key: &str) -> Option<&'static Tool> {
    TOOLS.iter().find(|tool| tool.key.eq_ignore_ascii_case(key))
}

pub fn all_tools() -> Vec<&'static Tool> {
    TOOLS.iter().collect()
}

pub fn tool_sort_key(tool: &Tool) -> (u8, &'static str) {
    (tool.tag.sort_rank(), tool.display_name)
}

pub fn resolve_requested_tools(requested: &[String]) -> Vec<&'static Tool> {
    if requested.is_empty() {
        return Vec::new();
    }

    let mut selected = Vec::new();
    for item in requested {
        match item.trim().to_ascii_lowercase().as_str() {
            "all" => selected.extend(all_tools()),
            "core" => selected.extend(
                TOOLS
                    .iter()
                    .filter(|tool| tool.category == ToolCategory::Core),
            ),
            "helpers" | "helper" => selected.extend(
                TOOLS
                    .iter()
                    .filter(|tool| tool.category == ToolCategory::Helper),
            ),
            other => {
                if let Some(tool) = find_tool(other) {
                    selected.push(tool);
                }
            }
        }
    }

    selected.sort_by_key(|tool| tool_sort_key(tool));
    selected.dedup_by_key(|tool| tool.key);
    selected
}

pub fn expected_tool_dir(root: Option<&Path>, tool: &Tool) -> Option<PathBuf> {
    root.map(|base| match tool.backend {
        Backend::Scoop => base
            .join("scoop")
            .join("apps")
            .join(tool.package_name)
            .join("current"),
        Backend::Native => base.join(tool.dir_name),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_tool_returns_correct_tool() {
        let tool = find_tool("claude").unwrap();
        assert_eq!(tool.command, "claude");
        assert_eq!(tool.display_name, "Claude Code");
        assert_eq!(tool.backend, Backend::Scoop);
    }

    #[test]
    fn find_tool_case_insensitive() {
        assert!(find_tool("CLAUDE").is_some());
        assert!(find_tool("Claude").is_some());
    }

    #[test]
    fn find_tool_returns_none_for_unknown() {
        assert!(find_tool("nonexistent").is_none());
    }

    #[test]
    fn resolve_all_returns_all_tools() {
        let tools = resolve_requested_tools(&["all".to_string()]);
        assert_eq!(tools.len(), TOOLS.len());
    }

    #[test]
    fn resolve_core_returns_core_tools() {
        let tools = resolve_requested_tools(&["core".to_string()]);
        assert!(tools.iter().all(|t| t.category == ToolCategory::Core));
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn resolve_helpers_returns_helper_tools() {
        let tools = resolve_requested_tools(&["helpers".to_string()]);
        assert!(tools.iter().all(|t| t.category == ToolCategory::Helper));
        assert_eq!(tools.len(), 19);
    }

    #[test]
    fn resolve_deduplicates() {
        let tools = resolve_requested_tools(&["uv".to_string(), "uv".to_string()]);
        assert_eq!(tools.len(), 1);
    }

    #[test]
    fn tools_sort_by_tag_then_display_name() {
        let tools = resolve_requested_tools(&["all".to_string()]);
        let keys: Vec<_> = tools.into_iter().map(|tool| tool.key).collect();
        assert_eq!(
            &keys[..7],
            &["msvc", "claude", "codex", "nano", "vscode", "zed", "7zip"]
        );
        assert!(keys.ends_with(&["which", "yq"]));
    }

    #[test]
    fn find_tool_git() {
        let tool = find_tool("git").unwrap();
        assert_eq!(tool.command, "git");
        assert_eq!(tool.package_name, "git");
        assert_eq!(tool.backend, Backend::Scoop);
        assert_eq!(tool.category, ToolCategory::Helper);
    }

    #[test]
    fn find_tool_uv() {
        let tool = find_tool("uv").unwrap();
        assert_eq!(tool.command, "uv");
        assert_eq!(tool.package_name, "uv");
        assert_eq!(tool.backend, Backend::Scoop);
    }

    #[test]
    fn find_tool_zed() {
        let tool = find_tool("zed").unwrap();
        assert_eq!(tool.command, "zed");
        assert_eq!(tool.package_name, "zed");
        assert_eq!(tool.backend, Backend::Scoop);
    }

    #[test]
    fn find_tool_vscode() {
        let tool = find_tool("vscode").unwrap();
        assert_eq!(tool.command, "code");
        assert_eq!(tool.package_name, "vscode");
        assert_eq!(tool.backend, Backend::Scoop);
    }

    #[test]
    fn find_tool_nano() {
        let tool = find_tool("nano").unwrap();
        assert_eq!(tool.command, "nano");
        assert_eq!(tool.package_name, "nano");
        assert_eq!(tool.backend, Backend::Scoop);
    }

    #[test]
    fn find_tool_python() {
        let tool = find_tool("python").unwrap();
        assert_eq!(tool.command, "python3");
        assert_eq!(tool.package_name, "python");
        assert_eq!(tool.backend, Backend::Scoop);
    }

    #[test]
    fn find_tool_which() {
        let tool = find_tool("which").unwrap();
        assert_eq!(tool.command, "which");
        assert_eq!(tool.package_name, "which");
        assert_eq!(tool.backend, Backend::Scoop);
    }

    #[test]
    fn find_tool_cmake() {
        let tool = find_tool("cmake").unwrap();
        assert_eq!(tool.command, "cmake");
        assert_eq!(tool.package_name, "cmake");
        assert_eq!(tool.backend, Backend::Scoop);
    }

    #[test]
    fn find_tool_7zip() {
        let tool = find_tool("7zip").unwrap();
        assert_eq!(tool.command, "7z");
        assert_eq!(tool.package_name, "7zip");
        assert_eq!(tool.backend, Backend::Scoop);
        assert_eq!(tool.version_args, &["i"]);
    }

    #[test]
    fn find_tool_ninja() {
        let tool = find_tool("ninja").unwrap();
        assert_eq!(tool.command, "ninja");
        assert_eq!(tool.package_name, "ninja");
        assert_eq!(tool.backend, Backend::Scoop);
    }

    #[test]
    fn find_msvc_toolchain() {
        let tool = find_tool("msvc").unwrap();
        assert_eq!(tool.kind, EntityKind::Toolchain);
        assert_eq!(tool.backend, Backend::Native);
        assert_eq!(tool.tag, ToolTag::Toolchain);
    }

    #[test]
    fn infer_owned_root_from_scoop_path() {
        let tool = find_tool("git").unwrap();
        let path = Path::new("D:/spoon/scoop/apps/git/current/bin/git.exe");
        assert_eq!(
            tool.infer_owned_root_from_path(None, path),
            Some(PathBuf::from("D:/spoon"))
        );
    }

    #[test]
    fn infer_owned_root_from_managed_toolchain_path() {
        let tool = find_tool("msvc").unwrap();
        let path = Path::new(
            "D:/spoon/msvc/managed/toolchain/VC/Tools/MSVC/14.44.35207/bin/Hostx64/x64/cl.exe",
        );
        assert_eq!(
            tool.infer_owned_root_from_path(None, path),
            Some(PathBuf::from("D:/spoon"))
        );
    }

    #[test]
    fn resolve_empty_returns_empty() {
        let tools = resolve_requested_tools(&[]);
        assert!(tools.is_empty());
    }
}
