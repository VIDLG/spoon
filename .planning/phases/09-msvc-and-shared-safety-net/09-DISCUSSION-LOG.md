# Phase 9: MSVC and Shared Safety Net - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-04-01
**Phase:** 09-msvc-and-shared-safety-net
**Areas discussed:** testing emphasis, failure-path coverage, shared-contract regression layering, real smoke scope

---

## Testing Emphasis

| Option | Description | Selected |
|--------|-------------|----------|
| Backend-heavy safety net | Backend owns risky lifecycle/state/contract regressions; app stays shell-focused | x |
| App-heavy end-to-end emphasis | |
| Mostly micro unit tests only | |

**User's choice:** Backend-heavy safety net.
**Notes:** Keep the backend as the truth-owner even in the test strategy.

---

## Failure-Path Coverage

| Option | Description | Selected |
|--------|-------------|----------|
| Key failure boundaries only | Lock the most dangerous stops and residues instead of full matrices | x |
| Full stage success/failure matrix | |
| Minimal smoke-only failure coverage | |

**User's choice:** Key failure boundaries only.
**Notes:** The goal is risk coverage, not exhaustive combinatorics.

---

## Shared Contract Regression Layering

| Option | Description | Selected |
|--------|-------------|----------|
| Layered backend + integration + app-shell translation checks | x |
| Mainly app-flow validation | |
| Mainly local unit tests | |

**User's choice:** Layered backend + integration + app-shell translation checks.
**Notes:** Cross-cutting contracts need both local and integration-style protection.

---

## Real Smoke Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Sparse, isolated, opt-in real smoke | x |
| Large real-environment expansion | |
| Almost no real smoke | |

**User's choice:** Sparse, isolated, opt-in real smoke.
**Notes:** Real smoke should provide confidence, not dominate the safety strategy.

---

## the agent's Discretion

- Planning may choose the most efficient split between backend-local regressions and app-shell regressions as long as app tests do not drift into backend reimplementation.

## Deferred Ideas

- Broad environment matrix testing
- Full reliability/repair platform
