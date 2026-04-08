#![allow(dead_code)]

use std::time::Duration;

pub const DEFAULT_WAIT: Duration = Duration::from_secs(2);
pub const EXTENDED_WAIT: Duration = Duration::from_secs(3);
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(120);
pub const SCOOP_INSTALL_TIMEOUT: Duration = Duration::from_secs(180);

pub const PAYLOAD_STANDARD: usize = 512 * 1024;
pub const PAYLOAD_LARGE: usize = 1024 * 1024;
pub const CHUNK_STANDARD: usize = 32 * 1024;
pub const CHUNK_SMALL: usize = 16 * 1024;
pub const PAYLOAD_CHUNK_DELAY_MS: u64 = 20;
