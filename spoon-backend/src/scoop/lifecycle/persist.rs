use std::path::Path;

use crate::Result;

use super::super::host::persist::{
    restore_persist_entries_into_root as restore_impl,
    sync_persist_entries_from_root as sync_impl,
};
use super::super::models::PersistEntry;

pub(crate) async fn restore_persist_entries(
    install_root: &Path,
    persist_root: &Path,
    entries: &[PersistEntry],
) -> Result<()> {
    restore_impl(install_root, persist_root, entries).await
}

pub(crate) async fn sync_persist_entries(
    install_root: &Path,
    persist_root: &Path,
    entries: &[PersistEntry],
) -> Result<()> {
    sync_impl(install_root, persist_root, entries).await
}
