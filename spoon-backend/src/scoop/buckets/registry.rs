use std::path::Path;

use rusqlite::params;

use crate::db::Db;
use crate::layout::RuntimeLayout;
use crate::Result;

use super::models::{Bucket, BucketSpec};

pub async fn load_buckets_from_registry(tool_root: &Path) -> Vec<Bucket> {
    let layout = RuntimeLayout::from_root(tool_root);
    let Ok(db) = Db::open(&layout.scoop.db_path()).await else {
        return Vec::new();
    };
    db.call(|conn| {
        let mut stmt = conn.prepare(
            "SELECT name, remote_url, branch FROM bucket_registry ORDER BY rowid",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Bucket {
                name: row.get(0)?,
                source: row.get(1)?,
                branch: row.get(2)?,
            })
        })?;
        Ok(rows.filter_map(|row| row.ok()).collect::<Vec<_>>())
    })
    .await
    .unwrap_or_default()
}

pub async fn upsert_bucket_to_registry(tool_root: &Path, spec: &BucketSpec) -> Result<Vec<Bucket>> {
    let bucket = spec.resolve()?;
    let layout = RuntimeLayout::from_root(tool_root);
    let db = Db::open(&layout.scoop.db_path()).await?;
    let bucket_name = bucket.name.clone();
    let source = bucket.source.clone();
    let branch = bucket.branch.clone();
    db.call(move |conn| {
        conn.execute(
            "INSERT INTO bucket_registry (name, remote_url, branch)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(name) DO UPDATE SET
                remote_url = excluded.remote_url,
                branch = excluded.branch",
            params![bucket_name, source, branch],
        )?;
        Ok(())
    })
    .await?;
    Ok(load_buckets_from_registry(tool_root).await)
}

pub async fn remove_bucket_from_registry_record(
    tool_root: &Path,
    name: &str,
) -> Result<Vec<Bucket>> {
    let layout = RuntimeLayout::from_root(tool_root);
    let db = Db::open(&layout.scoop.db_path()).await?;
    let name = name.to_string();
    db.call(move |conn| {
        conn.execute(
            "DELETE FROM bucket_registry WHERE lower(name) = lower(?1)",
            params![name],
        )?;
        Ok(())
    })
    .await?;
    Ok(load_buckets_from_registry(tool_root).await)
}

pub async fn sync_main_bucket_registry(tool_root: &Path) -> Result<()> {
    upsert_bucket_to_registry(
        tool_root,
        &BucketSpec {
            name: "main".to_string(),
            source: None,
            branch: None,
        },
    )
    .await?;
    Ok(())
}
