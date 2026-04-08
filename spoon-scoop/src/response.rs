use schemars::JsonSchema;
use serde::Serialize;

use crate::{Bucket, InstalledPackageSummary};

// ── Runtime status response ──

#[derive(Debug, Serialize, JsonSchema)]
pub struct ScoopRuntimeStatus {
    pub root: String,
    pub shims: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ScoopPaths {
    pub apps: String,
    pub cache: String,
    pub persist: String,
    pub state: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ScoopStatus {
    pub kind: &'static str,
    pub success: bool,
    pub runtime: ScoopRuntimeStatus,
    pub buckets: Vec<Bucket>,
    pub installed_packages: Vec<InstalledPackageSummary>,
    pub paths: ScoopPaths,
}

// ── Search response ──

#[derive(Debug, Serialize, JsonSchema)]
pub struct ScoopSearchMatch {
    pub package_name: String,
    pub bucket: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ScoopSearchResults {
    pub kind: &'static str,
    pub success: bool,
    pub query: Option<String>,
    pub matches: Vec<ScoopSearchMatch>,
}

// ── Package info response ──

#[derive(Debug, Serialize)]
pub struct ScoopPackageMetadata {
    pub name: String,
    pub bucket: String,
    pub latest_version: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub manifest: String,
    pub license: Option<String>,
    pub depends: Option<serde_json::Value>,
    pub suggest: Option<serde_json::Value>,
    pub extract_dir: Option<serde_json::Value>,
    pub extract_to: Option<serde_json::Value>,
    pub notes: Vec<String>,
    pub download_urls: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageInstall {
    pub installed: bool,
    pub installed_version: Option<String>,
    pub current: String,
    pub installed_size_bytes: Option<u64>,
    pub cache_size_bytes: Option<u64>,
    pub bins: Vec<String>,
    pub state: Option<String>,
    pub persist_root: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageDetails<D> {
    pub kind: &'static str,
    pub success: bool,
    pub package: ScoopPackageMetadata,
    pub install: ScoopPackageInstall,
    pub integration: ScoopPackageIntegration<D>,
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageError {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageDetailsError {
    pub kind: &'static str,
    pub success: bool,
    pub package: String,
    pub error: ScoopPackageError,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ScoopPackageDetailsOutcome<D> {
    Details(ScoopPackageDetails<D>),
    Error(ScoopPackageDetailsError),
}

// ── Integration details ──

#[derive(Debug, Serialize)]
pub struct ScoopCommandIntegration {
    pub shims: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ScoopEnvironmentIntegration {
    pub add_path: Vec<String>,
    pub set: Vec<String>,
    pub persist: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ScoopSystemIntegration {
    pub shortcuts: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ScoopPolicyAppliedValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct ScoopPolicyIntegration<D> {
    pub desired: Vec<D>,
    pub applied_values: Vec<ScoopPolicyAppliedValue>,
    pub config_files: Vec<String>,
    pub config_directories: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ScoopPackageIntegration<D> {
    pub commands: ScoopCommandIntegration,
    pub environment: ScoopEnvironmentIntegration,
    pub system: ScoopSystemIntegration,
    pub policy: ScoopPolicyIntegration<D>,
}

// ── Manifest response ──

#[derive(Debug, Serialize)]
pub struct ScoopPackageManifestOutcome {
    pub kind: &'static str,
    pub package: String,
    pub status: spoon_core::CommandStatus,
    pub title: String,
    pub manifest_path: Option<String>,
    pub content: Option<String>,
    pub error: Option<ScoopPackageError>,
}

impl ScoopPackageManifestOutcome {
    pub const fn is_success(&self) -> bool {
        self.status.is_success()
    }
}

// ── Operation outcomes ──

/// Package reference for action/operation outcomes.
#[derive(Debug, Clone, Serialize)]
pub struct ScoopActionPackage {
    pub name: String,
    pub display_name: String,
}

/// Install state summary for action/operation outcomes.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ScoopPackageInstallState {
    pub installed: bool,
    pub installed_version: Option<String>,
    pub current: Option<String>,
}

/// Outcome of a package action (for TUI display).
#[derive(Debug, Serialize)]
pub struct ScoopPackageActionOutcome {
    pub kind: &'static str,
    pub action: String,
    pub package: ScoopActionPackage,
    pub success: bool,
    pub title: String,
    pub state: ScoopPackageInstallState,
}

/// Outcome of a package operation (for streaming/CLI).
#[derive(Debug, Serialize)]
pub struct ScoopPackageOperationOutcome {
    pub kind: &'static str,
    pub action: String,
    pub package: ScoopActionPackage,
    pub status: spoon_core::CommandStatus,
    pub title: String,
    pub state: ScoopPackageInstallState,
}

impl ScoopPackageOperationOutcome {
    pub const fn is_success(&self) -> bool {
        self.status.is_success()
    }
}

// ── Bucket operation outcomes ──

/// Outcome of a bucket operation (add/remove/update).
#[derive(Debug, Clone, Serialize)]
pub struct ScoopBucketOperationOutcome {
    pub kind: &'static str,
    pub action: String,
    pub targets: Vec<String>,
    pub status: spoon_core::CommandStatus,
    pub title: String,
    pub bucket_count: usize,
    pub buckets: Vec<Bucket>,
}

/// Outcome of a doctor operation.
#[derive(Debug, Clone, Serialize)]
pub struct ScoopDoctorDetails {
    pub kind: &'static str,
    pub success: bool,
    pub runtime: ScoopRuntimeDetails,
    pub ensured_paths: Vec<String>,
    pub registered_buckets: Vec<Bucket>,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoopRuntimeDetails {
    pub root: String,
    pub state_root: String,
    pub shims_root: String,
}