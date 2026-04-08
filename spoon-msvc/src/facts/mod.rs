pub mod cache;
pub mod manifest;
pub mod package_rules;
pub mod rules;

pub use cache::{clear, prune};
pub use manifest::{
    companion_cab_payloads_for_selected_msi_from_cached_manifest,
    latest_toolchain_target_from_cached_manifest, selected_payloads_from_cached_manifest,
    sync_release_manifest_cache_async,
    Payload, SelectedPayload,
};
pub use package_rules::{
    ArchiveKind, ManagedPackageKind, ManifestPackageId, PayloadKind, archive_kind,
    identify_manifest_package_id, identify_payload, manifest_package_matches_msvc_target,
    normalize_msvc_build_version, package_kind, sdk_payload_matches_target,
};
pub use rules::{
    ToolchainTarget, installed_state_path, package_token_after_prefix,
    parse_toolchain_target_from_lines, pick_higher_version, read_installed_toolchain_target,
    select_latest_toolchain_from_packages, version_key, write_installed_toolchain_target,
};
