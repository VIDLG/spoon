# Requirements: Backend Architecture Completion

**Defined:** 2026-03-31
**Core Value:** Make `spoon-backend` the single trusted backend core and keep `spoon` as the thin app shell that orchestrates and presents it.

## v1 Requirements

### MSVC Architecture

- [ ] **MSVC-01**: `spoon-backend` exposes explicit MSVC operation and query entry points so the app shell no longer depends on MSVC module internals.
- [ ] **MSVC-02**: MSVC persisted facts and read models are represented through canonical backend-owned state rather than scattered module-local structs.
- [ ] **MSVC-03**: MSVC install, update, remove, and repair-style flows execute through explicit backend lifecycle stages with clear side-effect ownership.
- [ ] **MSVC-04**: Spoon app surfaces MSVC status, detail, and progress from backend models/events without reconstructing backend behavior locally.

### Shared Backend Contracts

- [ ] **BECT-01**: Backend event semantics for core product flows use stronger typed contracts instead of relying on weak stringly progress categories.
- [ ] **BECT-02**: Backend error handling distinguishes domain, infrastructure, and user-action-needed failures without defaulting broad cases to `Other(String)` or equivalent buckets.
- [ ] **BECT-03**: Reusable filesystem and path operations are centralized into backend-owned helpers instead of remaining duplicated across runtime domains.
- [ ] **BECT-04**: Production backend runtime logic avoids hardcoded install-sensitive absolute paths when layout, system, or runtime abstractions can derive them.

### Safety and Verification

- [ ] **TEST-04**: Focused backend tests cover MSVC lifecycle regressions and shared backend contract changes close to the logic that changed.
- [ ] **TEST-05**: App-shell tests for MSVC and shared output remain translation/orchestration focused and do not re-implement backend semantics.
- [ ] **TEST-06**: Real or ignored smoke coverage stays narrow but still exercises the highest-value MSVC/shared backend integration seams.

## v2 Requirements

### Cross-Domain Reliability

- **RELY-03**: Backend repair and doctor flows can reconcile both Scoop and MSVC residue through shared recovery semantics.
- **RELY-04**: Backend event/output contracts carry richer operator diagnostics and actionable recovery guidance.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Reopening a second large Scoop architectural rewrite | Scoop was stabilized in `v0.5.0`; only spillover cleanup should happen here. |
| New user-facing CLI/TUI feature expansion unrelated to backend contracts | This milestone is architecture-first, not product-surface-first. |
| Winget ownership or editor-install routing redesign | Still outside the owned runtime model. |
| Cross-platform abstraction beyond current Windows-first constraints | Adds scope without serving the immediate backend cleanup goal. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| MSVC-01 | Phase 6 | Pending |
| MSVC-02 | Phase 7 | Pending |
| MSVC-03 | Phase 7 | Pending |
| MSVC-04 | Phase 6 | Pending |
| BECT-01 | Phase 8 | Pending |
| BECT-02 | Phase 8 | Pending |
| BECT-03 | Phase 8 | Pending |
| BECT-04 | Phase 8 | Pending |
| TEST-04 | Phase 9 | Pending |
| TEST-05 | Phase 9 | Pending |
| TEST-06 | Phase 9 | Pending |

**Coverage:**
- v1 requirements: 11 total
- Mapped to phases: 11
- Unmapped: 0

---
*Requirements defined: 2026-03-31*
*Last updated: 2026-03-31 after milestone start*
