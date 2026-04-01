use std::path::Path;

use crate::BackendContext;

pub use super::status::{
    MsvcIntegration, MsvcStatus, installed_toolchain_version_label,
    latest_toolchain_version_label, latest_toolchain_version_label_with_context, user_facing_toolchain_label,
};

pub async fn status(tool_root: &Path) -> MsvcStatus {
    super::status::status(tool_root).await
}

pub async fn status_with_context<P>(context: &BackendContext<P>) -> MsvcStatus {
    super::status::status_with_context(context).await
}
