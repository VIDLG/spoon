use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

static TEST_MODE: AtomicBool = AtomicBool::new(false);
static HOME_OVERRIDE: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

pub fn enable_test_mode() {
    TEST_MODE.store(true, Ordering::Relaxed);
}

pub fn test_mode_enabled() -> bool {
    TEST_MODE.load(Ordering::Relaxed)
}

pub fn set_home_override(path: PathBuf) {
    let slot = HOME_OVERRIDE.get_or_init(|| Mutex::new(None));
    *slot.lock().expect("home override lock") = Some(path.clone());
}

pub fn home_dir() -> PathBuf {
    if let Some(slot) = HOME_OVERRIDE.get()
        && let Some(path) = slot.lock().expect("home override lock").clone()
    {
        return path;
    }
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}
