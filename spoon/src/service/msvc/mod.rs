#[path = "report.rs"]
mod report;

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::service::{CommandResult, CommandStatus, StreamChunk, stream_chunk_from_event};
pub use spoon_msvc::ToolchainFlags;
pub(crate) use spoon_msvc::status::installed_toolchain_version_label;

fn configured_proxy() -> String {
    crate::config::load_global_config().proxy.clone()
}

fn msvc_request(tool_root: &Path) -> spoon_msvc::MsvcRequest {
    spoon_msvc::MsvcRequest::for_tool_root(tool_root).proxy(configured_proxy())
}

fn command_result_from_msvc_outcome(
    outcome: spoon_msvc::MsvcOperationOutcome,
) -> CommandResult {
    CommandResult {
        title: outcome.title,
        status: if outcome.status {
            CommandStatus::Success
        } else {
            CommandStatus::Failed
        },
        output: outcome.output,
        streamed: outcome.streamed,
    }
}

pub(crate) fn command_result_from_msvc_result(
    result: spoon_core::Result<spoon_msvc::MsvcOperationOutcome>,
) -> Result<CommandResult> {
    result
        .map(command_result_from_msvc_outcome)
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub(crate) fn forward_backend_event_to_stream<'a, F>(emit: &'a mut F) -> impl FnMut(spoon_core::SpoonEvent) + 'a
where
    F: FnMut(StreamChunk),
{
    move |event: spoon_core::SpoonEvent| {
        if let Some(chunk) = stream_chunk_from_event(event) {
            emit(chunk);
        }
    }
}

pub(crate) fn runtime_state_path(tool_root: &Path) -> PathBuf {
    spoon_msvc::paths::msvc_state_root(tool_root)
}

pub mod official {
    use std::path::Path;

    use anyhow::Result;

    use crate::service::{CommandResult, StreamChunk};
    use spoon_core::CancellationToken;
    pub use spoon_msvc::OfficialInstallerMode;
    pub use spoon_msvc::official::{
        OfficialInstalledState, installed_state_path,
        official_instance_root, probe, read_installed_version_label, runtime_state_path,
        vswhere_path, windows_kits_root,
    };

    use super::{command_result_from_msvc_result, forward_backend_event_to_stream, msvc_request};

    pub async fn install_toolchain_async_with_mode(
        tool_root: &Path,
        mode: OfficialInstallerMode,
    ) -> Result<CommandResult> {
        command_result_from_msvc_result(
            spoon_msvc::official::install_toolchain_async(&msvc_request(tool_root), mode).await,
        )
    }

    pub async fn update_toolchain_async_with_mode(
        tool_root: &Path,
        mode: OfficialInstallerMode,
    ) -> Result<CommandResult> {
        command_result_from_msvc_result(
            spoon_msvc::official::update_toolchain_async(&msvc_request(tool_root), mode).await,
        )
    }

    pub async fn uninstall_toolchain_async(
        tool_root: &Path,
        mode: OfficialInstallerMode,
    ) -> Result<CommandResult> {
        command_result_from_msvc_result(
            spoon_msvc::official::uninstall_toolchain_async(&msvc_request(tool_root), mode).await,
        )
    }

    pub async fn install_toolchain_streaming<F>(
        tool_root: &Path,
        mode: OfficialInstallerMode,
        cancel: Option<&CancellationToken>,
        emit: &mut F,
    ) -> Result<CommandResult>
    where
        F: FnMut(StreamChunk),
    {
        let request = msvc_request(tool_root);
        let mut backend_emit = forward_backend_event_to_stream(emit);
        command_result_from_msvc_result(
            spoon_msvc::official::install_toolchain_streaming(
                &request, mode, cancel, &mut backend_emit,
            )
            .await,
        )
    }

    pub async fn update_toolchain_streaming<F>(
        tool_root: &Path,
        mode: OfficialInstallerMode,
        cancel: Option<&CancellationToken>,
        emit: &mut F,
    ) -> Result<CommandResult>
    where
        F: FnMut(StreamChunk),
    {
        let request = msvc_request(tool_root);
        let mut backend_emit = forward_backend_event_to_stream(emit);
        command_result_from_msvc_result(
            spoon_msvc::official::update_toolchain_streaming(
                &request, mode, cancel, &mut backend_emit,
            )
            .await,
        )
    }

    pub async fn uninstall_toolchain_streaming<F>(
        tool_root: &Path,
        mode: OfficialInstallerMode,
        cancel: Option<&CancellationToken>,
        emit: &mut F,
    ) -> Result<CommandResult>
    where
        F: FnMut(StreamChunk),
    {
        let request = msvc_request(tool_root);
        let mut backend_emit = forward_backend_event_to_stream(emit);
        command_result_from_msvc_result(
            spoon_msvc::official::uninstall_toolchain_streaming(
                &request, mode, cancel, &mut backend_emit,
            )
            .await,
        )
    }

    pub async fn validate_toolchain(tool_root: &Path) -> Result<CommandResult> {
        command_result_from_msvc_result(
            spoon_msvc::official::validate_official_toolchain_async(&msvc_request(tool_root)).await,
        )
    }
}

