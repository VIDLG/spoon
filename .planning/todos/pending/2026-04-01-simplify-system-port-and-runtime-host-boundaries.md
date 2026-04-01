---
created: 2026-04-01T00:00:00Z
title: Simplify SystemPort and ScoopRuntimeHost boundaries
area: backend
files:
  - spoon-backend/src/ports.rs
  - spoon-backend/src/scoop/runtime/execution.rs
  - spoon-backend/src/scoop/runtime/surface.rs
  - spoon/src/service/mod.rs
---

## Problem

`SystemPort` currently mixes two different concerns:

- genuine host-environment mutation capabilities
  - `ensure_user_path_entry`
  - `ensure_process_path_entry`
  - `remove_user_path_entry`
  - `remove_process_path_entry`
- a path-source helper
  - `home_dir`

After review, `home_dir()` does not look like a core system-mutation port capability. It is mostly used as an incidental path source in Scoop runtime code, especially around shortcut/test-mode handling.

There is also explicit responsibility overlap between:

- `SystemPort`
- `ScoopRuntimeHost`

because `ScoopRuntimeHost` repeats the same PATH/home methods instead of clearly composing `SystemPort` plus Scoop-specific capabilities.

## Solution

Clean this up as part of **Phase 8 shared backend contract hardening**:

1. Remove `home_dir()` from `SystemPort`.
2. Keep `SystemPort` focused on genuine host-environment mutation.
3. Rework `ScoopRuntimeHost` so it no longer redundantly redefines the same generic system capabilities unless there is a strong domain-specific reason.
4. Move any remaining home-directory/path-source behavior to a more appropriate layout/config/runtime-specific boundary.

This should be treated as shared contract cleanup, not as an urgent Phase 7 blocker.
