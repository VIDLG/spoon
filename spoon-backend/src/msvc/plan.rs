use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{BackendContext, CommandStatus};

use super::paths;

#[derive(Debug, Clone)]
pub(crate) struct MsvcRequest {
    pub root: PathBuf,
    pub proxy: String,
    pub command_profile: String,
    pub selected_target_arch: String,
    pub test_mode: bool,
}

impl MsvcRequest {
    pub(crate) fn for_tool_root(tool_root: &Path) -> Self {
        Self {
            root: tool_root.to_path_buf(),
            proxy: String::new(),
            command_profile: "default".to_string(),
            selected_target_arch: paths::native_msvc_arch().to_string(),
            test_mode: false,
        }
    }

    pub(crate) fn from_context<P>(context: &BackendContext<P>) -> Self {
        Self {
            root: context.root.clone(),
            proxy: context.proxy.clone().unwrap_or_default(),
            command_profile: context.msvc_command_profile.clone(),
            selected_target_arch: context.msvc_target_arch.clone(),
            test_mode: context.test_mode,
        }
    }

    pub(crate) fn normalized_target_arch(&self) -> String {
        let selected = self.selected_target_arch.trim();
        if selected.is_empty() {
            return paths::native_msvc_arch().to_string();
        }
        match selected.to_ascii_lowercase().as_str() {
            "auto" => paths::native_msvc_arch().to_string(),
            "x64" | "x86" | "arm64" | "arm" => selected.to_ascii_lowercase(),
            _ => paths::native_msvc_arch().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainFlags {
    pub compiler: PathBuf,
    pub linker: PathBuf,
    pub librarian: PathBuf,
    pub resource_compiler: Option<PathBuf>,
    pub manifest_tool: Option<PathBuf>,
    pub nmake: Option<PathBuf>,
    pub dumpbin: Option<PathBuf>,
    pub include_dirs: Vec<PathBuf>,
    pub lib_dirs: Vec<PathBuf>,
    pub path_dirs: Vec<PathBuf>,
}

impl ToolchainFlags {
    pub fn cflags(&self) -> Vec<String> {
        self.include_dirs
            .iter()
            .map(|path| format!("/I\"{}\"", path.display()))
            .collect()
    }

    pub fn libs(&self) -> Vec<String> {
        self.lib_dirs
            .iter()
            .map(|path| format!("/LIBPATH:\"{}\"", path.display()))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MsvcRuntimeKind {
    Managed,
    Official,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MsvcOperationKind {
    Install,
    Update,
    Uninstall,
    Validate,
}

#[derive(Debug, Clone, Serialize)]
pub struct MsvcOperationOutcome {
    pub kind: &'static str,
    pub runtime: MsvcRuntimeKind,
    pub operation: MsvcOperationKind,
    pub status: CommandStatus,
    pub title: String,
    pub output: Vec<String>,
    pub streamed: bool,
}

impl MsvcOperationOutcome {
    pub const fn is_success(&self) -> bool {
        self.status.is_success()
    }
}
