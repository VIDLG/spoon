#[path = "report.rs"]
mod report;

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::service::{
    BackendEvent, CancellationToken, CommandResult, CommandStatus, StreamChunk, backend_to_anyhow,
    build_msvc_backend_context, command_result_from_msvc_outcome, stream_chunk_from_backend_event,
};
/// MSVC adapter functions construct an explicit [BackendContext](spoon_backend::BackendContext)
/// at the app boundary and delegate to context-driven backend entry points.
/// See [`build_msvc_backend_context`] for the context construction helper.

pub use spoon_backend::msvc::ToolchainFlags;
pub(crate) use spoon_backend::msvc::{
    installed_toolchain_version_label, latest_toolchain_version_label,
};

type MsvcBackendContext = spoon_backend::BackendContext<()>;
type MsvcBackendOutcome = spoon_backend::msvc::MsvcOperationOutcome;

fn context_for(tool_root: &Path) -> MsvcBackendContext {
    build_msvc_backend_context(tool_root)
}

fn forward_backend_event_to_stream<'a, F>(emit: &'a mut F) -> impl FnMut(BackendEvent) + 'a
where
    F: FnMut(StreamChunk),
{
    move |event: BackendEvent| {
        if let Some(chunk) = stream_chunk_from_backend_event(event) {
            emit(chunk);
        }
    }
}

fn command_result_from_backend_outcome(
    result: spoon_backend::Result<MsvcBackendOutcome>,
) -> Result<CommandResult> {
    backend_to_anyhow(result.map(command_result_from_msvc_outcome))
}

fn command_result_from_streamed_msvc_outcome(
    mut outcome: spoon_backend::msvc::MsvcOperationOutcome,
) -> CommandResult {
    outcome.streamed = false;
    command_result_from_msvc_outcome(outcome)
}

pub(crate) fn runtime_state_path(tool_root: &Path) -> PathBuf {
    spoon_backend::msvc::runtime_state_path(tool_root)
}

pub mod official {
    use std::path::Path;

    use anyhow::Result;

    use crate::service::{
        CancellationToken, CommandResult, StreamChunk, backend_to_anyhow,
        msvc::command_result_from_backend_outcome, msvc::command_result_from_streamed_msvc_outcome,
        msvc::context_for, msvc::forward_backend_event_to_stream,
    };

    pub use spoon_backend::msvc::official::{
        OfficialInstalledState, OfficialInstallerMode, installed_state_path,
        official_instance_root, probe, read_installed_version_label, runtime_state_path,
        vswhere_path, windows_kits_root,
    };

    pub async fn install_toolchain_async_with_mode(
        tool_root: &Path,
        mode: OfficialInstallerMode,
    ) -> Result<CommandResult> {
        let context = context_for(tool_root);
        command_result_from_backend_outcome(
            spoon_backend::msvc::official::install_toolchain_async_with_mode_and_context(
                &context, mode,
            )
            .await,
        )
    }

    pub async fn update_toolchain_async_with_mode(
        tool_root: &Path,
        mode: OfficialInstallerMode,
    ) -> Result<CommandResult> {
        let context = context_for(tool_root);
        command_result_from_backend_outcome(
            spoon_backend::msvc::official::update_toolchain_async_with_mode_and_context(
                &context, mode,
            )
            .await,
        )
    }

