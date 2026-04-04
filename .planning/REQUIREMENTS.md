# Requirements: Scoop Legacy Cleanup and Domain Refinement

**Defined:** 2026-04-01
**Core Value:** Make `spoon-backend` the single trusted backend core and keep `spoon` as the thin app shell that orchestrates and presents it.

## v1 Requirements

### Scoop Legacy Cleanup

- [ ] **SLEG-01**: Active Scoop runtime paths no longer depend on JSON-era package-state or bucket-registry concepts except where legacy detection/repair explicitly requires them.
- [ ] **SLEG-02**: Scoop runtime/helper boundaries are simpler and more explicit, with fewer stale adapter layers or duplicated host-style seams inside `spoon-backend/src/scoop/`.
- [ ] **SLEG-03**: Scoop read models and status/detail outputs avoid low-value derivable redundancies when those fields add no meaningful contract value.
- [ ] **SLEG-04**: Remaining deprecated or legacy-only Scoop path helpers are either removed, downgraded to explicit legacy handling, or clearly isolated from active runtime behavior.

### Shared Cleanup Spillover

- [ ] **BECT-05**: Shared cleanup needed to finish the Scoop legacy pass is resolved without reopening the full backend contract-hardening phase.
- [ ] **BECT-06**: Remaining shared helper debt touched by the Scoop cleanup stays aligned with the backend-owned contract model rather than reintroducing app-side or legacy indirection.
- [ ] **BECT-07**: Control-plane access is path-first at its core; layout-aware opening remains a convenience rather than the primary abstraction.
- [ ] **BECT-08**: Control-plane migrations are simplified and hardened enough that future schema growth does not depend on ad hoc migration plumbing.

### Safety and Verification

- [ ] **TEST-07**: Backend-focused regressions protect the Scoop legacy cleanup near the logic that changed.
- [ ] **TEST-08**: App-shell Scoop tests remain translation/orchestration focused while still catching regressions caused by the legacy cleanup.

## v2 Requirements

### Future Cleanup

- **CLEAN-01**: Broader repository-wide deprecated path helper cleanup beyond the Scoop domain.
- **CLEAN-02**: Additional shared archive/runtime abstraction work after the current backlog and seeds are revisited.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Reopening a full Scoop lifecycle redesign | Scoop lifecycle/state/control-plane architecture is already established; this milestone is cleanup, not a third major redesign. |
| Reopening MSVC canonical-state/lifecycle architecture | Recently completed in `v0.6.0`; only incidental spillover fixes belong here. |
| Major new CLI/TUI features unrelated to Scoop cleanup | This milestone is still architecture/debt-focused. |
| Large repo-wide path cleanup | Keep the effort targeted to the Scoop domain and directly adjacent shared debt. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SLEG-01 | Phase 10 | Pending |
| SLEG-02 | Phase 12.2 | Pending |
| SLEG-03 | Phase 12 | Pending |
| SLEG-04 | Phase 10 | Pending |
| BECT-05 | Phase 12.2 | Pending |
| BECT-06 | Phase 12 | Pending |
| BECT-07 | Phase 12.1 | Pending |
| BECT-08 | Phase 12.1 | Pending |
| TEST-07 | Phase 13 | Pending |
| TEST-08 | Phase 13 | Pending |

**Coverage:**
- v1 requirements: 10 total
- Mapped to phases: 10
- Unmapped: 0

---
*Requirements defined: 2026-04-01*
*Last updated: 2026-04-01 after milestone start*
