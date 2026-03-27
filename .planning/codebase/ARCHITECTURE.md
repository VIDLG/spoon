# Architecture

**Analysis Date:** 2024-03-28

## Pattern Overview

**Overall:** Plugin-Service-Backend Architecture with skills as extensibility points

**Key Characteristics:**
- Claude Code plugin architecture with Rust core
- Separation of concerns: CLI, skills, backend services, and plugin interface
- Event-driven backend with progress tracking
- Async runtime with cancellation support
- Testable configuration system with environment variable fallbacks

## Layers

**Plugin Layer (Claude Code Interface):**
- Purpose: Interacts with Claude Code plugin system
- Location: `.claude-plugin/`
- Contains: Plugin manifest, hooks, marketplace configuration
- Depends on: Exposed spoon commands and interface
- Used by: Claude Code plugin system

**CLI Layer (`spoon/`):**
- Purpose: Command-line interface and TUI
- Location: `spoon/src/`
- Contains: CLI parsing, terminal UI, command execution
- Depends on: Backend service layer
- Used by: End users via spoon.exe

**Service Layer:**
- Purpose: Application logic and skill orchestration
- Location: `spoon/src/service/`, `spoon/src/actions/`
- Contains: Tool actions, package management, cache management
- Depends on: Backend layer for async operations
- Used by: CLI layer

**Backend Layer (`spoon-backend/`):**
- Purpose: Core abstractions and event system
- Location: `spoon-backend/src/`
- Contains: Backend traits, events, async abstractions
- Depends on: Standard async runtime, file system operations
- Used by: Service layer for async operations

**Skills Layer:**
- Purpose: Domain-specific tool management
- Location: `skills/`
- Contains: Scoop, proxy, ai-toolchain, python-via-uv skills
- Depends on: Service layer through skill boundaries
- Used by: Service layer to delegate specific operations

## Data Flow

**Command Execution Flow:**

1. CLI parses user input via clap
2. Service layer resolves skill boundaries and delegates
3. Backend layer handles async operations with event streaming
4. Results formatted and returned through CLI layer

**Event Flow:**
```
Backend Task → ProgressEvent → CLI Output → User Terminal
Backend Task → FinishEvent → CLI Result → Success/Error Display
```

**Skill Boundary Flow:**
```
Service Layer → Skill Selection → Skill Execution → Result Processing
```

**State Management:**
- Configuration: Global and policy configs loaded from registry/env
- Cache: Persistent cache management with pruning capabilities
- Environment: PATH management and process environment updates

## Key Abstractions

**BackendService:**
- Purpose: Core async operation abstractions
- Examples: `spoon-backend/src/lib.rs`
- Pattern: Trait-based with error propagation

**ToolAction:**
- Purpose: Represents a tool execution action
- Examples: `spoon/src/actions/model.rs`
- Pattern: Command + args + execution context

**StreamChunk:**
- Purpose: Streaming output processing
- Examples: `spoon/src/service/mod.rs`
- Pattern: Append/Replace for progressive updates

**CommandResult:**
- Purpose: Command execution result container
- Examples: `spoon/src/service/mod.rs`
- Pattern: Success/Fail status with output collection

## Entry Points

**Main Application:**
- Location: `spoon/src/main.rs`
- Triggers: CLI arguments, TUI launch
- Responsibilities: Initialization, logging, command routing

**Service Entry:**
- Location: `spoon/src/service/mod.rs`
- Triggers: CLI command delegation
- Responsibilities: Skill boundary resolution, backend configuration

**Backend Entry:**
- Location: `spoon-backend/src/lib.rs`
- Triggers: Async task creation
- Responsibilities: Event management, progress tracking, error propagation

## Error Handling

**Strategy:** Propagate errors with context, support JSON output mode

**Patterns:**
- Backend errors converted to anyhow::Result
- JSON mode structured error responses
- Graceful cancellation with cancellation tokens
- Progress state preserved during cancellations

## Cross-Cutting Concerns

**Logging:**
- Structured logging with verbosity levels
- Buffered stdout for clean TUI interaction
- Command start tracking

**Configuration:**
- Registry-based with environment overrides
- Test mode support for deterministic behavior
- Policy and global config separation

**Cancellation:**
- Token-based async cancellation
- Clean process termination
- Event cancellation propagation

**Environment Management:**
- PATH registry integration
- Environment variable synchronization
- User scope modifications

---

*Architecture analysis: 2024-03-28*
*Focus: Plugin-Service-Backend architecture with skill extensibility*
