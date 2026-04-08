use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI8, Ordering};
use std::sync::{Mutex, OnceLock};

static TEST_MODE: AtomicBool = AtomicBool::new(false);
static TEST_CANDIDATE_AVAILABILITY: AtomicI8 = AtomicI8::new(-1);
static AVAILABILITY_OVERRIDES: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();

pub fn enable_test_mode() {
    TEST_MODE.store(true, Ordering::Relaxed);
}

pub fn test_mode_enabled() -> bool {
    TEST_MODE.load(Ordering::Relaxed)
}

pub fn set_test_candidate_availability(value: Option<bool>) {
    let encoded = match value {
        Some(true) => 1,
        Some(false) => 0,
        None => -1,
    };
    TEST_CANDIDATE_AVAILABILITY.store(encoded, Ordering::Relaxed);
}

pub fn test_candidate_availability() -> Option<bool> {
    match TEST_CANDIDATE_AVAILABILITY.load(Ordering::Relaxed) {
        1 => Some(true),
        0 => Some(false),
        _ => None,
    }
}

fn normalize_command(command: &str) -> String {
    command.trim().to_ascii_lowercase()
}

pub fn availability_override(command: &str) -> Option<bool> {
    let key = normalize_command(command);
    AVAILABILITY_OVERRIDES
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .ok()
        .and_then(|overrides| overrides.get(&key).copied())
}

pub fn set_availability_override(command: &str, value: Option<bool>) {
    let key = normalize_command(command);
    let mut overrides = AVAILABILITY_OVERRIDES
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .expect("editor availability overrides poisoned");
    match value {
        Some(value) => {
            overrides.insert(key, value);
        }
        None => {
            overrides.remove(&key);
        }
    }
}

pub fn reset_availability_overrides() {
    AVAILABILITY_OVERRIDES
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .expect("editor availability overrides poisoned")
        .clear();
}
