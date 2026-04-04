use std::path::Path;

use crate::layout::RuntimeLayout;

use super::models::{Bucket, ResolvedBucket};
use super::registry::load_buckets_from_registry;

pub async fn resolve_manifest(tool_root: &Path, package_name: &str) -> Option<ResolvedBucket> {
    let layout = RuntimeLayout::from_root(tool_root);
    let buckets = load_buckets_from_registry(tool_root).await;
    for bucket in buckets {
        let manifest_path = layout
            .scoop
            .bucket_root(&bucket.name)
            .join("bucket")
            .join(format!("{package_name}.json"));
        if tokio::fs::metadata(&manifest_path).await.is_ok() {
            return Some(ResolvedBucket {
                bucket,
                manifest_path,
            });
        }
    }
    None
}

pub fn resolve_manifest_sync(tool_root: &Path, package_name: &str) -> Option<ResolvedBucket> {
    let layout = RuntimeLayout::from_root(tool_root);
    let conn = rusqlite::Connection::open(layout.scoop.db_path()).ok()?;
    let mut stmt = conn
        .prepare("SELECT name, remote_url, branch FROM bucket_registry ORDER BY rowid")
        .ok()?;
    let buckets = stmt
        .query_map([], |row| {
            Ok(Bucket {
                name: row.get(0)?,
                source: row.get(1)?,
                branch: row.get(2)?,
            })
        })
        .ok()?
        .filter_map(|row| row.ok())
        .collect::<Vec<_>>();

    for bucket in buckets {
        let manifest_path = layout
            .scoop
            .bucket_root(&bucket.name)
            .join("bucket")
            .join(format!("{package_name}.json"));
        if manifest_path.exists() {
            return Some(ResolvedBucket {
                bucket,
                manifest_path,
            });
        }
    }
    None
}
