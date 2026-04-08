//! MSVC domain types — single source of truth for all domain types.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Operation request
// ---------------------------------------------------------------------------

/// MSVC operation request configuration.
#[derive(Debug, Clone)]
pub struct MsvcRequest {
    pub root: PathBuf,
    pub proxy: String,
    pub command_profile: String,
    pub selected_target_arch: String,
    pub test_mode: bool,
}

impl MsvcRequest {
    pub fn for_tool_root(tool_root: &std::path::Path) -> Self {
        Self {
            root: tool_root.to_path_buf(),
            proxy: String::new(),
            command_profile: "default".to_string(),
            selected_target_arch: crate::paths::native_msvc_arch().to_string(),
            test_mode: false,
        }
    }

    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = proxy.into();
        self
    }

    pub fn command_profile(mut self, profile: impl Into<String>) -> Self {
        self.command_profile = profile.into();
        self
    }

    pub fn test_mode(mut self, enabled: bool) -> Self {
        self.test_mode = enabled;
        self
    }

    pub fn normalized_target_arch(&self) -> String {
        let selected = self.selected_target_arch.trim();
        if selected.is_empty() {
            return crate::paths::native_msvc_arch().to_string();
        }
        match selected.to_ascii_lowercase().as_str() {
            "auto" => crate::paths::native_msvc_arch().to_string(),
            "x64" | "x86" | "arm64" | "arm" => selected.to_ascii_lowercase(),
            _ => crate::paths::native_msvc_arch().to_string(),
        }
    }
}



// ---------------------------------------------------------------------------
// Toolchain flags
// ---------------------------------------------------------------------------

/// Toolchain compiler/linker flags and include/library directories.
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

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MsvcRuntimeKind {
    Managed,
    Official,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MsvcRuntimePreference {
    Auto,
    Managed,
    Official,
}

impl MsvcRuntimePreference {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Managed => "managed",
            Self::Official => "official",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MsvcOperationKind {
    Install,
    Update,
    Uninstall,
    Validate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MsvcLifecycleStage {
    Planned,
    Detecting,
    Resolving,
    Executing,
    Validating,
    StateCommitting,
    Completed,
}

impl MsvcLifecycleStage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Detecting => "detecting",
            Self::Resolving => "resolving",
            Self::Executing => "executing",
            Self::Validating => "validating",
            Self::StateCommitting => "state_committing",
            Self::Completed => "completed",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MsvcValidationStatus {
    Valid,
    Invalid,
    Unknown,
}

// ---------------------------------------------------------------------------
// Operation request struct
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct MsvcOperationRequest {
    pub operation: MsvcOperationKind,
    pub runtime_preference: MsvcRuntimePreference,
}

impl MsvcOperationRequest {
    pub const fn new(
        operation: MsvcOperationKind,
        runtime_preference: MsvcRuntimePreference,
    ) -> Self {
        Self {
            operation,
            runtime_preference,
        }
    }

    pub const fn install(runtime_preference: MsvcRuntimePreference) -> Self {
        Self::new(MsvcOperationKind::Install, runtime_preference)
    }

    pub const fn update(runtime_preference: MsvcRuntimePreference) -> Self {
        Self::new(MsvcOperationKind::Update, runtime_preference)
    }

    pub const fn uninstall(runtime_preference: MsvcRuntimePreference) -> Self {
        Self::new(MsvcOperationKind::Uninstall, runtime_preference)
    }

    pub const fn validate(runtime_preference: MsvcRuntimePreference) -> Self {
        Self::new(MsvcOperationKind::Validate, runtime_preference)
    }
}

// ---------------------------------------------------------------------------
// Operation outcome
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct MsvcOperationOutcome {
    pub kind: &'static str,
    pub runtime: MsvcRuntimeKind,
    pub operation: MsvcOperationKind,
    pub status: bool,
    pub title: String,
    pub output: Vec<String>,
    pub streamed: bool,
}

impl MsvcOperationOutcome {
    pub const fn is_success(&self) -> bool {
        self.status
    }
}

// ---------------------------------------------------------------------------
// Official installer mode
// ---------------------------------------------------------------------------

/// Official installer display/install mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficialInstallerMode {
    Quiet,
    Passive,
}

impl OfficialInstallerMode {
    pub fn as_cli_token(self) -> &'static str {
        match self {
            Self::Quiet => "quiet",
            Self::Passive => "passive",
        }
    }
}
