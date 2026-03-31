//! Tests for proxy URL normalization and validation.

use crate::normalize_proxy_url;

#[test]
fn proxy_normalization_various_formats() {
    // Empty input
    assert_eq!(normalize_proxy_url("").unwrap(), None);
    assert_eq!(normalize_proxy_url("   ").unwrap(), None);

    // Adds default http:// scheme
    assert_eq!(
        normalize_proxy_url("127.0.0.1:7897").unwrap(),
        Some("http://127.0.0.1:7897".to_string())
    );

    // Removes trailing slash
    assert_eq!(
        normalize_proxy_url("http://127.0.0.1:7897/").unwrap(),
        Some("http://127.0.0.1:7897".to_string())
    );

    // Preserves various proxy schemes
    assert_eq!(
        normalize_proxy_url("https://proxy.example.com:3128").unwrap(),
        Some("https://proxy.example.com:3128".to_string())
    );
    assert_eq!(
        normalize_proxy_url("socks5://localhost:1080").unwrap(),
        Some("socks5://localhost:1080".to_string())
    );
    assert_eq!(
        normalize_proxy_url("socks5://127.0.0.1:1080").unwrap(),
        Some("socks5://127.0.0.1:1080".to_string())
    );
}

#[test]
fn proxy_trims_whitespace() {
    assert_eq!(
        normalize_proxy_url("  http://127.0.0.1:7897  ").unwrap(),
        Some("http://127.0.0.1:7897".to_string())
    );

    assert_eq!(
        normalize_proxy_url("  127.0.0.1:7897  ").unwrap(),
        Some("http://127.0.0.1:7897".to_string())
    );
}
