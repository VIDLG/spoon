pub mod model;
pub mod store;

pub use model::{InstalledPackageState, InstalledPackageSummary};
pub use store::{
    list_installed_states, read_installed_state, remove_installed_state, write_installed_state,
};

use crate::Result;
use crate::layout::RuntimeLayout;

pub(crate) async fn commit_installed_state(
    layout: &RuntimeLayout,
    state: &InstalledPackageState,
) -> Result<()> {
    write_installed_state(layout, state).await
}

pub(crate) async fn remove_canonical_installed_state(
    layout: &RuntimeLayout,
    package_name: &str,
) -> Result<()> {
    remove_installed_state(layout, package_name).await
}
