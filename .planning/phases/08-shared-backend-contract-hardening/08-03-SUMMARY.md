---
phase: 08-shared-backend-contract-hardening
plan: 3
completed: 2026-04-01
requirements-completed: [BECT-03]
---

# Phase 08 Plan 3 Summary

Shared backend IO primitives are now less duplicated across Scoop and MSVC.

## Key Outcomes

- Added a shared [`download.rs`](/d:/projects/spoon/spoon-backend/src/download.rs) primitive for local-copy/remote-download with shared progress emission.
- Added a shared [`archive.rs`](/d:/projects/spoon/spoon-backend/src/archive.rs) primitive for ZIP extraction.
- Moved Scoop and MSVC callers onto those shared primitives instead of keeping duplicate inline implementations.
- Kept `fsx` narrow rather than turning it into a general-purpose archive/download utility bucket.

## Verification

- `cargo check -p spoon-backend -p spoon`
- `cargo test -p spoon --test tui_msvc_download_flow -- --nocapture`
- `cargo test -p spoon --test scoop_flow scoop_install_package_runs_spoon_owned_runtime_for_real -- --nocapture`
- `cargo test -p spoon-backend install_toolchain_prepares_extracted_zip_payloads_in_cache -- --nocapture`
