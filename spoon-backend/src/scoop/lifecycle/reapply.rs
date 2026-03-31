use crate::{BackendEvent, Result};

use super::super::runtime::{
    ScoopRuntimeHost, reapply_package_command_surface_streaming_with_host,
    reapply_package_integrations_streaming_with_host,
};

pub(crate) async fn reapply(
    tool_root: &std::path::Path,
    package_name: &str,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<Vec<String>> {
    let mut output = reapply_package_command_surface_streaming_with_host(
        tool_root,
        package_name,
        host,
        emit,
    )
    .await?;
    output.extend(
        reapply_package_integrations_streaming_with_host(tool_root, package_name, host, emit)
            .await?,
    );
    Ok(output)
}