    pub async fn uninstall_toolchain_async(
        tool_root: &Path,
        mode: OfficialInstallerMode,
    ) -> Result<CommandResult> {
        let context = context_for(tool_root);
        backend_to_anyhow(
            spoon_backend::msvc::official::uninstall_toolchain_async_with_context(&context, mode)
                .await
                .map(super::command_result_from_msvc_outcome),
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
        let context = context_for(tool_root);
        let mut backend_emit = forward_backend_event_to_stream(emit);
        backend_to_anyhow(
            spoon_backend::msvc::official::install_toolchain_streaming_with_context(
                &context,
                mode,
                cancel,
                &mut backend_emit,
            )
            .await
            .map(command_result_from_streamed_msvc_outcome),
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
        let context = context_for(tool_root);
        let mut backend_emit = forward_backend_event_to_stream(emit);
        backend_to_anyhow(
            spoon_backend::msvc::official::update_toolchain_streaming_with_context(
                &context,
                mode,
                cancel,
                &mut backend_emit,
            )
            .await
            .map(command_result_from_streamed_msvc_outcome),
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
        let context = context_for(tool_root);
        let mut backend_emit = forward_backend_event_to_stream(emit);
        backend_to_anyhow(
            spoon_backend::msvc::official::uninstall_toolchain_streaming_with_context(
                &context,
                mode,
                cancel,
                &mut backend_emit,
            )
            .await
            .map(command_result_from_streamed_msvc_outcome),
        )
    }

    pub async fn validate_toolchain(tool_root: &Path) -> Result<CommandResult> {
        let context = context_for(tool_root);
        command_result_from_backend_outcome(
            spoon_backend::msvc::official::validate_toolchain_with_context(&context).await,
        )
    }
}

pub async fn status_report(tool_root: &Path) -> CommandResult {
    let context = context_for(tool_root);
    let output =
        report::status_report_lines(spoon_backend::msvc::status_with_context(&context).await);
    CommandResult {
        title: "status MSVC runtimes".to_string(),
        status: CommandStatus::Success,
        output,
        streamed: false,
    }
}

pub async fn status(tool_root: &Path) -> spoon_backend::msvc::MsvcStatus {
    let context = context_for(tool_root);
    spoon_backend::msvc::status_with_context(&context).await
}

pub async fn validate_toolchain(tool_root: &Path) -> Result<CommandResult> {
    let context = context_for(tool_root);
    command_result_from_backend_outcome(
        spoon_backend::msvc::validate_toolchain_with_context(&context).await,
    )
}

pub async fn managed_toolchain_flags(tool_root: &Path) -> Result<ToolchainFlags> {
    let context = context_for(tool_root);
    backend_to_anyhow(spoon_backend::msvc::managed_toolchain_flags_with_context(&context).await)
}

pub fn write_managed_toolchain_wrappers(
    tool_root: &Path,
    command_profile: &str,
    flags: &ToolchainFlags,
) -> Result<Vec<String>> {
    backend_to_anyhow(spoon_backend::msvc::write_managed_toolchain_wrappers(
        tool_root,
        command_profile,
        flags,
    ))
}

pub fn remove_managed_toolchain_wrappers(tool_root: &Path) -> Result<Vec<String>> {
    backend_to_anyhow(spoon_backend::msvc::remove_managed_toolchain_wrappers(
        tool_root,
    ))
}

pub async fn reapply_managed_command_surface_streaming(
    tool_root: &Path,
    command_profile: &str,
    emit: &mut dyn FnMut(StreamChunk),
) -> Result<Vec<String>> {
    let mut backend_emit = |event: BackendEvent| {
        if let Some(chunk) = stream_chunk_from_backend_event(event) {
            emit(chunk);
        }
    };
    backend_to_anyhow(
        spoon_backend::msvc::reapply_managed_command_surface_streaming(
            tool_root,
            command_profile,
            &mut backend_emit,
        )
        .await,
    )
}

pub async fn install_toolchain_async(tool_root: &Path) -> Result<CommandResult> {
    let context = context_for(tool_root);
    command_result_from_backend_outcome(
        spoon_backend::msvc::install_toolchain_async_with_context(&context).await,
    )
}

pub async fn update_toolchain_async(tool_root: &Path) -> Result<CommandResult> {
    let context = context_for(tool_root);
    command_result_from_backend_outcome(
        spoon_backend::msvc::update_toolchain_async_with_context(&context).await,
    )
}

pub(crate) async fn install_toolchain_async_streaming<F>(
    tool_root: &Path,
    emit: &mut F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let context = context_for(tool_root);
    let mut backend_emit = forward_backend_event_to_stream(emit);
    backend_to_anyhow(
        spoon_backend::msvc::install_toolchain_streaming_with_context(
            &context,
            None,
            &mut backend_emit,
        )
        .await
        .map(command_result_from_streamed_msvc_outcome),
    )
}

pub(crate) async fn update_toolchain_async_streaming<F>(
    tool_root: &Path,
    emit: &mut F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let context = context_for(tool_root);
    let mut backend_emit = forward_backend_event_to_stream(emit);
    backend_to_anyhow(
        spoon_backend::msvc::update_toolchain_streaming_with_context(
            &context,
            None,
            &mut backend_emit,
        )
        .await
        .map(command_result_from_streamed_msvc_outcome),
    )
}

pub(crate) async fn install_toolchain_streaming<F>(
    tool_root: &Path,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let context = context_for(tool_root);
    let mut backend_emit = forward_backend_event_to_stream(emit);
    backend_to_anyhow(
        spoon_backend::msvc::install_toolchain_streaming_with_context(
            &context,
            cancel,
            &mut backend_emit,
        )
        .await
        .map(command_result_from_streamed_msvc_outcome),
    )
}

pub(crate) async fn update_toolchain_streaming<F>(
    tool_root: &Path,
    cancel: Option<&CancellationToken>,
    emit: &mut F,
) -> Result<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let context = context_for(tool_root);
    let mut backend_emit = forward_backend_event_to_stream(emit);
    backend_to_anyhow(
        spoon_backend::msvc::update_toolchain_streaming_with_context(
            &context,
            cancel,
            &mut backend_emit,
        )
        .await
        .map(command_result_from_streamed_msvc_outcome),
    )
}

pub async fn uninstall_toolchain(tool_root: &Path) -> Result<CommandResult> {
    let context = context_for(tool_root);
    command_result_from_backend_outcome(
        spoon_backend::msvc::uninstall_toolchain_with_context(&context).await,
    )
}
