use std::path::Path;

use serde::Serialize;

use crate::official;
use crate::paths;
use crate::rules::read_installed_toolchain_target;
use crate::types::MsvcRuntimeKind;

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

fn installed_toolchain_version_label(tool_root: &Path) -> Option<String> {
    let target = read_installed_toolchain_target(&paths::msvc_root(tool_root))?;
    Some(user_facing_toolchain_label(&target.label()))
}

fn user_facing_toolchain_label(raw: &str) -> String {
    raw.replace("msvc-", "").replace("sdk-", "")
}

fn runtime_state_path(tool_root: &Path) -> std::path::PathBuf {
    crate::paths::msvc_state_root(tool_root).join("runtime.json")
}
