# AGENTS.md

This document provides guidance for AI coding agents working in the umem codebase.

## Project Overview

umem is a Rust workspace for memory management with AI capabilities, vector storage, and multiple service interfaces (gRPC, MCP). It uses a modular crate architecture under `crates/`.

## Build, Test, and Lint Commands

### Prerequisites

- Rust stable toolchain (see `rust-toolchain`)
- Protocol Buffers compiler: `apt-get install protobuf-compiler` (Linux) or `brew install protobuf` (macOS)

### Build

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo build -p <crate>   # Build specific crate (e.g., cargo build -p umem_core)
```

### Test

```bash
cargo test                              # Run all tests
cargo test <test_name>                  # Run tests matching name
cargo test -p <crate>                   # Run tests for specific crate
cargo test -p <crate> <test_name>       # Run specific test in crate
cargo test -p <crate> -- --nocapture    # Run with stdout visible
```

### Lint and Format

```bash
cargo fmt                # Format all code
cargo fmt --check        # Check formatting without changes
cargo clippy             # Run linter
cargo clippy --fix       # Auto-fix linter warnings
```

## Code Style Guidelines

For comprehensive examples, see `docs/guidelines.md`. Key patterns summarized below.

### Error Handling

1. **Use `thiserror` for error types** with `#[derive(Debug, Error)]`
2. **Use `#[from]` for error composition** to enable seamless `?` operator usage
3. **Define module-local `Result<T>` alias**: `type Result<T> = std::result::Result<T, MyError>;`
4. **Lowercase error messages**: `#[error("invalid memory kind: {0}")]` not `"Invalid memory kind"`
5. **Use `#[error(transparent)]`** when wrapping errors without adding context

```rust
#[derive(Debug, Error, Clone)]
pub enum MemoryControllerError {
    #[error("memory context failed: {0}")]
    MemoryContextError(#[from] MemoryContextError),

    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

type Result<T> = std::result::Result<T, MemoryControllerError>;
```

### Module Organization

1. **Re-export public types from `lib.rs`**: `pub use crate::{module_a::*, module_b::*};`
2. **Use `pub(crate)` for internal APIs** that shouldn't be publicly exposed
3. **Use `mod.rs` pattern** for directories with multiple related modules

### Types and Structs

1. **Use `TypedBuilder`** for structs with multiple optional fields
2. **Use newtype pattern** for constrained values (e.g., `Credence(f32)` for 0.0-1.0 range)
3. **Derive order**: `Debug`, schema, `Clone`/`Copy`, `Serialize`/`Deserialize`, `Default`, `PartialEq`/`Eq`
4. **Use `#[serde(tag = "type")]`** for polymorphic enum serialization

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum LifecycleState { ... }

#[derive(TypedBuilder, Serialize, Deserialize)]
pub struct Query {
    limit: u32,
    context: MemoryContext,
    #[builder(default = false)]
    include_archived: bool,
    #[builder(default, setter(strip_option))]
    vector: Option<Vec<f32>>,
}
```

### Async Patterns

1. **Use `#[async_trait]`** for async trait methods
2. **Use `tokio::sync::OnceCell`** for lazily-initialized async singletons
3. **Use `backon`** for retry logic with exponential backoff
4. **Use `lazy_static!`** for global configuration and compiled regex

### Naming Conventions

**Type suffixes:**
- `*Error` for error types
- `*Request` / `*Response` for API payloads
- `*Builder` for builder types
- `*Config` for configuration
- `*Base` for trait definitions

**Method naming:**
- `new()` for constructors
- Simple getters without `get_` prefix: `context()`, `summary()`
- `get_*` for computed accessors: `get_id()`, `get_certainty()`
- `is_*` / `has_*` for boolean queries: `is_active()`, `has_user()`
- `as_str()` for string conversion
- `for_*` for factory methods: `for_user()`, `for_agent()`

**Parameters:**
- Use `impl Into<String>` for owned string parameters

```rust
pub fn new(summary: impl Into<String>) -> Result<Self> {
    let summary = summary.into();
    // ...
}
```

### Import Organization

Group imports in order: std library, external crates (alphabetical), workspace crates.

```rust
use std::sync::Arc;

use chrono::Utc;
use thiserror::Error;
use typed_builder::TypedBuilder;

use umem_core::{Memory, MemoryContext, Query};
```

### Builder Pattern

1. Methods take `mut self` and return `Self` for chaining
2. Validate in `build()` method, returning `Result<T>`

### Service Implementation

1. Use unit structs for stateless services: `pub struct MemoryController;`
2. Map errors at service boundaries (gRPC, MCP) to appropriate status types

### Testing

1. Use `#[tokio::test]` or `#[tokio::test(flavor = "multi_thread")]` for async tests
2. Place `#[cfg(test)] mod tests` at end of file
3. Use `super::*` to import from parent module in tests

## Project Structure

```
crates/
├── umem_core/          # Core memory types (Memory, MemoryContext, Query, etc.)
├── umem_controller/    # Memory operations controller
├── umem_ai/            # AI provider integrations (OpenAI, Anthropic, etc.)
├── umem_vector_store/  # Vector store backends (Qdrant, pgvector)
├── umem_embeddings/    # Embedding generation
├── umem_config/        # Configuration management
├── umem_grpc_server/   # gRPC service implementation
├── umem_mcp/           # MCP (Model Context Protocol) service
├── umem_proto/         # Protocol buffer definitions
└── ...
src/
├── bin/                # Binary entry points (grpc.rs, mcp.rs)
├── memory_machine/     # Memory machine implementation
└── lib.rs              # Main library exports
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `thiserror` | Error type derivation |
| `typed-builder` | Builder pattern macro |
| `serde` / `serde_json` | Serialization |
| `async-trait` | Async trait methods |
| `lazy_static` | Global singletons |
| `qdrant-client` | Vector database client |
| `reqwest` | HTTP client |
| `tracing` | Logging/instrumentation |

## Environment

Copy `.env.example` to `.env` and configure required variables for local development.
