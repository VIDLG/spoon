use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BackendError, CommandStatus, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Bucket {
    pub name: String,
    pub source: String,
    pub branch: String,
}

#[derive(Debug, Clone)]
pub struct BucketSpec {
    pub name: String,
    pub source: Option<String>,
    pub branch: Option<String>,
}

impl BucketSpec {
    /// Resolve a partial spec to a complete bucket.
    /// Source is filled from known buckets if missing.
    /// Branch defaults to "master" if missing.
    pub fn resolve(&self) -> Result<Bucket> {
        let source = match &self.source {
            Some(s) => s.clone(),
            None => known_bucket_source(&self.name)
                .ok_or_else(|| BackendError::Config(format!("unknown bucket: {}", self.name)))?,
        };
        let branch = self.branch.clone().unwrap_or_else(|| "master".to_string());
        Ok(Bucket {
            name: self.name.clone(),
            source,
            branch,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedBucket {
    pub bucket: Bucket,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BucketUpdateSummary {
    pub updated: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoopBucketInventory {
    pub kind: &'static str,
    pub success: bool,
    pub bucket_count: usize,
    pub buckets: Vec<Bucket>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoopBucketOperationOutcome {
    pub kind: &'static str,
    pub action: String,
    pub targets: Vec<String>,
    pub status: CommandStatus,
    pub title: String,
    pub streamed: bool,
    pub output: Vec<String>,
    pub bucket_count: usize,
    pub buckets: Vec<Bucket>,
}

impl ScoopBucketOperationOutcome {
    pub const fn is_success(&self) -> bool {
        self.status.is_success()
    }
}

const KNOWN_BUCKETS: &[(&str, &str)] = &[
    ("main", "https://github.com/ScoopInstaller/Main"),
    ("extras", "https://github.com/ScoopInstaller/Extras"),
    ("versions", "https://github.com/ScoopInstaller/Versions"),
    ("nirsoft", "https://github.com/ScoopInstaller/Nirsoft"),
    (
        "sysinternals",
        "https://github.com/niheaven/scoop-sysinternals",
    ),
    ("php", "https://github.com/ScoopInstaller/PHP"),
    (
        "nerd-fonts",
        "https://github.com/matthewjberger/scoop-nerd-fonts",
    ),
    (
        "nonportable",
        "https://github.com/ScoopInstaller/Nonportable",
    ),
    ("java", "https://github.com/ScoopInstaller/Java"),
    ("games", "https://github.com/Calinou/scoop-games"),
];

pub fn known_bucket_source(name: &str) -> Option<String> {
    let env_var = format!(
        "SPOON_TEST_SCOOP_BUCKET_{}_SOURCE",
        name.to_ascii_uppercase().replace('-', "_")
    );
    std::env::var(&env_var).ok().or_else(|| {
        KNOWN_BUCKETS
            .iter()
            .find(|(known, _)| known.eq_ignore_ascii_case(name))
            .map(|(_, source)| (*source).to_string())
    })
}
