---
date: 2026-04-04
topic: spoon-backend
focus: spoon-backend overall
---

# Ideation: Spoon Backend Improvement Directions

## Codebase Context

`spoon-backend` is the shared backend core for Scoop and MSVC flows, while `spoon` remains the thinner app shell. The repo is in the middle of a deliberate cleanup arc:

- control-plane access is now path-first through [`db.rs`](/d:/projects/spoon/spoon-backend/src/db.rs)
- the Scoop domain has recently been decomposed into smaller modules for `surface`, `actions`, `extract`, `info`, and `buckets`
- the lifecycle layer was trimmed so only phase-meaningful modules remain
- the host layer is moving toward “name = responsibility” with distinct `runtime`, `helpers`, `surface`, `hooks`, `persist`, and `download` concerns

Recent learnings from Phase 12.2:

- structural decomposition already delivered clear readability wins
- the next risk is not “another giant file,” but drift between modules, contracts, and tests
- residual cleanup space remains around error semantics, filesystem helper reuse, and boundaries that still encode policy weakly

Institutional learnings search:

- no `docs/solutions/` learnings corpus was present
- the strongest local evidence came from the current milestone’s phase verification and the codebase shape itself

## Ranked Ideas

### 1. Contract-Harden the Scoop Backend Surface
**Description:** Introduce a focused contract-hardening pass over the main Scoop backend entrypoints: package actions, reapply surfaces, doctor output, bucket outcomes, and manifest/raw-load helpers. Normalize naming and reduce “weak stringly” seams where the domain already has clear meaning.
**Rationale:** The codebase has already paid the cost to decompose modules. The highest-leverage next move is to make those smaller boundaries stable and explicit so future cleanup stops leaking ambiguity back in.
**Downsides:** This creates a short burst of renames and test churn across backend and app glue.
**Confidence:** 93%
**Complexity:** Medium
**Status:** Unexplored

### 2. Split `InstalledPackageState` into Semantic Facets
**Description:** Refactor the Scoop installed-state model into grouped substructures such as identity, command surface, integration state, and uninstall state, while keeping the physical storage conservative at first.
**Rationale:** The backend has already identified this as follow-up work. It directly improves comprehension, reduces accidental field sprawl, and makes future schema decisions much easier.
**Downsides:** The storage layer and all readers/writers need careful coordinated updates, even if the DB schema stays logically flat for one phase.
**Confidence:** 91%
**Complexity:** High
**Status:** Unexplored

### 3. Build a Scoop Backend Safety-Net Layer Focused on Structural Seams
**Description:** Add focused tests near the newly decomposed seams: `surface/`, `actions/`, `extract/`, `buckets/`, host reapply flows, and manifest loading helpers, with regression assertions that the new boundaries still preserve behavior.
**Rationale:** The repo has just gone through heavy structural cleanup. The most valuable protection now is not broad coverage, but seam-level regression tests that stop refactor wins from silently regressing runtime behavior.
**Downsides:** Less glamorous than feature work and easy to under-scope unless the test targets stay disciplined.
**Confidence:** 95%
**Complexity:** Medium
**Status:** Unexplored

### 4. Introduce a Shared Scoop Environment Resolution Subdomain
**Description:** Pull all env interpolation and path-resolution rules into a dedicated Scoop environment layer shared by shims, integrations, package info, and any future runtime wrappers.
**Rationale:** The codebase is already discovering that env handling is broader than “shim helpers.” Centralizing this now would reduce policy drift and make package-local runtime environment behavior deterministic.
**Downsides:** There is a risk of over-abstracting if the shared layer grows faster than the concrete reuse.
**Confidence:** 84%
**Complexity:** Medium
**Status:** Unexplored

### 5. Unify Backend Error Semantics Around Domain Families
**Description:** Continue replacing generic `BackendError::Other(...)` usage with tighter domain-specific errors or helper constructors, especially across Scoop host/surface/buckets and MSVC shared helpers.
**Rationale:** Structural decomposition makes vague errors stand out more sharply. Better error families would improve debugging, test assertions, and the app-shell presentation layer.
**Downsides:** This can become a wide but shallow cleanup if not kept tied to real call sites and user-facing value.
**Confidence:** 88%
**Complexity:** Medium
**Status:** Unexplored

### 6. Create a Typed Action Execution Kernel Under `spoon-backend`
**Description:** Introduce a narrower internal execution kernel that coordinates operation locking, stage transitions, journaling, and outcome construction for backend actions, with Scoop and MSVC plugging into it explicitly.
**Rationale:** Both domains already share backend architecture patterns. A small execution kernel could remove repeated action-scaffolding code and make future backend domains cheaper to add.
**Downsides:** This is the boldest idea here and has the highest risk of premature generalization if attempted too early.
**Confidence:** 72%
**Complexity:** High
**Status:** Unexplored

## Rejection Summary

| # | Idea | Reason Rejected |
|---|------|-----------------|
| 1 | Replace all async boundaries with sync helpers | Not grounded; the repo is intentionally async in backend orchestration. |
| 2 | Move more logic back into `spoon/` | Directly conflicts with repo direction and AGENTS guidance. |
| 3 | Rebuild the Scoop manifest parser entirely around one giant typed schema | Too expensive relative to current value; the mixed typed/raw approach is already working. |
| 4 | Eliminate `lifecycle/` completely | Weaker than the current selective approach; some phase modules still buy readability. |
| 5 | Add broad PTY-based end-to-end coverage | Conflicts with stated testing strategy and offers poor leverage. |
| 6 | Add a generic plugin system inside `spoon-backend` | Too vague and not grounded in the present codebase pain points. |
| 7 | Replace all `PathBuf` handling with a custom path abstraction | High churn, low near-term value. |
| 8 | Turn every backend model into serde-first derive structs | Overgeneralizes from one cleanup thread and would likely regress boundary clarity. |
| 9 | Merge Scoop and MSVC domains into one unified tool domain now | Too large and not proportional to current milestone goals. |
| 10 | Add AI-generated docs for every module | Documentation-only improvement is weaker than contract/testing/state work. |
| 11 | Remove all `with_context` / `with_host` suffixes everywhere | Some still carry real semantic distinction; blanket removal would make APIs worse. |
| 12 | Convert all Windows-specific shell operations into PowerShell scripts again | Conflicts with repo direction and current backend ownership. |

## Session Log

- 2026-04-04: Initial ideation - 18 candidates considered, 6 survivors kept after adversarial filtering
