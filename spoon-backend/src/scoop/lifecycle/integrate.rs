use std::path::Path;

use crate::{BackendEvent, Result};

use super::super::host::ScoopRuntimeHost;
use super::super::ports::AppliedIntegration;

pub(crate) async fn run_integrations(
    host: &dyn ScoopRuntimeHost,
    package_name: &str,
    current_root: &Path,
    persist_root: &Path,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<AppliedIntegration>> {
    host.apply_integrations(package_name, current_root, persist_root, emit)
        .await
}
