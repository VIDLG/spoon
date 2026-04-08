# AGENTS.md

## Scope Rules
- `spoon/` owns application orchestration, CLI/TUI, config, and integration glue
- `spoon-core/` owns shared infrastructure — no domain logic here
- `spoon-scoop/` owns Scoop domain logic
- `spoon-msvc/` owns MSVC domain logic
- Do not move shared logic back into `spoon/` unless it is app-specific glue

## Editing Safety
- Make the smallest possible change that satisfies the request
- No unrelated refactors or formatting churn in the same patch
- Do not revert changes you did not create unless explicitly requested

## Git Safety
- Never run destructive operations unless the user explicitly asks
- Stage only intended files; keep commits atomic
- Do not mix unrelated worktree changes into your commit

## Encoding (Windows)
- UTF-8 without BOM for all text files
- Do not commit GBK, ANSI, or UTF-16 files
- Do not rely on PowerShell default encoding for non-ASCII text
- Verify file bytes, not just terminal display, when encoding is in question

## Testing
- Domain tests in their crate (spoon-scoop, spoon-msvc); `spoon` tests for app flow
- Prefer focused unit tests over broad suites
- When a flow test exposes a bug, add a unit test near the logic
