use std::sync::{Mutex, OnceLock};

use anyhow::{Context, Result};

static TEST_MODE: OnceLock<bool> = OnceLock::new();
static TEST_CLIPBOARD: OnceLock<Mutex<Option<String>>> = OnceLock::new();

pub fn enable_test_mode() {
    let _ = TEST_MODE.set(true);
}

pub fn write_text(text: &str) -> Result<()> {
    if *TEST_MODE.get_or_init(|| false) {
        *TEST_CLIPBOARD
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(text.to_string());
        return Ok(());
    }

    let mut clipboard = arboard::Clipboard::new().context("failed to access system clipboard")?;
    clipboard
        .set_text(text)
        .context("failed to write text to system clipboard")
}

pub fn test_contents() -> Option<String> {
    TEST_CLIPBOARD
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}
