use std::path::Path;

use crate::Result;
use crate::{BackendError, BackendEvent};
use crate::db::Db;
use crate::layout::RuntimeLayout;
use crate::scoop::{resolve_package_source, ResolvedPackageSource};
use crate::scoop::state::{read_installed_state, write_installed_state};

use crate::scoop::manifest;

use super::super::ScoopRuntimeHost;
use super::shims::{remove_shims, write_shims};

pub async fn reapply_package_command_surface(
    tool_root: &Path,
    package_name: &str,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<()> {
    let layout = RuntimeLayout::from_root(tool_root);
    let db = Db::open(&layout.scoop.db_path()).await?;
    let Some(mut state) = read_installed_state(&db, package_name).await else {
        tracing::info!(
            "Skipped command-surface reapply for '{}': installed state was not found.",
            package_name
        );
        return Ok(());
    };
    let current_root = layout.scoop.package_current_root(package_name);
    if !current_root.exists() {
        tracing::info!(
            "Skipped command-surface reapply for '{}': current install root was not found.",
            package_name
        );
        return Ok(());
    }
    let resolved = manifest::resolve_package_manifest(package_name, tool_root)
        .await
        .ok_or(BackendError::ManifestUnavailable)?;
    let manifest = manifest::load_manifest_value(&resolved.manifest_path).await?;
    let source: ResolvedPackageSource = resolve_package_source(&manifest)?;
    remove_shims(tool_root, &state.command_surface.bins).await?;
    let shims_root = layout.shims.clone();
    let persist_root = layout.scoop.package_persist_root(package_name);
    let aliases = write_shims(
        package_name,
        &shims_root,
        &current_root,
        &persist_root,
        &source,
        host,
        emit,
    )
    .await?;
    state.command_surface.bins = aliases.clone();
    state.command_surface.env_add_path = source.env_add_path.clone();
    state.command_surface.env_set = source.env_set.clone();
    write_installed_state(&db, &state).await?;
    tracing::info!("Reapplied command surface for '{}'.", package_name);
    tracing::info!("Managed shims: {}", aliases.join(", "));
    Ok(())
}
