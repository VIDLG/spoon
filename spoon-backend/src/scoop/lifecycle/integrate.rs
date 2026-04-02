use std::collections::BTreeMap;
use std::path::Path;

use crate::{BackendEvent, Result};

use super::super::host::{ScoopRuntimeHost, apply_package_integrations};

pub(crate) async fn run_integrations(
    host: &dyn ScoopRuntimeHost,
    package_name: &str,
    current_root: &Path,
    persist_root: &Path,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<BTreeMap<String, String>> {
    apply_package_integrations(host, package_name, current_root, persist_root, emit).await
}
