mod cache;
pub mod msvc;
pub mod scoop;

use std::path::{Path, PathBuf};

pub use crate::formatting::format_bytes;
pub use cache::{
    CacheActionOutcome, CachePaths, CacheRoots, CacheScope,
    action_result_for_tool_root as cache_action_result, clear_for_tool_root as clear_cache,
    prune_for_tool_root as prune_cache, prune_lines as cache_prune_lines,
    clear_lines as cache_clear_lines, roots_for_tool_root as cache_roots_for_tool_root,
};
pub use spoon_core::{
    SpoonEvent, CancellationToken, CommandStatus, FinishEvent,
    NoticeEvent, NoticeLevel, ProgressEvent, ProgressKind, ProgressState, ProgressUnit,
    StageEvent,
};

pub(crate) type GlobalConfig = crate::config::GlobalConfig;
pub(crate) type PolicyConfig = crate::config::PolicyConfig;
pub(crate) type ConfigEntry = crate::packages::ConfigEntry;

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub title: String,
    pub status: CommandStatus,
}

impl CommandResult {
    pub const fn is_success(&self) -> bool {
        self.status.is_success()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamChunk {
    Append(String),
    ReplaceLast(String),
}

#[derive(Debug, Clone, Copy)]
pub struct PackageRef {
    pub display_name: &'static str,
    pub package_name: &'static str,
}

pub fn configured_tool_root() -> Option<PathBuf> {
    crate::config::configured_tool_root()
}

pub(crate) fn load_policy_config() -> PolicyConfig {
    crate::config::load_policy_config()
}

/// App-owned port that implements `ScoopPorts`
/// for use with spoon-scoop workflow functions.
pub(crate) struct AppSystemPort;

impl spoon_scoop::ScoopPorts for AppSystemPort {
    fn ensure_user_path_entry(&self, path: &Path) -> spoon_scoop::Result<()> {
        crate::config::ensure_user_path_entry(path)
            .map_err(|err| spoon_scoop::ScoopError::Other(format!("failed to update user PATH: {err}")))
    }

    fn ensure_process_path_entry(&self, path: &Path) {
        crate::config::ensure_process_path_entry(path);
    }

    fn remove_user_path_entry(&self, path: &Path) -> spoon_scoop::Result<()> {
        crate::config::remove_user_path_entry(path)
            .map_err(|err| spoon_scoop::ScoopError::Other(format!("failed to update user PATH: {err}")))
    }

    fn remove_process_path_entry(&self, path: &Path) {
        crate::config::remove_process_path_entry(path);
    }

    fn supplemental_shims(
        &self,
        package_name: &str,
        current_root: &Path,
    ) -> Vec<spoon_scoop::SupplementalShimSpec> {
        crate::packages::supplemental_shims(package_name, current_root)
            .into_iter()
            .map(|spec| spoon_scoop::SupplementalShimSpec {
                alias: spec.alias,
                relative_path: spec.relative_path,
            })
            .collect()
    }

    fn apply_integrations(
        &self,
        _package_name: &str,
        _current_root: &Path,
        _persist_root: &Path,
    ) -> spoon_scoop::Result<Vec<spoon_scoop::AppliedIntegration>> {
        // Integration scripts are platform-specific and handled by the binary layer.
        // The spoon-scoop workflow functions don't call this yet (_ports is unused).
        Ok(Vec::new())
    }
}

pub(crate) static APP_PORTS: AppSystemPort = AppSystemPort;

pub fn msvc_arch_from_config(global: &GlobalConfig) -> String {
    crate::config::msvc_arch_from_config(global)
}

pub fn native_msvc_arch() -> &'static str {
    crate::config::native_msvc_arch()
}

pub fn ensure_process_path_entry(path: &Path) {
    crate::config::ensure_process_path_entry(path)
}

pub fn ensure_user_path_entry(path: &Path) -> anyhow::Result<()> {
    crate::config::ensure_user_path_entry(path)
}

pub fn remove_process_path_entry(path: &Path) {
    crate::config::remove_process_path_entry(path)
}

pub fn remove_user_path_entry(path: &Path) -> anyhow::Result<()> {
    crate::config::remove_user_path_entry(path)
}

#[cfg(test)]
pub fn enable_test_mode() {
    crate::config::enable_test_mode()
}

#[cfg(test)]
pub fn set_home_override(path: PathBuf) {
    crate::config::set_home_override(path)
}

pub fn test_mode_enabled() -> bool {
    crate::config::test_mode_enabled()
}

pub(crate) fn desired_policy_entries(package_name: &str) -> Vec<ConfigEntry> {
    let policy = load_policy_config();
    crate::packages::desired_policy_entries(package_name, &policy)
}

pub(crate) fn resolved_pip_mirror_url_for_display(policy_value: &str) -> String {
    crate::packages::python::resolved_pip_mirror_url_for_display(policy_value)
}

fn format_stage(stage: &StageEvent) -> String {
    match stage.state {
        ProgressState::Completed => format!("Stage complete: {}", stage.stage.as_str()),
        ProgressState::Running => format!("Stage: {}", stage.stage.as_str()),
    }
}

fn format_progress(progress: &ProgressEvent) -> String {
    match progress.unit {
        ProgressUnit::Bytes => match (progress.current, progress.total) {
            (Some(current), Some(total)) => {
                let percent = if total == 0 {
                    0
                } else {
                    ((current as f64 / total as f64) * 100.0)
                        .clamp(0.0, 100.0)
                        .round() as u64
                };
                let current_mb = current as f64 / (1024.0 * 1024.0);
                let total_mb = total as f64 / (1024.0 * 1024.0);
                format!(
                    "Download progress {percent}% ({current_mb:.1} MB / {total_mb:.1} MB) {}",
                    progress.label
                )
            }
            (Some(current), None) => {
                let downloaded_mb = current as f64 / (1024.0 * 1024.0);
                format!(
                    "Download progress ({downloaded_mb:.1} MB downloaded) {}",
                    progress.label
                )
            }
            _ => progress.label.clone(),
        },
        _ => match (progress.current, progress.total) {
            (Some(current), Some(total)) => format!("{} {}/{}", progress.label, current, total),
            (Some(current), None) => format!("{} {}", progress.label, current),
            _ => progress.label.clone(),
        },
    }
}

fn format_notice(notice: &NoticeEvent) -> String {
    match notice.level {
        NoticeLevel::Info => notice.message.clone(),
        NoticeLevel::Warning => format!("Warning: {}", notice.message),
    }
}

pub fn stream_chunk_from_event(event: SpoonEvent) -> Option<StreamChunk> {
    match event {
        SpoonEvent::Stage(stage) => Some(StreamChunk::ReplaceLast(format_stage(&stage))),
        SpoonEvent::Progress(progress) => {
            Some(StreamChunk::ReplaceLast(format_progress(&progress)))
        }
        SpoonEvent::Notice(notice) => Some(StreamChunk::Append(format_notice(&notice))),
        SpoonEvent::Finished(finish) => match finish.message {
            Some(message) => Some(StreamChunk::Append(message)),
            None if matches!(finish.status, CommandStatus::Success) => None,
            None if matches!(finish.status, CommandStatus::Cancelled) => {
                Some(StreamChunk::Append("Cancelled by user.".to_string()))
            }
            None if matches!(finish.status, CommandStatus::Failed) => {
                Some(StreamChunk::Append("Operation failed.".to_string()))
            }
            None => Some(StreamChunk::Append("Operation blocked.".to_string())),
        },
    }
}

#[cfg(test)]
pub use crate::runtime::test_block_on;
