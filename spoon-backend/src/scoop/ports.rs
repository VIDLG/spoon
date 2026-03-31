use std::collections::BTreeMap;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;

use crate::{BackendEvent, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupplementalShimSpec {
    pub alias: String,
    pub relative_path: String,
}

/// App-owned Scoop integration callbacks invoked by backend lifecycle code.
///
/// These ports stay defined in `spoon-backend` because the backend decides
/// when lifecycle steps need host-owned package integrations, but the actual
/// implementation belongs to the app shell.
pub trait ScoopIntegrationPort {
    fn supplemental_shims(
        &self,
        package_name: &str,
        current_root: &Path,
    ) -> Vec<SupplementalShimSpec>;

    fn apply_integrations<'a>(
        &'a self,
        package_name: &'a str,
        current_root: &'a Path,
        persist_root: &'a Path,
        emit: &'a mut dyn FnMut(BackendEvent),
    ) -> Pin<Box<dyn Future<Output = Result<BTreeMap<String, String>>> + 'a>>;
}
