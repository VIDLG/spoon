use crate::packages::tool::{
    Backend, DetailRuntimeKind, EntityKind, ProbePathPolicy, Tool, ToolCategory, ToolTag,
    UpdateStrategy,
};

const VERSION_FLAG: &[&str] = &["--version"];
const SEVEN_ZIP_VERSION_ARGS: &[&str] = &["i"];

const fn scoop_tool(
    key: &'static str,
    display_name: &'static str,
    summary: &'static str,
    homepage: &'static str,
    command: &'static str,
    package_name: &'static str,
    dir_name: &'static str,
    tag: ToolTag,
    version_args: &'static [&'static str],
) -> Tool {
    Tool {
        key,
        display_name,
        summary,
        homepage,
        command,
        package_name,
        dir_name,
        category: ToolCategory::Helper,
        tag,
        kind: EntityKind::Tool,
        backend: Backend::Scoop,
        detail_runtime: DetailRuntimeKind::Standard,
        probe_path_policy: ProbePathPolicy::Default,
        owned_root_markers: None,
        depends_on: &[],
        version_args,
        update_strategy: UpdateStrategy::Backend,
    }
}

pub(crate) const GH_TOOL: Tool = scoop_tool(
    "gh",
    "GitHub CLI",
    "Command line interface for GitHub repositories, issues, PRs, and auth.",
    "https://cli.github.com",
    "gh",
    "gh",
    "gh",
    ToolTag::Helper,
    VERSION_FLAG,
);

pub(crate) const ZED_TOOL: Tool = scoop_tool(
    "zed",
    "Zed",
    "Fast multiplayer code editor with strong keyboard-driven workflows.",
    "https://zed.dev",
    "zed",
    "zed",
    "zed",
    ToolTag::Editor,
    VERSION_FLAG,
);

pub(crate) const VSCODE_TOOL: Tool = scoop_tool(
    "vscode",
    "VS Code",
    "Popular extensible code editor for source editing, debugging, and integrated terminal workflows.",
    "https://code.visualstudio.com",
    "code",
    "vscode",
    "vscode",
    ToolTag::Editor,
    VERSION_FLAG,
);

pub(crate) const NANO_TOOL: Tool = scoop_tool(
    "nano",
    "Nano",
    "Lightweight terminal editor for quick command-line text editing.",
    "https://www.nano-editor.org",
    "nano",
    "nano",
    "nano",
    ToolTag::Editor,
    VERSION_FLAG,
);

pub(crate) const RG_TOOL: Tool = scoop_tool(
    "rg",
    "ripgrep",
    "Fast recursive text search tool optimized for codebases.",
    "https://github.com/BurntSushi/ripgrep",
    "rg",
    "ripgrep",
    "ripgrep",
    ToolTag::Helper,
    VERSION_FLAG,
);

pub(crate) const FD_TOOL: Tool = scoop_tool(
    "fd",
    "fd",
    "Simple and fast file finder for directories and project trees.",
    "https://github.com/sharkdp/fd",
    "fd",
    "fd",
    "fd",
    ToolTag::Helper,
    VERSION_FLAG,
);

pub(crate) const JQ_TOOL: Tool = scoop_tool(
    "jq",
    "jq",
    "Command line JSON processor for filtering, transforming, and formatting data.",
    "https://jqlang.org",
    "jq",
    "jq",
    "jq",
    ToolTag::Helper,
    VERSION_FLAG,
);

pub(crate) const BAT_TOOL: Tool = scoop_tool(
    "bat",
    "bat",
    "Cat clone with syntax highlighting and Git-aware output.",
    "https://github.com/sharkdp/bat",
    "bat",
    "bat",
    "bat",
    ToolTag::Helper,
    VERSION_FLAG,
);

pub(crate) const CMAKE_TOOL: Tool = scoop_tool(
    "cmake",
    "CMake",
    "Cross-platform build system generator for native projects and toolchain validation workflows.",
    "https://cmake.org",
    "cmake",
    "cmake",
    "cmake",
    ToolTag::Helper,
    VERSION_FLAG,
);

pub(crate) const SEVEN_ZIP_TOOL: Tool = scoop_tool(
    "7zip",
    "7-Zip",
    "Archive extraction helper for Spoon-managed Scoop payloads such as 7z and MSI-based helper flows.",
    "https://www.7-zip.org/",
    "7z",
    "7zip",
    "7zip",
    ToolTag::Helper,
    SEVEN_ZIP_VERSION_ARGS,
);

pub(crate) const DELTA_TOOL: Tool = scoop_tool(
    "delta",
    "delta",
    "Syntax-highlighted pager for Git diffs and patch output.",
    "https://github.com/dandavison/delta",
    "delta",
    "delta",
    "delta",
    ToolTag::Helper,
    VERSION_FLAG,
);

pub(crate) const NINJA_TOOL: Tool = scoop_tool(
    "ninja",
    "Ninja",
    "Small, high-speed build executor commonly paired with CMake for native project builds.",
    "https://ninja-build.org",
    "ninja",
    "ninja",
    "ninja",
    ToolTag::Helper,
    VERSION_FLAG,
);

pub(crate) const SG_TOOL: Tool = scoop_tool(
    "sg",
    "ast-grep",
    "Structural code search and rewrite tool powered by AST matching.",
    "https://ast-grep.github.io",
    "sg",
    "ast-grep",
    "ast-grep",
    ToolTag::Helper,
    VERSION_FLAG,
);

pub(crate) const YQ_TOOL: Tool = scoop_tool(
    "yq",
    "yq",
    "Portable YAML, JSON, XML, and TOML processor with jq-like queries.",
    "https://github.com/mikefarah/yq",
    "yq",
    "yq",
    "yq",
    ToolTag::Helper,
    VERSION_FLAG,
);

pub(crate) const UV_TOOL: Tool = scoop_tool(
    "uv",
    "uv",
    "Fast Python package, virtualenv, and project manager.",
    "https://github.com/astral-sh/uv",
    "uv",
    "uv",
    "uv",
    ToolTag::Helper,
    VERSION_FLAG,
);

pub(crate) const WHICH_TOOL: Tool = scoop_tool(
    "which",
    "which",
    "Command locator for resolving executables on PATH and inspecting command origins.",
    "https://github.com/uutils/which",
    "which",
    "which",
    "which",
    ToolTag::Helper,
    VERSION_FLAG,
);
