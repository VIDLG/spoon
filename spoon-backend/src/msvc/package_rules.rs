#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedPackageKind {
    Autoenv,
    Msvc,
    Sdk,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestPackageId {
    MsvcHostTarget {
        build_version: String,
        host_arch: String,
        target_arch: String,
    },
    Msbuild,
    Diasdk,
    Ninja {
        version: String,
    },
    Cmake {
        version: String,
    },
    Unknown,
}

pub fn normalize_msvc_build_version(version: &str) -> String {
    let mut parts = Vec::new();
    for part in version.split('.') {
        if part.chars().all(|ch| ch.is_ascii_digit()) {
            parts.push(part);
        } else {
            break;
        }
    }
    if parts.is_empty() {
        version.to_string()
    } else {
        parts.join(".")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadKind {
    Sdk,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveKind {
    Vsix,
    Msi,
    Cab,
    Zip,
}

pub fn package_kind(package: &str) -> ManagedPackageKind {
    if package.eq_ignore_ascii_case("autoenv") {
        ManagedPackageKind::Autoenv
    } else if package.starts_with("msvc-") {
        ManagedPackageKind::Msvc
    } else if package.starts_with("sdk-") {
        ManagedPackageKind::Sdk
    } else {
        ManagedPackageKind::Unknown
    }
}

pub fn identify_manifest_package_id(id: &str) -> ManifestPackageId {
    if id == "Microsoft.Build" || id == "Microsoft.Build.Arm64" {
        return ManifestPackageId::Msbuild;
    }
    if id == "Microsoft.VisualCpp.DIA.SDK" {
        return ManifestPackageId::Diasdk;
    }
    if let Some(rest) = id.strip_prefix("ninja-") {
        return ManifestPackageId::Ninja {
            version: rest.to_string(),
        };
    }
    if let Some(rest) = id.strip_prefix("cmake-") {
        return ManifestPackageId::Cmake {
            version: rest.to_string(),
        };
    }
    if let Some(rest) = id.strip_prefix("Microsoft.VC.") {
        let Some(tools_offset) = rest.find(".Tools.") else {
            return ManifestPackageId::Unknown;
        };
        let build_version = &rest[..tools_offset];
        let rest = &rest[tools_offset + ".Tools.".len()..];
        let Some(host_offset) = rest.find(".Target") else {
            return ManifestPackageId::Unknown;
        };
        let host = &rest[..host_offset];
        let rest = &rest[host_offset + 1..];
        let Some(name_offset) = rest[6..].find('.') else {
            return ManifestPackageId::Unknown;
        };
        let Some(host_arch) = host.strip_prefix("Host") else {
            return ManifestPackageId::Unknown;
        };
        let target_end = 6 + name_offset;
        let Some(target_arch) = rest[..target_end].strip_prefix("Target") else {
            return ManifestPackageId::Unknown;
        };
        return ManifestPackageId::MsvcHostTarget {
            build_version: normalize_msvc_build_version(build_version),
            host_arch: host_arch.to_ascii_lowercase(),
            target_arch: target_arch.to_ascii_lowercase(),
        };
    }
    ManifestPackageId::Unknown
}

pub fn manifest_package_matches_msvc_target(
    id: &str,
    target_version: &str,
    host_arch: &str,
    target_arch: &str,
) -> bool {
    let Some(rest) = id.strip_prefix("Microsoft.VC.") else {
        return false;
    };

    if let Some(rest) = rest.strip_prefix(target_version) {
        if rest == ".Servicing" || rest == ".Servicing.Compilers" || rest == ".Servicing.CrtHeaders"
        {
            return true;
        }
        if rest == ".Tools.Core.Props" {
            return true;
        }
        if rest == ".Props" || rest == format!(".Props.{target_arch}") {
            return true;
        }
        if let Some(crt) = rest.strip_prefix(".CRT.") {
            if crt == "Headers.base" {
                return true;
            }
            if crt.starts_with("Redist.") {
                return crt.starts_with(target_arch) && crt.ends_with(".base");
            }
            return crt.starts_with(target_arch)
                && (crt.ends_with(".Desktop.base")
                    || crt.ends_with(".Desktop.debug.base")
                    || crt.ends_with(".Store.base"));
        }
    }

    match identify_manifest_package_id(id) {
        ManifestPackageId::MsvcHostTarget {
            build_version,
            host_arch: package_host_arch,
            target_arch: package_target_arch,
        } => {
            build_version == target_version
                && package_host_arch == host_arch
                && package_target_arch == target_arch
                && (id.ends_with(".base") || id.ends_with(".Res.base"))
        }
        _ => false,
    }
}

pub fn identify_payload(payload_filename: &str) -> PayloadKind {
    let sdk_prefixes = [
        "Installers\\Universal CRT Headers Libraries and Sources-",
        "Installers\\Windows SDK Desktop Headers ",
        "Installers\\Windows SDK OnecoreUap Headers ",
        "Installers\\Windows SDK Desktop Libs ",
        "Installers\\Windows SDK Signing Tools-",
        "Installers\\Windows SDK for Windows Store Apps Headers-",
        "Installers\\Windows SDK for Windows Store Apps Headers OnecoreUap-",
        "Installers\\Windows SDK for Windows Store Apps Libs-",
        "Installers\\Windows SDK for Windows Store Apps Tools-",
    ];

    if sdk_prefixes
        .iter()
        .any(|prefix| payload_filename.starts_with(prefix))
    {
        PayloadKind::Sdk
    } else {
        PayloadKind::Unknown
    }
}

pub fn sdk_payload_matches_target(payload_filename: &str, target_arch: &str) -> bool {
    let path = payload_filename.to_ascii_lowercase();
    let target_arch = target_arch.to_ascii_lowercase();
    let arch_prefixes = [
        "installers\\windows sdk desktop headers ",
        "installers\\windows sdk onecoreuap headers ",
        "installers\\windows sdk desktop libs ",
    ];

    for prefix in arch_prefixes {
        if let Some(rest) = path.strip_prefix(prefix) {
            return rest.starts_with(&(target_arch + "-"));
        }
    }

    true
}

pub fn archive_kind(url: &str) -> Option<ArchiveKind> {
    if url.ends_with(".vsix") {
        Some(ArchiveKind::Vsix)
    } else if url.ends_with(".msi") {
        Some(ArchiveKind::Msi)
    } else if url.ends_with(".cab") {
        Some(ArchiveKind::Cab)
    } else if url.ends_with(".zip") {
        Some(ArchiveKind::Zip)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArchiveKind, ManagedPackageKind, ManifestPackageId, PayloadKind, archive_kind,
        identify_manifest_package_id, identify_payload, manifest_package_matches_msvc_target,
        package_kind, sdk_payload_matches_target,
    };

    #[test]
    fn package_kind_classifies_core_managed_packages() {
        assert_eq!(package_kind("autoenv"), ManagedPackageKind::Autoenv);
        assert_eq!(package_kind("msvc-14.44.17.14"), ManagedPackageKind::Msvc);
        assert_eq!(package_kind("sdk-10.0.22621.7"), ManagedPackageKind::Sdk);
        assert_eq!(package_kind("ninja-1.12.1"), ManagedPackageKind::Unknown);
    }

    #[test]
    fn identify_payload_marks_sdk_installers() {
        assert_eq!(
            identify_payload("Installers\\Windows SDK Desktop Headers x64-x86_en-us.msi"),
            PayloadKind::Sdk
        );
        assert_eq!(
            identify_payload("Installers\\Windows SDK OnecoreUap Headers x64-x86_en-us.msi"),
            PayloadKind::Sdk
        );
        assert_eq!(
            identify_payload(
                "Installers\\Windows SDK for Windows Store Apps Headers OnecoreUap-x86_en-us.msi"
            ),
            PayloadKind::Sdk
        );
        assert_eq!(
            identify_payload("Installers\\Windows SDK for Windows Store Apps Tools-x86_en-us.msi"),
            PayloadKind::Sdk
        );
        assert_eq!(
            identify_payload(
                "Installers\\Universal CRT Headers Libraries and Sources-x86_en-us.msi"
            ),
            PayloadKind::Sdk
        );
        assert_eq!(
            identify_payload("Packages\\Some Other Payload.vsix"),
            PayloadKind::Unknown
        );
    }

    #[test]
    fn sdk_payload_matches_target_filters_explicit_arch_variants_only() {
        assert!(sdk_payload_matches_target(
            "Installers\\Windows SDK Desktop Headers x64-x86_en-us.msi",
            "x64"
        ));
        assert!(!sdk_payload_matches_target(
            "Installers\\Windows SDK Desktop Headers arm64-x86_en-us.msi",
            "x64"
        ));
        assert!(!sdk_payload_matches_target(
            "Installers\\Windows SDK Desktop Libs x86-x86_en-us.msi",
            "x64"
        ));
        assert!(sdk_payload_matches_target(
            "Installers\\Windows SDK for Windows Store Apps Tools-x86_en-us.msi",
            "x64"
        ));
        assert!(sdk_payload_matches_target(
            "Installers\\Windows SDK for Windows Store Apps Headers-x86_en-us.msi",
            "x64"
        ));
    }

    #[test]
    fn identify_manifest_package_id_recognizes_msvc_host_target() {
        assert_eq!(
            identify_manifest_package_id("Microsoft.VC.14.44.35207.Tools.HostX64.TargetX64.base"),
            ManifestPackageId::MsvcHostTarget {
                build_version: "14.44.35207".to_string(),
                host_arch: "x64".to_string(),
                target_arch: "x64".to_string(),
            }
        );
    }

    #[test]
    fn identify_manifest_package_id_normalizes_premium_suffix_in_build_version() {
        assert_eq!(
            identify_manifest_package_id(
                "Microsoft.VC.14.44.17.14.Premium.Tools.HostX64.TargetX64.base"
            ),
            ManifestPackageId::MsvcHostTarget {
                build_version: "14.44.17.14".to_string(),
                host_arch: "x64".to_string(),
                target_arch: "x64".to_string(),
            }
        );
    }

    #[test]
    fn identify_manifest_package_id_recognizes_auxiliary_packages() {
        assert_eq!(
            identify_manifest_package_id("Microsoft.Build"),
            ManifestPackageId::Msbuild
        );
        assert_eq!(
            identify_manifest_package_id("Microsoft.VisualCpp.DIA.SDK"),
            ManifestPackageId::Diasdk
        );
        assert_eq!(
            identify_manifest_package_id("ninja-1.12.1"),
            ManifestPackageId::Ninja {
                version: "1.12.1".to_string()
            }
        );
    }

    #[test]
    fn manifest_package_matches_msvc_target_recognizes_crt_and_host_target_packages() {
        assert!(manifest_package_matches_msvc_target(
            "Microsoft.VC.14.44.35207.CRT.Headers.base",
            "14.44.35207",
            "x64",
            "x64"
        ));
        assert!(manifest_package_matches_msvc_target(
            "Microsoft.VC.14.44.35207.CRT.x64.Desktop.base",
            "14.44.35207",
            "x64",
            "x64"
        ));
        assert!(manifest_package_matches_msvc_target(
            "Microsoft.VC.14.44.35207.Tools.HostX64.TargetX64.Res.base",
            "14.44.35207",
            "x64",
            "x64"
        ));
        assert!(!manifest_package_matches_msvc_target(
            "Microsoft.VC.14.44.35207.CRT.arm64.Desktop.base",
            "14.44.35207",
            "x64",
            "x64"
        ));
        assert!(!manifest_package_matches_msvc_target(
            "Microsoft.VC.14.44.35207.Tools.HostX86.TargetARM64.base",
            "14.44.35207",
            "x64",
            "x64"
        ));
        assert!(manifest_package_matches_msvc_target(
            "Microsoft.VC.14.44.35207.Props.x64",
            "14.44.35207",
            "x64",
            "x64"
        ));
        assert!(manifest_package_matches_msvc_target(
            "Microsoft.VC.14.44.35207.Servicing.Compilers",
            "14.44.35207",
            "x64",
            "x64"
        ));
    }

    #[test]
    fn archive_kind_detects_supported_payload_extensions() {
        assert_eq!(
            archive_kind("https://example.invalid/a.vsix"),
            Some(ArchiveKind::Vsix)
        );
        assert_eq!(
            archive_kind("https://example.invalid/a.msi"),
            Some(ArchiveKind::Msi)
        );
        assert_eq!(
            archive_kind("https://example.invalid/a.cab"),
            Some(ArchiveKind::Cab)
        );
        assert_eq!(
            archive_kind("https://example.invalid/a.zip"),
            Some(ArchiveKind::Zip)
        );
        assert_eq!(archive_kind("https://example.invalid/a.exe"), None);
    }
}
