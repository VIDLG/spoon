use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ScoopError;

/// Re-export of spoon_core::ScoopLayout for convenience.
pub use spoon_core::ScoopLayout;

/// A registered scoop bucket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Bucket {
    pub name: String,
    pub source: String,
    pub branch: String,
}

/// Specification for adding a new bucket.
#[derive(Debug, Clone)]
pub struct BucketSpec {
    pub name: String,
    pub source: Option<String>,
    pub branch: Option<String>,
}

impl BucketSpec {
    pub fn resolve(&self) -> crate::Result<Bucket> {
        let source = match &self.source {
            Some(s) => s.clone(),
            None => known_bucket_source(&self.name)
                .ok_or_else(|| ScoopError::Config(format!("unknown bucket: {}", self.name)))?,
        };
        let branch = self.branch.clone().unwrap_or_else(|| "master".to_string());
        Ok(Bucket {
            name: self.name.clone(),
            source,
            branch,
        })
    }
}

/// A bucket with its resolved manifest directory path.
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

const KNOWN_BUCKETS: &[(&str, &str)] = &[
    ("main", "https://github.com/ScoopInstaller/Main"),
    ("extras", "https://github.com/ScoopInstaller/Extras"),
    ("versions", "https://github.com/ScoopInstaller/Versions"),
    ("nirsoft", "https://github.com/ScoopInstaller/Nirsoft"),
    ("sysinternals", "https://github.com/niheaven/scoop-sysinternals"),
    ("php", "https://github.com/ScoopInstaller/PHP"),
    ("nerd-fonts", "https://github.com/matthewjberger/scoop-nerd-fonts"),
    ("nonportable", "https://github.com/ScoopInstaller/Nonportable"),
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

// ── Bucket registry (JSON-based, no SQLite) ──

use std::path::Path;

/// Registry file name relative to scoop root.
const REGISTRY_FILE: &str = "buckets.json";

/// Load buckets from JSON registry file.
pub async fn load_buckets_from_registry(scoop_root: &Path) -> Vec<Bucket> {
    let registry_path = scoop_root.join(REGISTRY_FILE);
    if !registry_path.exists() {
        return Vec::new();
    }
    let content = tokio::fs::read_to_string(&registry_path).await.unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

/// Save buckets to JSON registry file.
async fn save_buckets_to_registry(scoop_root: &Path, buckets: &[Bucket]) -> crate::Result<()> {
    let registry_path = scoop_root.join(REGISTRY_FILE);
    let content = serde_json::to_string_pretty(buckets).map_err(ScoopError::from)?;
    tokio::fs::write(&registry_path, content)
        .await
        .map_err(|e| ScoopError::fs("write", &registry_path, e))?;
    Ok(())
}

/// Upsert a bucket to the registry.
pub async fn upsert_bucket_to_registry(scoop_root: &Path, spec: &BucketSpec) -> crate::Result<Vec<Bucket>> {
    let bucket = spec.resolve()?;
    let mut buckets = load_buckets_from_registry(scoop_root).await;

    // Update existing or add new
    if let Some(existing) = buckets.iter_mut().find(|b| b.name == bucket.name) {
        existing.source = bucket.source;
        existing.branch = bucket.branch;
    } else {
        buckets.push(bucket);
    }

    save_buckets_to_registry(scoop_root, &buckets).await?;
    Ok(buckets)
}

/// Remove a bucket from the registry.
pub async fn remove_bucket_from_registry(scoop_root: &Path, name: &str) -> crate::Result<Vec<Bucket>> {
    let mut buckets = load_buckets_from_registry(scoop_root).await;
    buckets.retain(|b| !b.name.eq_ignore_ascii_case(name));
    save_buckets_to_registry(scoop_root, &buckets).await?;
    Ok(buckets)
}

/// Ensure main bucket is registered.
pub async fn ensure_main_bucket_ready(scoop_root: &Path) -> crate::Result<()> {
    let buckets = load_buckets_from_registry(scoop_root).await;
    if !buckets.iter().any(|b| b.name == "main") {
        upsert_bucket_to_registry(scoop_root, &BucketSpec {
            name: "main".to_string(),
            source: None,
            branch: None,
        }).await?;
    }
    Ok(())
}
