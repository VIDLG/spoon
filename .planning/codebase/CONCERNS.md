# Codebase Concerns

**Analysis Date:** 2026-03-28

## Tech Debt

**Deleted ai-setup Directory:**
- Issue: ai-setup directory was removed but may have lingering functionality
- Files: `ai-setup/` (entire directory deleted)
- Impact: Potential functionality gaps or migration needs
- Fix approach: Review all ai-setup features and ensure they're covered by spoon/skills or document intentional removals

**Backend Module Consolidation:**
- Issue: Backend logic scattered across multiple modules during transition
- Files: `spoon-backend/src/` - modules in transition state
- Impact: Inconsistent dependency patterns and unclear module boundaries
- Fix approach: Continue migration to explicit seams (`config`, `runtime`, `catalog`, `env`, `format`) per WORKLINES.md

**Test Organization:**
- Issue: Tests mixed with implementation code
- Files: `spoon-backend/src/msvc/tests/root.rs` (1,782 lines)
- Impact: Hard to navigate implementation and understand core behavior
- Fix approach: Move tests to dedicated `backend/.../tests/...` directories as per WORKLINES.md

## Architecture Concerns

**Backend vs Frontend Boundaries:**
- Issue: Potential confusion about responsibilities between spoon and spoon-backend
- Files: `spoon/src/` vs `spoon-backend/src/`
- Impact: Unclear where certain logic should reside
- Fix approach: Continue narrowing backend to explicit seams, keep product-layer dependencies separated

**Package Management Duplication:**
- Issue: `backend/scoop` may have duplicate config-entry model
- Files: `spoon-backend/src/scoop/`
- Impact: Inconsistent state management and duplicate logic
- Fix approach: Reuse shared package/catalog entry model instead of maintaining duplicate

## Performance Considerations

**Memory Usage in View Layer:**
- Issue: Frequent string cloning in tool row rendering
- Files: `spoon/src/view/tools/row.rs`
- Pattern: Multiple `.to_string()` calls in view code
- Impact: Potential performance degradation in large tool lists
- Improvement path: Use Cow<str> or reference strings where possible

**Large Test Files:**
- Issue: Some test files are excessively large
- Files: `spoon-backend/src/msvc/tests/root.rs` (1,782 lines)
- Impact: Hard to maintain and slow to run
- Improvement path: Split into focused test modules

## Fragile Areas

**Error Handling Patterns:**
- Issue: Heavy use of `expect()` and `unwrap()` across multiple modules
- Files: `spoon/src/` - 156 occurrences across 22 files
- Impact: Brittle error handling that can cause panics
- Safe modification: Replace with proper error handling and graceful degradation
- Test coverage: Needs verification of failure paths

**Environment Variable Dependencies:**
- Issue: Direct environment variable manipulation without validation
- Files: `spoon/src/config/env.rs`
- Risk: Unset variables or invalid paths could cause failures
- Recommendations: Add validation for environment variables and graceful fallbacks

## Dependencies at Risk

**Version Conflicts:**
- Issue: gix version mismatch between spoon (v0.80) and spoon-backend (v0.70)
- Files: `spoon/Cargo.toml` and `spoon-backend/Cargo.toml`
- Impact: Potential API incompatibilities or feature differences
- Migration plan: Align versions or handle feature differences explicitly

**Outdated Dependencies:**
- Issue: Some dependencies may be outdated (cargo-outdated not available)
- Impact: Potential security vulnerabilities or missing bug fixes
- Migration plan: Regular dependency audits with cargo-audit and cargo-update

## Missing Critical Features

**Linux/macOS Support:**
- Issue: Backend design assumes Windows-only
- Problem: Code contains Windows-specific assumptions
- Blocks: Cross-platform development or testing
- Priority: Low - intentionally Windows-first per project guidelines

**Comprehensive Error Recovery:**
- Issue: Limited recovery paths for failed operations
- What's missing: Transactional rollback for partial failures
- Blocks: Robust package management operations

## Test Coverage Gaps

**Integration Test Coverage:**
- What's not tested: Complex multi-step operations (install + configure + verify)
- Files: `spoon/tests/` - mostly unit tests
- Risk: Real-world scenarios may fail in subtle ways
- Priority: High - Add integration tests for complete workflows

**Error Path Testing:**
- What's not tested: Network failures, permission errors, invalid inputs
- Files: `spoon/tests/cli/` - focuses on success paths
- Risk: Unknown failure modes could cause data loss
- Priority: Medium - Add error injection and recovery tests

## Security Considerations

**Environment Variable Exposure:**
- Area: API key handling in config
- Risk: Keys stored in environment variables could leak
- Files: `spoon/src/config/io.rs`
- Current mitigation: Uses std::env::var with defaults
- Recommendations: Consider secure storage for sensitive credentials

**Path Traversal Risks:**
- Area: File operations with user-provided paths
- Risk: Potential directory traversal attacks
- Files: Multiple files in `spoon/src/config/` and `spoon-backend/src/scoop/`
- Current mitigation: Uses validated path helpers
- Recommendations: Add additional validation for user-provided paths

## Code Quality Concerns

**Large Files:**
- Issue: Some source files exceed reasonable size limits
- Files: `spoon-backend/src/msvc/mod.rs` (1,752 lines)
- Why fragile: Hard to understand and maintain
- Safe modification: Split into logical modules
- Test coverage: Needs focused unit tests for individual components

**Todo List in Repository:**
- Issue: TODO.md contains what appears to be an API key
- Files: `TODO.md`
- Risk: Potential security exposure
- Recommendations: Remove sensitive data and create proper issues tracking

---

*Concerns audit: 2026-03-28*