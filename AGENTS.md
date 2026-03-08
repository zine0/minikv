# MiniKV Project Guide for AI Agents

## Project Overview

MiniKV is a **distributed key-value store** built with Rust and Tokio. It provides:
- Strong consistency via Raft consensus
- Dual deployment modes: single-node and cluster
- Pluggable storage engines (In-Memory, Disk)
- Dual API protocols: HTTP RESTful and gRPC

## Build Commands

```bash
# Build project (uses edition=2024)
cargo build

# Build in release mode
cargo build --release

# Build with specific features
cargo build --features rocksdb-backend
cargo build --features sled-backend

# Check code without building (fast feedback)
cargo check

# Generate documentation
cargo doc --open
```

## Test Commands

```bash
# Run all tests
cargo test

# Run a specific test by name
cargo test test_name          # matches any test containing "test_name"
cargo test module_name::test_name  # exact module path

# Run tests with output
cargo test -- --nocapture

# Run only doc tests
cargo test --doc

# Run benchmarks
cargo bench
```

## Lint and Format

```bash
# Format code
cargo fmt

# Check formatting without applying
cargo fmt -- --check

# Run linter with all warnings
cargo clippy -- -W clippy::all

# Run clippy on specific file
cargo clippy -- src/storage/engine.rs
```

## Code Style Guidelines

### Imports Organization

```rust
// Standard library imports first
use std::collections::HashMap;
use std::sync::Arc;

// External crate imports
use async_trait::async_trait;
use tokio::sync::RwLock;
use thiserror::Error;

// Internal module imports (use `crate::`)
use crate::errors::{KvError, Result};
use crate::storage::StorageEngine;
```

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Types (struct/enum/trait) | PascalCase | `StorageEngine`, `KvError` |
| Functions/variables | snake_case | `get_key`, `storage_engine` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_CONNECTIONS` |
| Modules | snake_case | `storage_engine`, `raft_consensus` |

### Type Conventions

- **Keys**: Use `&[u8]` for read operations (zero-copy), `Vec<u8>` for writes
- **Values**: Use `Vec<u8>` for storage layer, can use `bytes::Bytes` internally for zero-copy
- **Always add trait bounds**: `Send + Sync + Clone + 'static` for traits

### Error Handling

```rust
// Use thiserror for library errors (src/errors/)
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvError {
    #[error("Key not found: {0}")]
    NotFound(String),
    
    #[error("Storage I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, KvError>;

// Use anyhow for application errors
use anyhow::{Context, Result};

fn load_config() -> Result<Config> { ... }
```

### Async Code Patterns

```rust
// Use async_trait for async traits
#[async_trait]
pub trait StorageEngine: Send + Sync + Clone + 'static {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
}

// Use tokio::spawn for async tasks
tokio::spawn(async move { process(request).await });
```

### Trait Design

```rust
// Prefer generics over dyn Trait for performance
pub struct Engine<C: Consensus, S: StorageEngine> {
    consensus: C,
    storage: S,
}
```

## Architecture Layers

```
API Layer (src/api/)     → Engine Layer (src/engine/)
        ↓                              ↓
Consensus Layer (src/consensus/) → Storage Layer (src/storage/)
```

### Module Responsibilities

| Module | Responsibility |
|--------|----------------|
| `api/` | HTTP (Axum) + gRPC (Tonic) handlers |
| `engine/` | Core business logic, command building |
| `consensus/` | Consensus trait + implementations (NoOp/Raft) |
| `storage/` | StorageEngine trait + implementations |
| `config/` | Configuration structures |
| `errors/` | Error types |
| `common/` | Shared types, constants |

## Key Traits

### StorageEngine Trait

```rust
#[async_trait]
pub trait StorageEngine: Send + Sync + Clone + 'static {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    async fn delete(&self, key: &[u8]) -> Result<()>;
    async fn scan(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
    async fn batch(&self, ops: Vec<Op>) -> Result<()>;
    async fn snapshot(&self) -> Result<Vec<u8>>;
    async fn restore(&self, snapshot: Vec<u8>) -> Result<()>;
}
```

## Development Workflow

```bash
cargo check      # Fast feedback
cargo fmt        # Format code
cargo clippy -- -W clippy::all  # Lint
cargo test       # Test
```

## Zero-Copy Design Principles

- **Key parameters**: Use `&[u8]` for read-only operations
- **Value storage**: Internally use `bytes::Bytes` for zero-copy
- **Avoid unnecessary allocations**: Prefer references when possible

## Prohibited Patterns

- `unwrap()` in production (use `?` or `expect()` with context)
- `dyn Trait` in hot paths (use generics)
- Type suppression (`as any`, `@ts-ignore`)
- Empty catch blocks

---

**Read first**: `docs/architecture/` for full architecture details.