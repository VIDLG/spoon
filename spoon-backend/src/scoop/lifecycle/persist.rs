use std::path::Path;

use crate::{BackendEvent, Result};

use super::super::runtime::PersistEntry;
use super::super::runtime::persist::{
    restore_persist_entries_into_root as restore_impl,
    sync_persist_entries_from_root as sync_impl,
};

pub(crate) async fn restore_persist_entries(
    install_root: &Path,
    persist_root: &Path,
    entries: &[PersistEntry],
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<()> {
    restore_impl(install_root, persist_root, entries, emit).await
}

pub(crate) async fn sync_persist_entries(
    install_root: &Path,
    persist_root: &Path,
    entries: &[PersistEntry],
    emit: &mut dyn FnMut(BackendEvent),
) -> Result<()> {
    sync_impl(install_root, persist_root, entries, emit).await
}
