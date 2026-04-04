use std::path::Path;

use crate::{BackendEvent, Result};
use crate::db::Db;
use crate::layout::RuntimeLayout;
use crate::scoop::AppliedIntegration;
use crate::scoop::state::{read_installed_state, write_installed_state};
use super::ScoopRuntimeHost;

pub async fn reapply_package_integrations(
    tool_root: &Path,
    package_name: &str,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<()> {
    let layout = RuntimeLayout::from_root(tool_root);
    let db = Db::open(&layout.scoop.db_path()).await?;
    let Some(mut state) = read_installed_state(&db, package_name).await else {
        tracing::info!(
            "Skipped integration reapply for '{}': installed state was not found.",
            package_name
        );
        return Ok(());
    };
    let current_root = layout.scoop.package_current_root(package_name);
    if !current_root.exists() {
        tracing::info!(
            "Skipped integration reapply for '{}': current install root was not found.",
            package_name
        );
        return Ok(());
    }
    let persist_root = layout.scoop.package_persist_root(package_name);
    let integrations = host
        .apply_integrations(package_name, &current_root, &persist_root, emit)
        .await?;
    state.integrations = integrations.clone();
    write_installed_state(&db, &state).await?;
    if integrations.is_empty() {
        tracing::info!("Reapplied integrations for '{}': no changes.", package_name);
    } else {
        tracing::info!("Reapplied integrations for '{}'.", package_name);
        for AppliedIntegration { key, value } in &integrations {
            tracing::info!("Applied integration: {key} = {value}");
        }
    }
    Ok(())
}
