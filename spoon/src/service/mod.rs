mod cache;
pub mod msvc;
pub mod scoop;

use std::collections::BTreeMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use anyhow::Result as AnyResult;

pub use crate::formatting::format_bytes;
pub use cache::{
    CacheActionOutcome, CachePaths, CacheRoots, CacheScope,
    action_result_for_tool_root as cache_action_result, clear_for_tool_root as clear_cache,
    prune_for_tool_root as prune_cache,
};
pub use spoon_backend::{
    BackendContext, BackendError, BackendEvent, CancellationToken, CommandStatus, FinishEvent,
    ProgressEvent, ProgressState, ProgressUnit, Result, SystemPort,
};
use spoon_backend::scoop::ScoopIntegrationPort;

pub(crate) type GlobalConfig = crate::config::GlobalConfig;
pub(crate) type PolicyConfig = crate::config::PolicyConfig;
pub(crate) type ConfigEntry = crate::packages::ConfigEntry;
pub(crate) type SupplementalShimSpec = crate::packages::SupplementalShimSpec;

pub(crate) fn backend_to_anyhow<T>(result: spoon_backend::Result<T>) -> AnyResult<T> {
    result.map_err(Into::into)
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub title: String,
    pub status: CommandStatus,
    pub output: Vec<String>,
    pub streamed: bool,
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

#[derive(Debug, Clone)]
pub struct BackendConfig {
    global: GlobalConfig,
    policy: PolicyConfig,
}

impl BackendConfig {
    pub fn root_override(&self) -> Option<&str> {
        let trimmed = self.global.root.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }

    pub fn proxy(&self) -> &str {
        self.global.proxy.as_str()
    }

    pub fn msvc_command_profile(&self) -> &str {
        self.policy.msvc.command_profile.as_str()
    }

    pub fn selected_msvc_arch(&self) -> String {
        msvc_arch_from_config(&self.global)
    }
}

pub fn configured_tool_root() -> Option<PathBuf> {
    crate::config::configured_tool_root()
}

pub fn home_dir() -> PathBuf {
    crate::config::home_dir()
}

pub(crate) fn load_global_config() -> GlobalConfig {
    crate::config::load_global_config()
}

pub(crate) fn load_policy_config() -> PolicyConfig {
    crate::config::load_policy_config()
}

pub(crate) fn load_backend_config() -> BackendConfig {
    BackendConfig {
        global: load_global_config(),
        policy: load_policy_config(),
    }
}

/// App-owned port that implements both `SystemPort` and `ScoopIntegrationPort`
/// for use with `BackendContext<AppSystemPort>`.
pub(crate) struct AppSystemPort;

impl SystemPort for AppSystemPort {
    fn home_dir(&self) -> PathBuf {
        crate::config::home_dir()
    }

    fn ensure_user_path_entry(&self, path: &Path) -> spoon_backend::Result<()> {
        crate::config::ensure_user_path_entry(path)
            .map_err(|err| BackendError::Other(format!("failed to update user PATH: {err}")))
    }

    fn ensure_process_path_entry(&self, path: &Path) {
        crate::config::ensure_process_path_entry(path);
    }

    fn remove_user_path_entry(&self, path: &Path) -> spoon_backend::Result<()> {
        crate::config::remove_user_path_entry(path)
            .map_err(|err| BackendError::Other(format!("failed to update user PATH: {err}")))
    }

    fn remove_process_path_entry(&self, path: &Path) {
        crate::config::remove_process_path_entry(path);
    }
}

impl SystemPort for &'static AppSystemPort {
    fn home_dir(&self) -> PathBuf {
        (*self).home_dir()
    }

    fn ensure_user_path_entry(&self, path: &Path) -> spoon_backend::Result<()> {
        (*self).ensure_user_path_entry(path)
    }

    fn ensure_process_path_entry(&self, path: &Path) {
        (*self).ensure_process_path_entry(path);
    }

    fn remove_user_path_entry(&self, path: &Path) -> spoon_backend::Result<()> {
        (*self).remove_user_path_entry(path)
    }

    fn remove_process_path_entry(&self, path: &Path) {
        (*self).remove_process_path_entry(path);
    }
}

impl ScoopIntegrationPort for AppSystemPort {
    fn supplemental_shims(
        &self,
        package_name: &str,
        current_root: &Path,
    ) -> Vec<spoon_backend::SupplementalShimSpec> {
        supplemental_shims(package_name, current_root)
            .into_iter()
            .map(|spec| spoon_backend::SupplementalShimSpec {
                alias: spec.alias,
                relative_path: spec.relative_path,
            })
            .collect()
    }

    fn apply_integrations<'a>(
        &'a self,
        package_name: &'a str,
        _current_root: &'a Path,
        _persist_root: &'a Path,
        _emit: &'a mut dyn FnMut(BackendEvent),
    ) -> Pin<Box<dyn Future<Output = spoon_backend::Result<BTreeMap<String, String>>> + 'a>> {
        Box::pin(async move {
            let mut mapped = |chunk: StreamChunk| match chunk {
                StreamChunk::Append(line) | StreamChunk::ReplaceLast(line) => {
                    tracing::info!("{line}")
                }
            };
            apply_integrations_backend(package_name, &mut mapped).await
        })
    }
}

