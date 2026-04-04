use std::path::Path;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::{BackendEvent, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupplementalShimSpec {
    pub alias: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AppliedIntegration {
    pub key: String,
    pub value: String,
}

/// App-owned Scoop integration callbacks invoked by backend lifecycle code.
///
/// These ports stay defined in `spoon-backend` because the backend decides
/// when lifecycle steps need host-owned package integrations, but the actual
/// implementation belongs to the app shell.
#[async_trait(?Send)]
pub trait ScoopIntegrationPort {
    fn supplemental_shims(
        &self,
        package_name: &str,
        current_root: &Path,
    ) -> Vec<SupplementalShimSpec>;

    async fn apply_integrations(
        &self,
        package_name: &str,
        current_root: &Path,
        persist_root: &Path,
        emit: &mut dyn FnMut(BackendEvent),
    ) -> Result<Vec<AppliedIntegration>>;
}
