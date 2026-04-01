//! Internal integration tests for spoon-backend.
//!
//! These tests exercise the backend's internal APIs without requiring
//! external services or a full system setup.

mod context;
mod control_plane;
mod event;
mod fsx;
mod platform;
mod proxy;
mod task;

/// Common utilities for writing tests.
use std::path::PathBuf;

/// Run an async future on a current-thread runtime.
pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}

/// Create a unique temporary directory for testing.
pub fn temp_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "spoon-{}-{}-{}",
        name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
