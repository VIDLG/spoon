use std::path::Path;

use serde::Serialize;
use tokio::fs;

use crate::control_plane::{
    DoctorIssueRecord, list_doctor_issues, sync_failed_lifecycle_issues,
};
use crate::layout::RuntimeLayout;
use crate::{BackendContext, BackendError, Result, SystemPort};

use super::buckets::load_buckets_from_registry;
use super::ports::ScoopIntegrationPort;
use super::host::{
    ContextRuntimeHost, ScoopRuntimeHost, ensure_scoop_shims_activated_with_host,
};

#[derive(Debug, Serialize)]
pub struct ScoopRuntimeDetails {
    pub root: String,
    pub state_root: String,
    pub shims_root: String,
}

#[derive(Debug, Serialize)]
pub struct ScoopDoctorDetails {
    pub kind: &'static str,
    pub success: bool,
    pub runtime: ScoopRuntimeDetails,
    pub ensured_paths: Vec<String>,
    pub shim_activation_output: Vec<String>,
    pub registered_buckets: Vec<super::buckets::Bucket>,
    pub control_plane_issues: Vec<DoctorIssueRecord>,
}

pub async fn doctor_with_host(
    tool_root: &Path,
    proxy: &str,
    host: &dyn ScoopRuntimeHost,
) -> Result<ScoopDoctorDetails> {
    let layout = RuntimeLayout::from_root(tool_root);
    let ensured_paths = vec![
        layout.scoop.root.clone(),
        layout.scoop.apps_root.clone(),
        layout.scoop.buckets_root.clone(),
        layout.scoop.cache_root.clone(),
        layout.scoop.persist_root.clone(),
        layout.scoop.state_root.clone(),
        layout.shims.clone(),
    ];

    for path in &ensured_paths {
        fs::create_dir_all(path)
            .await
            .map_err(|err| BackendError::fs("create", path, err))?;
    }

    super::buckets::ensure_main_bucket_ready(tool_root, proxy).await?;
    let shim_activation_output = ensure_scoop_shims_activated_with_host(tool_root, host).await?;

    sync_failed_lifecycle_issues(&layout).await?;
    let persisted_issues = list_doctor_issues(&layout).await?;
    let has_unresolved_persisted = persisted_issues.iter().any(|issue| !issue.resolved);

    Ok(ScoopDoctorDetails {
        kind: "scoop_doctor",
        success: !has_unresolved_persisted,
        runtime: ScoopRuntimeDetails {
            root: layout.scoop.root.display().to_string(),
            state_root: layout.scoop.state_root.display().to_string(),
            shims_root: layout.shims.display().to_string(),
        },
        ensured_paths: ensured_paths
            .into_iter()
            .map(|path| path.display().to_string())
            .collect(),
        shim_activation_output,
        registered_buckets: load_buckets_from_registry(tool_root).await,
        control_plane_issues: persisted_issues,
    })
}

pub async fn doctor_with_context<P>(context: &BackendContext<P>) -> Result<ScoopDoctorDetails>
where
    P: SystemPort + ScoopIntegrationPort,
{
    let host = ContextRuntimeHost::new(context);
    doctor_with_host(&context.root, context.proxy.as_deref().unwrap_or(""), &host).await
}
