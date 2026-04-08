//! MSVC doctor — health checks for the MSVC toolchain.

use serde::Serialize;

use spoon_core::RuntimeLayout;
use crate::detect;
use crate::state::read_canonical_state;
use crate::types::MsvcRuntimeKind;

#[derive(Debug, Clone, Serialize)]
pub struct MsvcDoctorIssue {
    pub category: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MsvcDoctorReport {
    pub kind: &'static str,
    pub healthy: bool,
    pub issues: Vec<MsvcDoctorIssue>,
}

pub async fn doctor(tool_root: &std::path::Path) -> MsvcDoctorReport {
    let layout = RuntimeLayout::from_root(tool_root);
    doctor_with_layout(&layout).await
}

async fn doctor_with_layout(layout: &RuntimeLayout) -> MsvcDoctorReport {
    let canonical = read_canonical_state(layout);
    let detected = detect::detect_runtimes(&layout.root);
    let mut issues = Vec::new();

    if let Some(state) = canonical {
        match state.runtime_kind {
            MsvcRuntimeKind::Managed => {
                if state.installed && !detected.managed.available {
                    issues.push(MsvcDoctorIssue {
                        category: "canonical_runtime_drift",
                        message: "canonical MSVC state says managed is installed, but managed runtime evidence is missing".to_string(),
                    });
                }
            }
            MsvcRuntimeKind::Official => {
                if state.installed && !detected.official.available {
                    issues.push(MsvcDoctorIssue {
                        category: "canonical_runtime_drift",
                        message: "canonical MSVC state says official is installed, but official runtime evidence is missing".to_string(),
                    });
                }
            }
        }

        if matches!(state.validation_status, Some(crate::state::MsvcValidationStatus::Invalid)) {
            issues.push(MsvcDoctorIssue {
                category: "validation_failure",
                message: state
                    .validation_message
                    .unwrap_or_else(|| "MSVC validation is marked invalid".to_string()),
            });
        }
    }

    MsvcDoctorReport {
        kind: "msvc_doctor",
        healthy: issues.is_empty(),
        issues,
    }
}
