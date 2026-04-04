# Group Scoop Installed State Facets Before Any Schema Split

## Why

`InstalledPackageState` currently stores valid canonical facts, but they are still mixed in one flat model:

- package identity
- command surface
- integration results
- uninstall / reapply hook data

Before splitting database columns or tables further, the Rust model should first be regrouped into clearer semantic facets.

## Intended Direction

- keep canonical non-derivable facts
- introduce semantic grouping inside `InstalledPackageState`
- consider facets such as:
  - identity
  - command surface
  - integrations
  - uninstall state
- only evaluate physical schema split after the semantic model is cleaner

## Trigger

Do this when continuing Scoop state / contract cleanup after the current host and control-plane refactors settle.