impl ScoopIntegrationPort for &'static AppSystemPort {
    fn supplemental_shims(
        &self,
        package_name: &str,
        current_root: &Path,
    ) -> Vec<spoon_backend::SupplementalShimSpec> {
        (*self).supplemental_shims(package_name, current_root)
    }

    fn apply_integrations<'a>(
        &'a self,
        package_name: &'a str,
        current_root: &'a Path,
        persist_root: &'a Path,
        emit: &'a mut dyn FnMut(BackendEvent),
    ) -> Pin<Box<dyn Future<Output = spoon_backend::Result<BTreeMap<String, String>>> + 'a>> {
        (*self).apply_integrations(package_name, current_root, persist_root, emit)
    }
}

static APP_PORTS: AppSystemPort = AppSystemPort;

/// Build a `BackendContext<AppSystemPort>` from the current app configuration.
/// This is the single shared entry point for constructing backend context in the
/// Scoop service layer, ensuring that root, proxy, and MSVC settings are read
/// once and passed explicitly into backend operations.
pub(crate) fn build_scoop_backend_context(
    tool_root: &Path,
) -> BackendContext<&'static AppSystemPort> {
    let backend = load_backend_config();
    BackendContext::new(
        tool_root.to_path_buf(),
        (!backend.proxy().trim().is_empty()).then(|| backend.proxy().to_string()),
        test_mode_enabled(),
        backend.selected_msvc_arch(),
        backend.msvc_command_profile(),
        &APP_PORTS,
    )
}

pub(crate) fn build_msvc_backend_context(tool_root: &Path) -> spoon_backend::BackendContext<()> {
    let backend = load_backend_config();
    spoon_backend::BackendContext::new(
        tool_root.to_path_buf(),
        (!backend.proxy().trim().is_empty()).then(|| backend.proxy().to_string()),
        test_mode_enabled(),
        backend.selected_msvc_arch(),
        backend.msvc_command_profile(),
        (),
    )
}

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

pub(crate) fn ensure_user_path_entry_backend(path: &Path) -> spoon_backend::Result<()> {
    ensure_user_path_entry(path)
        .map_err(|err| BackendError::Other(format!("failed to update user PATH: {err}")))
}

pub fn remove_process_path_entry(path: &Path) {
    crate::config::remove_process_path_entry(path)
}

pub fn remove_user_path_entry(path: &Path) -> anyhow::Result<()> {
    crate::config::remove_user_path_entry(path)
}

pub(crate) fn remove_user_path_entry_backend(path: &Path) -> spoon_backend::Result<()> {
    remove_user_path_entry(path)
        .map_err(|err| BackendError::Other(format!("failed to update user PATH: {err}")))
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

pub(crate) async fn apply_integrations(
    package_name: &str,
    emit: &mut dyn FnMut(StreamChunk),
) -> AnyResult<BTreeMap<String, String>> {
    crate::packages::apply_integrations(package_name, emit)
}

pub(crate) async fn apply_integrations_backend(
    package_name: &str,
    emit: &mut dyn FnMut(StreamChunk),
) -> spoon_backend::Result<BTreeMap<String, String>> {
    apply_integrations(package_name, emit)
        .await
        .map_err(|err| BackendError::Other(format!("failed to apply integrations: {err}")))
}

pub(crate) fn supplemental_shims(
    package_name: &str,
    current_root: &Path,
) -> Vec<SupplementalShimSpec> {
    crate::packages::supplemental_shims(package_name, current_root)
}

pub(crate) fn resolved_pip_mirror_url_for_display(policy_value: &str) -> String {
    crate::packages::python::resolved_pip_mirror_url_for_display(policy_value)
}

fn format_backend_progress(progress: &ProgressEvent) -> String {
    if let Some(stage) = progress.stage {
        return match progress.state {
            ProgressState::Completed => format!("Stage complete: {}", stage.as_str()),
            ProgressState::Running => format!("Stage: {}", stage.as_str()),
        };
    }
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

pub fn stream_chunk_from_backend_event(event: BackendEvent) -> Option<StreamChunk> {
    match event {
        BackendEvent::Progress(progress) => {
            Some(StreamChunk::ReplaceLast(format_backend_progress(&progress)))
        }
        BackendEvent::Finished(finish) => match finish.message {
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

pub(crate) fn command_result_from_msvc_outcome(
    outcome: spoon_backend::msvc::MsvcOperationOutcome,
) -> CommandResult {
    CommandResult {
        title: outcome.title,
        status: outcome.status,
        output: outcome.output,
        streamed: outcome.streamed,
    }
}

#[cfg(test)]
pub use crate::runtime::test_block_on;
