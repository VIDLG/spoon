use crate::Result;
use crate::layout::RuntimeLayout;
use crate::scoop::state::{
    InstalledPackageState, remove_installed_state as remove_impl, write_installed_state as write_impl,
};

pub(crate) async fn commit_installed_state(
    layout: &RuntimeLayout,
    state: &InstalledPackageState,
) -> Result<()> {
    write_impl(layout, state).await
}

pub(crate) async fn remove_installed_state(
    layout: &RuntimeLayout,
    package_name: &str,
) -> Result<()> {
    remove_impl(layout, package_name).await
}
