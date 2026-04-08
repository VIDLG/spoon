#[cfg(test)]
mod tests {
    use crate::*;

    // ── Manifest parsing tests ──

    #[test]
    fn parse_simple_manifest() {
        let json = r#"{
            "version": "1.0.0",
            "description": "A test package",
            "homepage": "https://example.com",
            "license": "MIT",
            "url": "https://example.com/test.zip",
            "hash": "abc123"
        }"#;

        let manifest = parse_manifest(json).expect("parse manifest");
        assert_eq!(manifest.version, Some("1.0.0".to_string()));
        assert_eq!(manifest.description, Some("A test package".to_string()));
        assert_eq!(manifest.homepage, Some("https://example.com".to_string()));
        assert!(matches!(manifest.license, Some(License::Simple(ref s)) if s == "MIT"));
    }

    #[test]
    fn manifest_with_bins() {
        let json = r#"{
            "version": "2.0.0",
            "bin": [["bin/app.exe", "app"]],
            "url": "https://example.com/app.zip",
            "hash": "def456"
        }"#;

        let manifest = parse_manifest(json).expect("parse manifest");
        let bins = manifest.bin.expect("bins");
        let bin_entries = bins.to_vec();
        assert_eq!(bin_entries.len(), 1);
        assert_eq!(bin_entries[0].path(), "bin/app.exe");
        assert_eq!(bin_entries[0].alias(), Some("app"));
    }

    #[test]
    fn manifest_with_architecture() {
        let json = r#"{
            "version": "3.0.0",
            "architecture": {
                "64bit": {
                    "url": "https://example.com/app-x64.zip",
                    "hash": "hash64"
                },
                "32bit": {
                    "url": "https://example.com/app-x86.zip",
                    "hash": "hash86"
                }
            }
        }"#;

        let manifest = parse_manifest(json).expect("parse manifest");
        let arch = manifest.architecture.expect("architecture");
        let x64 = arch.for_arch("64bit").expect("x64 config");
        assert!(x64.url.is_some());
    }

    #[test]
    fn manifest_with_shortcuts() {
        // Note: shortcuts in Scoop manifests are parsed differently in ResolvedPackageSource
        // The ScoopManifest.shortcuts field uses object/string format
        let json = r#"{
            "version": "1.0.0",
            "shortcuts": [{"name": "My App", "target": "bin/app.exe"}],
            "url": "https://example.com/app.zip",
            "hash": "abc"
        }"#;

        let manifest = parse_manifest(json).expect("parse manifest");
        let shortcuts = manifest.shortcuts.expect("shortcuts");
        assert_eq!(shortcuts.len(), 1);
    }

    // ── License tests ──

    #[test]
    fn license_simple() {
        let license = License::Simple("MIT".to_string());
        assert_eq!(license.identifier(), "MIT");
    }

    #[test]
    fn license_detailed() {
        let license = License::Detailed {
            identifier: "Apache-2.0".to_string(),
            url: Some("https://apache.org".to_string()),
        };
        assert_eq!(license.identifier(), "Apache-2.0");
    }

    // ── Notes tests ──

    #[test]
    fn notes_single_line() {
        let notes = Notes::Single("Install note".to_string());
        assert_eq!(notes.lines(), vec!["Install note"]);
    }

    #[test]
    fn notes_multiple_lines() {
        let notes = Notes::Multiple(vec!["Note 1".to_string(), "Note 2".to_string()]);
        assert_eq!(notes.lines(), vec!["Note 1", "Note 2"]);
    }

    // ── StringOrArray tests ──

    #[test]
    fn string_or_array_single_value() {
        let s = StringOrArray::Single("value".to_string());
        assert_eq!(s.to_vec(), vec!["value"]);
    }

    #[test]
    fn string_or_array_multiple_values() {
        let s = StringOrArray::Multiple(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(s.to_vec(), vec!["a", "b"]);
    }

    // ── Bucket tests ──

    #[test]
    fn bucket_spec_known_bucket() {
        let spec = BucketSpec {
            name: "main".to_string(),
            source: None,
            branch: None,
        };
        let bucket = spec.resolve().expect("resolve bucket");
        assert_eq!(bucket.name, "main");
        assert_eq!(bucket.source, "https://github.com/ScoopInstaller/Main");
        assert_eq!(bucket.branch, "master");
    }

    #[test]
    fn bucket_spec_unknown_bucket() {
        let spec = BucketSpec {
            name: "unknown-bucket".to_string(),
            source: None,
            branch: None,
        };
        assert!(spec.resolve().is_err());
    }

    #[test]
    fn bucket_spec_custom_source() {
        let spec = BucketSpec {
            name: "custom".to_string(),
            source: Some("https://github.com/user/custom".to_string()),
            branch: Some("develop".to_string()),
        };
        let bucket = spec.resolve().expect("resolve bucket");
        assert_eq!(bucket.name, "custom");
        assert_eq!(bucket.source, "https://github.com/user/custom");
        assert_eq!(bucket.branch, "develop");
    }

    // ── Package source tests ──

    #[test]
    fn arch_key() {
        let arch = current_architecture_key();
        if cfg!(target_arch = "x86_64") {
            assert_eq!(arch, "64bit");
        } else if cfg!(target_arch = "aarch64") {
            assert_eq!(arch, "arm64");
        } else {
            assert_eq!(arch, "32bit");
        }
    }

    #[test]
    fn dep_lookup_key() {
        assert_eq!(dependency_lookup_key("git"), "git");
        assert_eq!(dependency_lookup_key("extras/git"), "git");
        assert_eq!(dependency_lookup_key("https://github.com/user/repo"), "repo");
    }

    // ── State types tests ──

    #[test]
    fn installed_state() {
        let state = InstalledPackageState {
            identity: InstalledPackageIdentity {
                package: "git".to_string(),
                version: "2.40.0".to_string(),
                bucket: "main".to_string(),
                architecture: Some("64bit".to_string()),
                cache_size_bytes: Some(1024),
            },
            command_surface: InstalledPackageCommandSurface::default(),
            integrations: Vec::new(),
            uninstall: InstalledPackageUninstall::default(),
        };
        assert_eq!(state.package(), "git");
        assert_eq!(state.version(), "2.40.0");
        assert_eq!(state.bucket(), "main");
    }

    // ── Helpers tests ──

    #[test]
    fn display_string() {
        let v = serde_json::json!("test");
        assert_eq!(value_to_display(&v), Some("test".to_string()));
    }

    #[test]
    fn display_number() {
        let v = serde_json::json!(42);
        assert_eq!(value_to_display(&v), Some("42".to_string()));
    }

    #[test]
    fn display_null() {
        let v = serde_json::json!(null);
        assert_eq!(value_to_display(&v), None);
    }

    #[test]
    fn string_items_array() {
        let v = serde_json::json!(["a", "b", "c"]);
        let items = string_items(Some(v));
        assert_eq!(items, vec!["a", "b", "c"]);
    }

    #[test]
    fn urls_from_json() {
        let v = serde_json::json!({
            "url": "https://example.com/file.zip",
            "hash": "abc123"
        });
        let urls = collect_urls_vec(&v, false);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com/file.zip");
    }
}