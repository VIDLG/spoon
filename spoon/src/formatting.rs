use std::time::SystemTime;

pub use spoon_core::format_bytes;

pub fn format_system_time(time: SystemTime) -> String {
    humantime::format_rfc3339_seconds(time).to_string()
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::{format_bytes, format_system_time};

    #[test]
    fn format_bytes_uses_compact_binary_units() {
        assert_eq!(format_bytes(512), "512B");
        assert_eq!(format_bytes(2048), "2.0K");
        assert_eq!(format_bytes(3 * 1024 * 1024), "3.0M");
        assert_eq!(format_bytes(5 * 1024 * 1024 * 1024), "5.0G");
    }

    #[test]
    fn format_system_time_uses_rfc3339_seconds() {
        let formatted = format_system_time(UNIX_EPOCH + Duration::from_secs(60));
        assert_eq!(formatted, "1970-01-01T00:01:00Z");
    }
}
