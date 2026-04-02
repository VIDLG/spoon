use crate::{BackendEvent, Result};
use crate::layout::RuntimeLayout;

use super::super::actions;
use super::super::host::ScoopRuntimeHost;

pub(crate) async fn uninstall(
    tool_root: &std::path::Path,
    operation_id: i64,
    package_name: &str,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    let layout = RuntimeLayout::from_root(tool_root);
    actions::uninstall_package(tool_root, &layout, operation_id, package_name, host, emit).await
}
