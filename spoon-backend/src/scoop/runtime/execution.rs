use std::collections::BTreeMap;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;

use tokio::fs;

use crate::Result;
use crate::{BackendContext, BackendError, BackendEvent, SystemPort};

use super::super::ports::{ScoopIntegrationPort, SupplementalShimSpec};
use super::super::paths;

pub trait ScoopRuntimeHost {
    fn test_mode_enabled(&self) -> bool;
    fn ensure_user_path_entry(&self, path: &Path) -> Result<()>;
    fn ensure_process_path_entry(&self, path: &Path);
    fn remove_user_path_entry(&self, path: &Path) -> Result<()>;
    fn remove_process_path_entry(&self, path: &Path);
    fn supplemental_shims(
        &self,
        package_name: &str,
        current_root: &Path,
    ) -> Vec<SupplementalShimSpec>;
    fn apply_integrations<'a>(
        &'a self,
        package_name: &'a str,
        current_root: &'a Path,
        persist_root: &'a Path,
        emit: &'a mut dyn FnMut(BackendEvent),
    ) -> Pin<Box<dyn Future<Output = Result<BTreeMap<String, String>>> + 'a>>;
}

#[derive(Default)]
pub struct NoopScoopRuntimeHost;

impl ScoopRuntimeHost for NoopScoopRuntimeHost {
    fn test_mode_enabled(&self) -> bool {
        false
    }

    fn ensure_user_path_entry(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn ensure_process_path_entry(&self, _path: &Path) {}

    fn remove_user_path_entry(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn remove_process_path_entry(&self, _path: &Path) {}

    fn supplemental_shims(
        &self,
        _package_name: &str,
        _current_root: &Path,
    ) -> Vec<SupplementalShimSpec> {
        Vec::new()
    }

    fn apply_integrations<'a>(
        &'a self,
        _package_name: &'a str,
        _current_root: &'a Path,
        _persist_root: &'a Path,
        _emit: &'a mut dyn FnMut(BackendEvent),
    ) -> Pin<Box<dyn Future<Output = Result<BTreeMap<String, String>>> + 'a>> {
        Box::pin(async { Ok(BTreeMap::new()) })
    }
}

pub(crate) struct ContextRuntimeHost<'a, P> {
    context: &'a BackendContext<P>,
}

impl<'a, P> ContextRuntimeHost<'a, P> {
    pub(crate) fn new(context: &'a BackendContext<P>) -> Self {
        Self { context }
    }
}

impl<P> ScoopRuntimeHost for ContextRuntimeHost<'_, P>
where
    P: SystemPort + ScoopIntegrationPort,
{
    fn test_mode_enabled(&self) -> bool {
        self.context.test_mode
    }

    fn ensure_user_path_entry(&self, path: &Path) -> Result<()> {
        self.context.ports.ensure_user_path_entry(path)
    }

    fn ensure_process_path_entry(&self, path: &Path) {
        self.context.ports.ensure_process_path_entry(path);
    }

    fn remove_user_path_entry(&self, path: &Path) -> Result<()> {
        self.context.ports.remove_user_path_entry(path)
    }

    fn remove_process_path_entry(&self, path: &Path) {
        self.context.ports.remove_process_path_entry(path);
    }

    fn supplemental_shims(
        &self,
        package_name: &str,
        current_root: &Path,
    ) -> Vec<SupplementalShimSpec> {
        self.context
            .ports
            .supplemental_shims(package_name, current_root)
    }

    fn apply_integrations<'a>(
        &'a self,
        package_name: &'a str,
        current_root: &'a Path,
        persist_root: &'a Path,
        emit: &'a mut dyn FnMut(BackendEvent),
    ) -> Pin<Box<dyn Future<Output = Result<BTreeMap<String, String>>> + 'a>> {
        self.context
            .ports
            .apply_integrations(package_name, current_root, persist_root, emit)
    }
}

async fn remove_old_scoop_shims(
    tool_root: &Path,
    host: &dyn ScoopRuntimeHost,
) -> Result<Vec<String>> {
    let old_root = paths::scoop_root(tool_root).join("shims");
    let mut lines = Vec::new();
    if old_root.exists() {
        fs::remove_dir_all(&old_root).await.map_err(|err| {
            BackendError::Other(format!(
                "failed to remove old shim root {}: {err}",
                old_root.display()
            ))
        })?;
        lines.push(format!(
            "Removed old Scoop shim root: {}",
            old_root.display()
        ));
    }
    let _ = host.remove_user_path_entry(&old_root);
    host.remove_process_path_entry(&old_root);
    Ok(lines)
}

pub async fn ensure_scoop_shims_activated_with_host(
    tool_root: &Path,
    host: &dyn ScoopRuntimeHost,
) -> Result<Vec<String>> {
    let mut output = remove_old_scoop_shims(tool_root, host).await?;
    let shims_root = paths::shims_root(tool_root);
    fs::create_dir_all(&shims_root)
        .await
        .map_err(|err| BackendError::fs("create", &shims_root, err))?;
    host.ensure_user_path_entry(&shims_root)?;
    host.ensure_process_path_entry(&shims_root);
    output.push(format!(
        "Ensured Spoon shims are available on PATH: {}",
        shims_root.display()
    ));
    Ok(output)
}

pub async fn ensure_scoop_shims_activated(tool_root: &Path) -> Result<Vec<String>> {
    let host = NoopScoopRuntimeHost;
    ensure_scoop_shims_activated_with_host(tool_root, &host).await
}

pub async fn ensure_scoop_shims_activated_with_context<P>(
    context: &BackendContext<P>,
) -> Result<Vec<String>>
where
    P: SystemPort + ScoopIntegrationPort,
{
    let host = ContextRuntimeHost::new(context);
    ensure_scoop_shims_activated_with_host(&context.root, &host).await
}
