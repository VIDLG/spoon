use serde::Serialize;

use crate::BackendContext;
use crate::layout::RuntimeLayout;

use super::{MsvcRuntimeKind, detect_runtimes, read_canonical_state};

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

pub async fn doctor_with_context<P>(context: &BackendContext<P>) -> MsvcDoctorReport {
    doctor_with_layout(&context.layout).await
}

async fn doctor_with_layout(layout: &RuntimeLayout) -> MsvcDoctorReport {
    let canonical = read_canonical_state(layout).await;
    let detected = detect_runtimes(&layout.root);
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

        if matches!(state.validation_status, Some(super::MsvcValidationStatus::Invalid)) {
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