pub async fn status_report(tool_root: &Path) -> CommandResult {
    let output = report::status_report_lines(spoon_msvc::status::status(tool_root).await);
    CommandResult {
        title: "status MSVC runtimes".to_string(),
        status: CommandStatus::Success,
        output,
        streamed: false,
    }
}

pub async fn status(tool_root: &Path) -> spoon_msvc::status::MsvcStatus {
    spoon_msvc::status::status(tool_root).await
}

pub async fn validate_toolchain(tool_root: &Path) -> Result<CommandResult> {
    let request = msvc_request(tool_root);
    command_result_from_msvc_result(
        spoon_msvc::execute::validate_toolchain_async(&request).await,
    )
}

pub async fn managed_toolchain_flags(tool_root: &Path) -> Result<ToolchainFlags> {
    let request = msvc_request(tool_root);
    spoon_msvc::execute::managed_toolchain_flags_with_request(&request)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub fn write_managed_toolchain_wrappers(
    tool_root: &Path,
    command_profile: &str,
    flags: &ToolchainFlags,
) -> Result<Vec<String>> {
    spoon_msvc::wrappers::write_managed_toolchain_wrappers(tool_root, command_profile, flags)
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub fn remove_managed_toolchain_wrappers(tool_root: &Path) -> Result<Vec<String>> {
    spoon_msvc::wrappers::remove_managed_toolchain_wrappers(tool_root)
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub async fn reapply_managed_command_surface_streaming(
    tool_root: &Path,
    command_profile: &str,
    _emit: &mut dyn FnMut(StreamChunk),
) -> Result<Vec<String>> {
    let runtime_state = spoon_msvc::paths::msvc_state_root(tool_root);
    if !runtime_state.exists() {
        return Ok(vec![
            "Managed MSVC toolchain is not installed; no wrapper changes were applied.".to_string(),
        ]);
    }

    let flags = managed_toolchain_flags(tool_root).await?;
    let mut lines = write_managed_toolchain_wrappers(tool_root, command_profile, &flags)?;
    if lines.is_empty() {
        lines.push("Managed wrapper set already matches the selected command profile.".to_string());
    }
    for line in &lines {
        tracing::info!("{line}");
    }
    Ok(lines)
}

pub async fn install_toolchain_async(tool_root: &Path) -> Result<CommandResult> {
    let request = msvc_request(tool_root);
    command_result_from_msvc_result(
        spoon_msvc::execute::install_toolchain_async(&request).await,
    )
}

pub async fn update_toolchain_async(tool_root: &Path) -> Result<CommandResult> {
    let request = msvc_request(tool_root);
    command_result_from_msvc_result(
        spoon_msvc::execute::update_toolchain_async(&request).await,
    )
}

pub(crate) async fn install_toolchain_async_streaming<F>(
    tool_root: &Path,
    emit: &mut F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let request = msvc_request(tool_root);
    let mut backend_emit = forward_backend_event_to_stream(emit);
    command_result_from_msvc_result(
        spoon_msvc::execute::install_toolchain_streaming(&request, None, &mut backend_emit)
            .await,
    )
}

pub(crate) async fn update_toolchain_async_streaming<F>(
    tool_root: &Path,
    emit: &mut F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let request = msvc_request(tool_root);
    let mut backend_emit = forward_backend_event_to_stream(emit);
    command_result_from_msvc_result(
        spoon_msvc::execute::update_toolchain_streaming(&request, None, &mut backend_emit)
            .await,
    )
}

pub(crate) async fn install_toolchain_streaming<F>(
    tool_root: &Path,
    cancel: Option<&spoon_core::CancellationToken>,
    emit: &mut F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let request = msvc_request(tool_root);
    let mut backend_emit = forward_backend_event_to_stream(emit);
    command_result_from_msvc_result(
        spoon_msvc::execute::install_toolchain_streaming(&request, cancel, &mut backend_emit)
            .await,
    )
}

pub(crate) async fn update_toolchain_streaming<F>(
    tool_root: &Path,
    cancel: Option<&spoon_core::CancellationToken>,
    emit: &mut F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let request = msvc_request(tool_root);
    let mut backend_emit = forward_backend_event_to_stream(emit);
    command_result_from_msvc_result(
        spoon_msvc::execute::update_toolchain_streaming(&request, cancel, &mut backend_emit)
            .await,
    )
}

pub async fn uninstall_toolchain(tool_root: &Path) -> Result<CommandResult> {
    let request = msvc_request(tool_root);
    command_result_from_msvc_result(
        spoon_msvc::execute::uninstall_toolchain_async(&request).await,
    )
}

pub(crate) fn latest_toolchain_version_label(tool_root: &Path) -> Option<String> {
    let request = msvc_request(tool_root);
    let manifest_root = spoon_msvc::paths::msvc_manifest_root(&request.root);
    let target_arch = request.normalized_target_arch();
    spoon_msvc::facts::manifest::latest_toolchain_target_from_cached_manifest(
        &manifest_root,
        spoon_msvc::platform::native_host_arch(),
        &target_arch,
    )
    .map(|target| spoon_msvc::status::user_facing_toolchain_label(&target.label()))
}
