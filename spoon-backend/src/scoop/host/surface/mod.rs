mod activation;
mod environment;
mod reapply;
mod shims;
mod shortcuts;
mod validation;

pub use reapply::reapply_package_command_surface;
pub use activation::ensure_scoop_shims_activated_with_host;
pub use shims::{expanded_shim_targets, remove_shims};
pub use shortcuts::remove_shortcuts;
pub use validation::{installed_targets_exist, installer_layout_error};
pub(crate) use shims::write_shims;
pub(crate) use shortcuts::write_shortcuts;
