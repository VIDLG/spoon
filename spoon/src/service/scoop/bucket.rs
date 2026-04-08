use std::path::Path;

use anyhow::Result as AnyResult;

use crate::runtime::block_on_sync;
use crate::service::{
    CommandResult, CommandStatus, StreamChunk,
};

pub use spoon_core::RepoSyncOutcome;

use super::{
    ScoopBucketInventory, BucketSpec, ScoopBucketOperationOutcome, ScoopDoctorDetails,
    add_bucket_to_registry_outcome, command_result, configured_proxy, load_buckets_from_registry,
    remove_bucket_from_registry_outcome, runtime, update_buckets_outcome,
};

fn command_result_from_bucket_outcome(outcome: ScoopBucketOperationOutcome) -> CommandResult {
    command_result(
        outcome.title,
        outcome.status,
        outcome.output,
        outcome.streamed,
    )
}

pub async fn bucket_list_report(tool_root: &Path) -> CommandResult {
    let buckets = load_buckets_from_registry(tool_root).await;
    let mut output = Vec::new();
    if buckets.is_empty() {
        output.push("No Scoop buckets are registered.".to_string());
    } else {
        for bucket in buckets {
            output.push(format!(
                "{} | {} | {}",
                bucket.name, bucket.branch, bucket.source
            ));
        }
    }
    command_result("list Scoop buckets", CommandStatus::Success, output, false)
}

pub async fn bucket_inventory(tool_root: &Path) -> ScoopBucketInventory {
    let buckets = load_buckets_from_registry(tool_root).await;
    ScoopBucketInventory {
        kind: "scoop_bucket_list",
        success: true,
        bucket_count: buckets.len(),
        buckets,
    }
}

pub async fn doctor_summary(tool_root: &Path) -> AnyResult<CommandResult> {
    let details = doctor_report(tool_root).await?;
    let mut output = details
        .ensured_paths
        .into_iter()
        .map(|path| format!("Ensured Scoop directory: {path}"))
        .collect::<Vec<_>>();
    output.push(format!(
        "Registered Scoop buckets: {}",
        details.registered_buckets.len()
    ));
    output.push(format!("Scoop state root: {}", details.runtime.state_root));

    Ok(command_result(
        "doctor Scoop runtime",
        CommandStatus::Success,
        output,
        false,
    ))
}

pub async fn doctor_report(tool_root: &Path) -> AnyResult<ScoopDoctorDetails> {
    runtime::doctor_details(tool_root).await
}

pub fn bucket_action_result(
    tool_root: &Path,
    action: &str,
    target_names: &[String],
    result: &CommandResult,
) -> ScoopBucketOperationOutcome {
    let buckets = block_on_sync(load_buckets_from_registry(tool_root));
    ScoopBucketOperationOutcome {
        kind: "scoop_bucket_action",
        action: action.to_string(),
        targets: target_names.to_vec(),
        status: result.status,
        title: result.title.clone(),
        streamed: result.streamed,
        output: result.output.clone(),
        bucket_count: buckets.len(),
        buckets,
    }
}

pub async fn bucket_add(
    tool_root: &Path,
    name: &str,
    source: &str,
    branch: &str,
) -> AnyResult<CommandResult> {
    let spec = BucketSpec {
        name: name.to_string(),
        source: Some(source.to_string()),
        branch: Some(branch.to_string()),
    };
    Ok(
        add_bucket_to_registry_outcome(tool_root, &spec, &configured_proxy())
            .await
            .map(command_result_from_bucket_outcome)?,
    )
}

pub async fn bucket_remove(tool_root: &Path, name: &str) -> AnyResult<CommandResult> {
    Ok(remove_bucket_from_registry_outcome(tool_root, name)
        .await
        .map(command_result_from_bucket_outcome)?)
}

pub async fn bucket_update(tool_root: &Path, names: &[String]) -> AnyResult<CommandResult> {
    Ok(
        update_buckets_outcome(tool_root, names, &configured_proxy())
            .await
            .map(command_result_from_bucket_outcome)?,
    )
}

pub(crate) async fn bucket_update_streaming<F>(
    tool_root: &Path,
    names: &[String],
    mut emit: F,
) -> AnyResult<CommandResult>
where
    F: FnMut(StreamChunk),
{
    let result = update_buckets_outcome(tool_root, names, &configured_proxy()).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    for line in &result.output {
        emit(StreamChunk::Append(line.clone()));
    }
    Ok(command_result_from_bucket_outcome(result))
}
