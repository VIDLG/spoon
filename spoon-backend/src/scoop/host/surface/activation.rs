use std::path::Path;

use tokio::fs;

use crate::Result;
use crate::BackendError;
use crate::layout::RuntimeLayout;

use super::super::ScoopRuntimeHost;

pub async fn ensure_scoop_shims_activated_with_host(
    tool_root: &Path,
    host: &(dyn ScoopRuntimeHost + '_),
) -> Result<()> {
    let shims_root = RuntimeLayout::from_root(tool_root).shims;
    fs::create_dir_all(&shims_root)
        .await
        .map_err(|err| BackendError::fs("create", &shims_root, err))?;
    host.ensure_user_path_entry(&shims_root)?;
    host.ensure_process_path_entry(&shims_root);
    tracing::info!(
        "Ensured Spoon shims are available on PATH: {}",
        shims_root.display()
    );
    Ok(())
}
