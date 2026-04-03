use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use crate::control_plane::ControlPlaneDb;
use crate::layout::RuntimeLayout;
use crate::{BackendError, Result};

use super::{MsvcLifecycleStage, MsvcOperationKind, MsvcRuntimeKind};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MsvcValidationStatus {
    Valid,
    Invalid,
    Unknown,
}

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

#[derive(Debug)]
struct StoredMsvcStateRow {
    runtime_kind: String,
    installed: i64,
    version: Option<String>,
    sdk_version: Option<String>,
    last_operation: Option<String>,
    last_stage: Option<String>,
    validation_status: Option<String>,
    validation_message: Option<String>,
    managed_detail: String,
    official_detail: String,
}

impl TryFrom<StoredMsvcStateRow> for MsvcCanonicalState {
    type Error = ();

    fn try_from(row: StoredMsvcStateRow) -> std::result::Result<Self, Self::Error> {
        Ok(MsvcCanonicalState {
            runtime_kind: serde_json::from_str(&format!("\"{}\"", row.runtime_kind))
                .map_err(|_| ())?,
            installed: row.installed != 0,
            version: row.version,
            sdk_version: row.sdk_version,
            last_operation: row
                .last_operation
                .as_deref()
                .and_then(parse_operation_kind),
            last_stage: row.last_stage.as_deref().and_then(parse_lifecycle_stage),
            validation_status: row
                .validation_status
                .as_deref()
                .and_then(parse_validation_status),
            validation_message: row.validation_message,
            managed: serde_json::from_str(&row.managed_detail).map_err(|_| ())?,
            official: serde_json::from_str(&row.official_detail).map_err(|_| ())?,
        })
    }
}

fn parse_operation_kind(value: &str) -> Option<MsvcOperationKind> {
    match value {
        "install" => Some(MsvcOperationKind::Install),
        "update" => Some(MsvcOperationKind::Update),
        "uninstall" => Some(MsvcOperationKind::Uninstall),
        "validate" => Some(MsvcOperationKind::Validate),
        _ => None,
    }
}

fn parse_lifecycle_stage(value: &str) -> Option<MsvcLifecycleStage> {
    match value {
        "planned" => Some(MsvcLifecycleStage::Planned),
        "detecting" => Some(MsvcLifecycleStage::Detecting),
        "resolving" => Some(MsvcLifecycleStage::Resolving),
        "executing" => Some(MsvcLifecycleStage::Executing),
        "validating" => Some(MsvcLifecycleStage::Validating),
        "state_committing" => Some(MsvcLifecycleStage::StateCommitting),
        "completed" => Some(MsvcLifecycleStage::Completed),
        _ => None,
    }
}

fn parse_validation_status(value: &str) -> Option<MsvcValidationStatus> {
    match value {
        "valid" => Some(MsvcValidationStatus::Valid),
        "invalid" => Some(MsvcValidationStatus::Invalid),
        "unknown" => Some(MsvcValidationStatus::Unknown),
        _ => None,
    }
}

fn runtime_kind_label(kind: MsvcRuntimeKind) -> &'static str {
    match kind {
        MsvcRuntimeKind::Managed => "managed",
        MsvcRuntimeKind::Official => "official",
    }
}

fn operation_kind_label(kind: MsvcOperationKind) -> &'static str {
    match kind {
        MsvcOperationKind::Install => "install",
        MsvcOperationKind::Update => "update",
        MsvcOperationKind::Uninstall => "uninstall",
        MsvcOperationKind::Validate => "validate",
    }
}

fn validation_status_label(status: MsvcValidationStatus) -> &'static str {
    match status {
        MsvcValidationStatus::Valid => "valid",
        MsvcValidationStatus::Invalid => "invalid",
        MsvcValidationStatus::Unknown => "unknown",
    }
}

pub async fn read_canonical_state(layout: &RuntimeLayout) -> Option<MsvcCanonicalState> {
    let db = ControlPlaneDb::open_for_layout(layout).await.ok()?;
    let row = db
        .call(|conn| {
            conn.query_row(
                "SELECT runtime_kind, installed, version, sdk_version, last_operation, last_stage, validation_status, validation_message, managed_detail, official_detail
                 FROM msvc_runtime_state WHERE singleton_key = 'msvc'",
                [],
                |row| {
                    Ok(StoredMsvcStateRow {
                        runtime_kind: row.get(0)?,
                        installed: row.get(1)?,
                        version: row.get(2)?,
                        sdk_version: row.get(3)?,
                        last_operation: row.get(4)?,
                        last_stage: row.get(5)?,
                        validation_status: row.get(6)?,
                        validation_message: row.get(7)?,
                        managed_detail: row.get(8)?,
                        official_detail: row.get(9)?,
                    })
                },
            )
            .optional()
        })
        .await
        .ok()?;
    row.and_then(|row| MsvcCanonicalState::try_from(row).ok())
}

pub async fn write_canonical_state(
    layout: &RuntimeLayout,
    state: &MsvcCanonicalState,
) -> Result<()> {
    let db = ControlPlaneDb::open_for_layout(layout).await?;
    let runtime_kind = runtime_kind_label(state.runtime_kind).to_string();
    let installed = if state.installed { 1_i64 } else { 0_i64 };
    let version = state.version.clone();
    let sdk_version = state.sdk_version.clone();
    let last_operation = state.last_operation.map(operation_kind_label).map(str::to_string);
    let last_stage = state.last_stage.map(MsvcLifecycleStage::as_str).map(str::to_string);
    let validation_status = state
        .validation_status
        .clone()
        .map(validation_status_label)
        .map(str::to_string);
    let validation_message = state.validation_message.clone();
    let managed_detail = serde_json::to_string(&state.managed)
        .map_err(|err| BackendError::external("failed to serialize managed MSVC detail", err))?;
    let official_detail = serde_json::to_string(&state.official)
        .map_err(|err| BackendError::external("failed to serialize official MSVC detail", err))?;

    db.call_write(move |conn| {
        conn.execute(
            "INSERT INTO msvc_runtime_state
                (singleton_key, runtime_kind, installed, version, sdk_version, last_operation, last_stage, validation_status, validation_message, managed_detail, official_detail)
             VALUES ('msvc', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(singleton_key) DO UPDATE SET
                runtime_kind = excluded.runtime_kind,
                installed = excluded.installed,
                version = excluded.version,
                sdk_version = excluded.sdk_version,
                last_operation = excluded.last_operation,
                last_stage = excluded.last_stage,
                validation_status = excluded.validation_status,
                validation_message = excluded.validation_message,
                managed_detail = excluded.managed_detail,
                official_detail = excluded.official_detail,
                updated_at = datetime('now')",
            params![
                runtime_kind,
                installed,
                version,
                sdk_version,
                last_operation,
                last_stage,
                validation_status,
                validation_message,
                managed_detail,
                official_detail,
            ],
        )?;
        Ok(())
    })
    .await
}

pub async fn clear_canonical_state(layout: &RuntimeLayout) -> Result<()> {
    let db = ControlPlaneDb::open_for_layout(layout).await?;
    db.call_write(|conn| {
        conn.execute("DELETE FROM msvc_runtime_state WHERE singleton_key = 'msvc'", [])?;
        Ok(())
    })
    .await
}
