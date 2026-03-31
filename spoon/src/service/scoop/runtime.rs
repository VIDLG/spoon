use std::path::Path;

use anyhow::Result as AnyResult;

use crate::service::{
    BackendEvent, CancellationToken, StreamChunk, backend_to_anyhow,
    build_scoop_backend_context, stream_chunk_from_backend_event,
};

use super::ScoopPackagePlan;

pub(crate) async fn doctor_details(
    tool_root: &Path,
) -> AnyResult<spoon_backend::scoop::ScoopDoctorDetails> {
    let context = build_scoop_backend_context(tool_root);
    backend_to_anyhow(spoon_backend::scoop::doctor_with_context(&context).await)
}

pub(crate) fn resolved_pip_mirror_url_for_display(policy_value: &str) -> String {
    crate::service::resolved_pip_mirror_url_for_display(policy_value)
}

pub(crate) async fn reapply_package_integrations_streaming(
    tool_root: &Path,
    package_name: &str,
    emit: &mut dyn FnMut(StreamChunk),
) -> AnyResult<Vec<String>> {
    let context = build_scoop_backend_context(tool_root);
    let mut backend_emit = |event: BackendEvent| {
        if let Some(chunk) = stream_chunk_from_backend_event(event) {
            emit(chunk);
        }
    };
    backend_to_anyhow(
        spoon_backend::scoop::runtime::reapply_package_integrations_streaming_with_context(
            &context,
            package_name,
            &mut backend_emit,
        )
        .await,
    )
}

pub(crate) async fn reapply_package_command_surface_streaming(
    tool_root: &Path,
    package_name: &str,
    emit: &mut dyn FnMut(StreamChunk),
) -> AnyResult<Vec<String>> {
    let context = build_scoop_backend_context(tool_root);
    let mut backend_emit = |event: BackendEvent| {
        if let Some(chunk) = stream_chunk_from_backend_event(event) {
            emit(chunk);
        }
    };
    backend_to_anyhow(
        spoon_backend::scoop::runtime::reapply_package_command_surface_streaming_with_context(
            &context,
            package_name,
            &mut backend_emit,
        )
        .await,
    )
}

pub(crate) async fn execute_package_action_outcome_streaming(
    tool_root: &Path,
    plan: &ScoopPackagePlan,
    _proxy: &str,
    cancel: Option<&CancellationToken>,
    emit: Option<&mut dyn FnMut(StreamChunk)>,
) -> AnyResult<spoon_backend::scoop::ScoopPackageOperationOutcome> {
    let context = build_scoop_backend_context(tool_root);
    match emit {
        Some(emit) => {
            let mut backend_emit = |event: BackendEvent| {
                if let Some(chunk) = stream_chunk_from_backend_event(event) {
                    emit(chunk);
                }
            };
            backend_to_anyhow(
                spoon_backend::scoop::runtime::execute_package_action_outcome_streaming_with_context(
                    &context,
                    plan,
                    cancel,
                    Some(&mut backend_emit),
                )
                .await,
            )
        }
        None => backend_to_anyhow(
            spoon_backend::scoop::runtime::execute_package_action_outcome_streaming_with_context(
                &context, plan, cancel, None,
            )
            .await,
        ),
    }
}
