---
created: 2026-03-31T00:00:00Z
title: Remove hardcoded production paths
area: general
files:
  - spoon-backend/src/scoop/extract.rs
  - spoon-backend/src/scoop/runtime/hooks.rs
---

## Problem

There are still hardcoded production paths in backend runtime code, especially around MSI execution. These are acceptable in a narrow Windows-first sense, but they are still a portability, migration, and packaging risk because they encode assumptions about system locations directly in backend execution paths.

The most obvious current examples are the `C:\\Windows\\System32\\msiexec.exe` usages in Scoop extraction and hook execution code. That kind of path assumption is more risky than test-only fake paths like `tool-root` or renderer sample output, because it directly affects shipped runtime behavior.

## Solution

Audit backend production code for hardcoded absolute runtime paths and replace them with a narrower, better-isolated mechanism. For Windows system tools, prefer one well-named resolution/helper layer over repeating absolute paths inline. Keep this cleanup scoped to production/runtime code, not test fixtures or sample render strings.

