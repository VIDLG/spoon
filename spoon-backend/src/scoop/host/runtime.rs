use std::path::Path;

use async_trait::async_trait;

use crate::Result;
use crate::{BackendContext, BackendEvent, SystemPort};

use super::super::ports::{AppliedIntegration, ScoopIntegrationPort, SupplementalShimSpec};

pub trait ScoopRuntimeHost: SystemPort + ScoopIntegrationPort {}

#[derive(Default)]
pub struct NoopPorts;

impl ScoopRuntimeHost for NoopPorts {}

impl SystemPort for NoopPorts {
    fn ensure_user_path_entry(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn ensure_process_path_entry(&self, _path: &Path) {}

    fn remove_user_path_entry(&self, _path: &Path) -> Result<()> {
        Ok(())
    }

    fn remove_process_path_entry(&self, _path: &Path) {}
}

#[async_trait(?Send)]
impl ScoopIntegrationPort for NoopPorts {
    fn supplemental_shims(
        &self,
        _package_name: &str,
        _current_root: &Path,
    ) -> Vec<SupplementalShimSpec> {
        Vec::new()
    }

    async fn apply_integrations(
        &self,
        _package_name: &str,
        _current_root: &Path,
        _persist_root: &Path,
        _emit: &mut dyn FnMut(BackendEvent),
    ) -> Result<Vec<AppliedIntegration>> {
        Ok(Vec::new())
    }
}

impl<P> ScoopRuntimeHost for BackendContext<P>
where
    P: SystemPort + ScoopIntegrationPort,
{
}

impl<P> SystemPort for BackendContext<P>
where
    P: SystemPort + ScoopIntegrationPort,
{
    fn ensure_user_path_entry(&self, path: &Path) -> Result<()> {
        self.ports.ensure_user_path_entry(path)
    }

    fn ensure_process_path_entry(&self, path: &Path) {
        self.ports.ensure_process_path_entry(path);
    }

    fn remove_user_path_entry(&self, path: &Path) -> Result<()> {
        self.ports.remove_user_path_entry(path)
    }

    fn remove_process_path_entry(&self, path: &Path) {
        self.ports.remove_process_path_entry(path);
    }
}

#[async_trait(?Send)]
impl<P> ScoopIntegrationPort for BackendContext<P>
where
    P: SystemPort + ScoopIntegrationPort,
{
    fn supplemental_shims(
        &self,
        package_name: &str,
        current_root: &Path,
    ) -> Vec<SupplementalShimSpec> {
        self.ports.supplemental_shims(package_name, current_root)
    }

    async fn apply_integrations(
        &self,
        package_name: &str,
        current_root: &Path,
        persist_root: &Path,
        emit: &mut dyn FnMut(BackendEvent),
    ) -> Result<Vec<AppliedIntegration>> {
        self.ports
            .apply_integrations(package_name, current_root, persist_root, emit)
            .await
    }
}
