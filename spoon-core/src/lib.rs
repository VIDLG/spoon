//! spoon-core — shared infrastructure for the Spoon toolkit.
//!
//! Provides error types, layout paths, event channels, cancellation tokens,
//! HTTP downloads, git operations, archive extraction, and filesystem utilities.

mod archive;
mod cancellation;
mod download;
mod error;
mod events;
mod formatting;
mod fsx;
mod gitx;
pub mod layout;
pub mod proxy;

pub use archive::extract_zip_archive_sync;
pub use cancellation::{
    CancellationToken, TaskCancellation, check_token_cancel, is_token_cancelled,
    spawn_interrupt_on_cancel,
};
pub use download::{
    copy_or_download_to_file, hash_matches, materialize_to_file_with_hash,
};
pub use error::{CoreError, Result};
pub use events::{
    SpoonEvent, CommandStatus, EventReceiver, EventSender, FinishEvent, LifecycleStage,
    NoticeEvent, NoticeLevel, ProgressEvent, ProgressKind, ProgressState, ProgressUnit,
    StageEvent, event_bus, progress_kind,
};
pub use formatting::format_bytes;
pub use fsx::{copy_path_recursive, directory_size};
pub use gitx::{RepoSyncOutcome, clone_repo};
pub use layout::{
    ManagedMsvcLayout, MsvcLayout, OfficialMsvcLayout, RuntimeLayout, ScoopLayout,
};
pub use proxy::{ReqwestClientBuilder, normalize_proxy_url};

#[cfg(test)]
mod tests;
