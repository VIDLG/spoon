use std::path::Path;

use serde::Serialize;

use crate::BackendContext;

use super::{MsvcRuntimeKind, installed_toolchain_version_label, official, runtime_state_path};

#[derive(Debug, Clone, Serialize)]
pub struct DetectedMsvcRuntime {
    pub kind: MsvcRuntimeKind,
    pub available: bool,
    pub installed_version: Option<String>,
    pub runtime_state_present: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MsvcRuntimeDetection {
    pub managed: DetectedMsvcRuntime,
    pub official: DetectedMsvcRuntime,
}

pub fn detect_runtimes(tool_root: &Path) -> MsvcRuntimeDetection {
    let managed_installed = installed_toolchain_version_label(tool_root);
    let managed_runtime_state = runtime_state_path(tool_root);
    let (_official_root, official_available, official_installed) = official::probe(tool_root);
    let official_runtime_state = official::runtime_state_path(tool_root);

    MsvcRuntimeDetection {
        managed: DetectedMsvcRuntime {
            kind: MsvcRuntimeKind::Managed,
            available: managed_installed.is_some() || managed_runtime_state.exists(),
            installed_version: managed_installed,
            runtime_state_present: managed_runtime_state.exists(),
        },
        official: DetectedMsvcRuntime {
            kind: MsvcRuntimeKind::Official,
            available: official_available,
            installed_version: official_installed,
            runtime_state_present: official_runtime_state.exists(),
        },
    }
}

pub fn detect_runtimes_with_context<P>(context: &BackendContext<P>) -> MsvcRuntimeDetection {
    detect_runtimes(&context.root)
}
