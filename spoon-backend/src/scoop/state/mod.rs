pub mod model;
pub mod projections;
pub mod store;

pub use model::InstalledPackageState;
pub use projections::{
    InstalledPackageSummary, installed_package_summary, list_all_installed_states, list_installed_states_filtered,
    list_installed_summaries, list_installed_summaries_filtered,
};
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
