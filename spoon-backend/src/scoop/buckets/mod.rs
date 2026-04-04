mod manifest;
mod models;
mod operations;
mod registry;
mod update;

pub use manifest::{resolve_manifest, resolve_manifest_sync};
pub use models::{
    Bucket, BucketSpec, BucketUpdateSummary, ResolvedBucket, ScoopBucketInventory,
    ScoopBucketOperationOutcome, known_bucket_source,
};
pub use operations::{
    add_bucket_to_registry, add_bucket_to_registry_outcome, add_bucket_to_registry_with_context,
    ensure_main_bucket_ready, ensure_main_bucket_ready_with_context, remove_bucket_from_registry,
    remove_bucket_from_registry_outcome,
};
pub use registry::{
    load_buckets_from_registry, remove_bucket_from_registry_record, sync_main_bucket_registry,
    upsert_bucket_to_registry,
};
pub use update::{
    update_buckets, update_buckets_outcome, update_buckets_streaming,
    update_buckets_streaming_outcome, update_buckets_streaming_outcome_with_context,
    update_buckets_streaming_with_context, update_buckets_with_context,
};
