//! Common utilities for integration tests.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::Once;

static TEST_ENV_INIT: Once = Once::new();

fn load_test_env() {
    TEST_ENV_INIT.call_once(|| {
        let _ = dotenvy::from_filename(".env.test.local");
        let _ = dotenvy::from_filename(".env.local");
        let _ = dotenvy::dotenv();
    });
}

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

pub fn test_proxy() -> String {
    load_test_env();
    std::env::var("SPOON_TEST_PROXY").unwrap_or_default()
}
