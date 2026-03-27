# Spoon Backend Refactoring

## What This Is

Spoon is a Windows development environment management toolkit that runs as a Claude Code plugin. `spoon-backend/` is the Rust async core that handles scoop package management operations, git operations, proxy configuration, and MSVC toolchain support. This project focuses on a comprehensive refactoring of spoon-backend to improve module structure, code quality, and maintainability.

## Core Value

Clean, well-structured backend modules with clear boundaries — every piece of functionality should live in the right place with proper error handling and test coverage.

## Requirements

### Validated

- ✓ Scoop package management (install, update, uninstall, cache, buckets) — existing
- ✓ MSVC toolchain detection and installation — existing
- ✓ Git clone/sync operations — existing
- ✓ Proxy configuration and normalization — existing
- ✓ Event-driven progress tracking with cancellation — existing
- ✓ Async runtime with tokio — existing

### Active

- [ ] Refactor module structure to align with WORKLINES.md seam architecture (config, runtime, catalog, env, format)
- [ ] Fix all clippy warnings (20+ currently flagged)
- [ ] Eliminate unwrap/expect in non-test code — proper error propagation
- [ ] Unify gix dependency version between spoon and spoon-backend
- [ ] Remove duplicate config-entry model in backend/scoop
- [ ] Move inline tests out of implementation files into dedicated test directories
- [ ] Clean up large files (msvc/mod.rs at 1752 lines, msvc/tests/root.rs at 1782 lines)
- [ ] Add integration tests for critical multi-step workflows

### Out of Scope

- MSVC module structural changes — complex and stable, defer to future work
- Cross-platform support — intentionally Windows-first
- TUI/CLI layer changes — this is backend-only
- New feature development — pure refactoring, no new capabilities
- ai-setup migration — already removed, no longer relevant

## Context

- Brownfield Rust project with established async patterns
- WORKLINES.md documents ongoing module boundary work (seam architecture)
- CONCERNS.md identifies ~15 specific issues across tech debt, security, testing
- 20+ clippy warnings currently blocking clean builds with `-D warnings`
- spoon and spoon-backend share a Cargo workspace but have diverged on gix versions
- Backend has two main domain modules: `scoop/` and `msvc/` with shared infrastructure
- Test files are large and mixed with implementation code

## Constraints

- **Tech Stack**: Rust (edition 2024), tokio async runtime, must remain compatible with Claude Code plugin system
- **Platform**: Windows-only — no need for cross-platform abstractions
- **Backwards Compatibility**: Public API surface (`lib.rs` exports) must not break existing spoon/ consumers
- **Git Safety**: No destructive operations, preserve existing code until new code is verified

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Skip MSVC module restructuring | Complex and currently stable; changes carry high risk for low immediate reward | — Pending |
| Unify gix to 0.80 | Reduces maintenance burden, aligns versions across workspace | — Pending |
| Follow WORKLINES.md seam architecture | Established direction with clear module boundaries already defined | — Pending |
| Module structure as top priority | Clean boundaries unlock all other improvements (testing, error handling, deps) | — Pending |
| Fix clippy as early phase | Clean compiler output is a prerequisite for safe refactoring | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-28 after initialization*
