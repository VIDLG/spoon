//! MSVC domain state — JSON file backed, no SQLite.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use spoon_core::RuntimeLayout;

use crate::types::{
    MsvcRuntimeKind, MsvcOperationKind, MsvcLifecycleStage,
};

// Re-export MsvcValidationStatus for backward compatibility
pub use crate::types::MsvcValidationStatus;

#[derive(Debug, thiserror::Error)]
pub enum MsvcStateError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("state not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, MsvcStateError>;

// ---------------------------------------------------------------------------
// State detail types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ManagedMsvcStateDetail {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_target_arch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct OfficialMsvcStateDetail {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installer_mode: Option<String>,
}

// ---------------------------------------------------------------------------
// Canonical state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MsvcCanonicalState {
    pub runtime_kind: MsvcRuntimeKind,
    pub installed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sdk_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_operation: Option<MsvcOperationKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_stage: Option<MsvcLifecycleStage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_status: Option<MsvcValidationStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_message: Option<String>,
    #[serde(default)]
    pub managed: ManagedMsvcStateDetail,
    #[serde(default)]
    pub official: OfficialMsvcStateDetail,
}

fn state_path(layout: &RuntimeLayout) -> PathBuf {
    layout.msvc.managed.root.join("state").join("runtime.json")
}

/// Synchronous read of canonical state. Returns None if the state file does not exist.
pub fn read_canonical_state(layout: &RuntimeLayout) -> Option<MsvcCanonicalState> {
    let path = state_path(layout);
    if !path.exists() {
        return None;
    }
    let text = fs_err::read_to_string(&path).ok()?;
    let state: MsvcCanonicalState = serde_json::from_str(&text).ok()?;
    Some(state)
}

/// Synchronous write of canonical state.
pub fn write_canonical_state(layout: &RuntimeLayout, state: &MsvcCanonicalState) -> Result<()> {
    let path = state_path(layout);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(&path, json)?;
    Ok(())
}

pub fn clear_canonical_state(layout: &RuntimeLayout) -> Result<()> {
    let path = state_path(layout);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}
