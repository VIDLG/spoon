---
phase: 08-shared-backend-contract-hardening
plan: 4
completed: 2026-04-01
requirements-completed: [BECT-03, BECT-04]
---

# Phase 08 Plan 4 Summary

Shared host/path contracts are now cleaner, and the most obvious JSON-era layout/path leftovers have been reduced.

## Key Outcomes

- Removed `home_dir()` from [`SystemPort`](/d:/projects/spoon/spoon-backend/src/ports.rs) and reduced the duplicate generic host capability surface in [`ScoopRuntimeHost`](/d:/projects/spoon/spoon-backend/src/scoop/runtime/execution.rs).
- Updated Scoop runtime surface handling so test-mode path derivation no longer relies on `SystemPort::home_dir()`.
- Removed JSON-era fields `package_state_root` and `bucket_registry_path` from [`ScoopLayout`](/d:/projects/spoon/spoon-backend/src/layout.rs), with remaining paths now derived from `state_root`.
- Isolated Windows system tool resolution through [`platform.rs`](/d:/projects/spoon/spoon-backend/src/platform.rs) and removed repeated hardcoded `msiexec.exe` paths from production runtime code.

## Verification

- `cargo check -p spoon-backend -p spoon`
- `cargo test -p spoon-backend runtime_writes_canonical_scoop_state -- --nocapture`
- `cargo test -p spoon --test scoop_flow -- --nocapture`
