# Phase 7: Canonical MSVC State and Lifecycle - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-04-01
**Phase:** 07-canonical-msvc-state-and-lifecycle
**Areas discussed:** canonical state shape, lifecycle sharing, validate/doctor/repair relationship, canonical state vs detection, phase scope

---

## Canonical State Shape

| Option | Description | Selected |
|--------|-------------|----------|
| One canonical envelope plus runtime-specific detail | Shared high-level facts and lifecycle facts, with strategy-specific detail sections | x |
| Only minimal unified state, keep most detail outside | |
| Separate persisted states for managed and official | |

**User's choice:** One canonical envelope plus runtime-specific detail.
**Notes:** MSVC should not replay Scoop's old multi-model drift problem.

---

## Shared vs Split Lifecycle

| Option | Description | Selected |
|--------|-------------|----------|
| Shared high-level lifecycle contract, strategy-specific execute/validate branches | x |
| Separate managed and official lifecycles | |
| Only formalize managed, keep official as a special case | |

**User's choice:** Shared lifecycle contract with strategy-specific execute/validate branches.
**Notes:** Events, journals, tests, and app translation need one lifecycle language.

---

## Validate / Doctor / Repair

| Option | Description | Selected |
|--------|-------------|----------|
| Validate enters the official lifecycle story; doctor aligns to canonical state; repair stays deferred | x |
| Build validate/doctor/repair as a full system together now | |
| Keep validate outside lifecycle and defer everything else | |

**User's choice:** Validate in, doctor aligned, repair deferred.
**Notes:** This phase should strengthen the domain without becoming a full reliability milestone.

---

## Canonical State vs Detection

| Option | Description | Selected |
|--------|-------------|----------|
| Canonical state is authoritative; detection is evidence/refresh/reconcile | x |
| Only trust external detection, not canonical state | |
| Treat canonical and detection as co-equal truth sources | |

**User's choice:** Canonical state is authoritative; detection is evidence/refresh/reconcile.
**Notes:** Backend records should be trusted but evidence-backed, not imagined.

---

## Phase Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Canonical state + lifecycle execution + query/status/doctor alignment + focused regressions | x |
| Include event/error/fsx/archive/download hardening too | |
| Only do canonical state, leave lifecycle execution mostly unchanged | |

**User's choice:** Canonical state + lifecycle execution + query/status/doctor alignment + focused regressions.
**Notes:** Shared contract hardening remains Phase 8 work.

---

## the agent's Discretion

- Planning may decide the exact split between schema/store work and lifecycle execution work so long as Phase 7 remains centered on one canonical MSVC backend state machine.

## Deferred Ideas

- Event contract redesign
- Error contract redesign
- Full repair system
- Broad shared primitive extraction
