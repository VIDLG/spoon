use std::path::Path;

use crate::{BackendEvent, Result};

use super::super::runtime::{ScoopRuntimeHost, SelectedPackageSource, ShortcutEntry};
use super::super::runtime::surface::{
    remove_shims as remove_shims_impl, remove_shortcuts as remove_shortcuts_impl,
    write_shortcuts as write_shortcuts_impl, write_shims as write_shims_impl,
};

pub(crate) async fn apply_install_surface(
    package_name: &str,
    shims_root: &Path,
    install_root: &Path,
    persist_root: &Path,
    source: &SelectedPackageSource,
    host: &dyn ScoopRuntimeHost,
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<(Vec<String>, Vec<ShortcutEntry>)> {
    let aliases = write_shims_impl(
        package_name,
        shims_root,
        install_root,
        persist_root,
        source,
        host,
        emit,
    )
    .await?;
    let shortcuts = write_shortcuts_impl(install_root, persist_root, source, host, emit).await?;
    Ok((aliases, shortcuts))
}

pub(crate) async fn remove_surface(
    tool_root: &Path,
    bins: &[String],
    shortcuts: &[ShortcutEntry],
    host: &dyn ScoopRuntimeHost,
) -> Result<()> {
    remove_shims_impl(tool_root, bins).await?;
    remove_shortcuts_impl(shortcuts, host).await
}
