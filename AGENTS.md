# MiniKV Project Guide for AI Agents

## Project Overview

MiniKV is a **distributed key-value store** written in Rust using Tokio async runtime. It provides:
- Strong consistency via Raft consensus
- Dual deployment modes: single-node and cluster
- Pluggable storage engines (In-Memory, Disk, Sled)
- Dual API protocols: HTTP RESTful and gRPC

**Current Status**: Design phase - architecture documented, implementation pending.

## Build Commands

```bash
# Build project
cargo build

# Build in release mode
cargo build --release

# Build with specific features
cargo build --features rocksdb-backend  # Use RocksDB storage
cargo build --features sled-backend      # Use Sled storage

# Check code without building
cargo check

# Generate documentation
cargo doc --open
```

## Test Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests in a specific module
cargo test --module module_name

# Run integration tests
cargo test --test integration_test

# Run benchmarks
cargo bench

# Run tests with verbose output
cargo test -- --nocapture
```

## Lint and Format

```bash
# Format code
cargo fmt

# Check formatting without applying
cargo fmt -- --check

# Run linter
cargo clippy

# Run clippy with all warnings
cargo clippy -- -W clippy::all

# Run clippy on specific file
cargo clippy -- src/module/file.rs
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
use crate::common::types::{Key, Value};
use crate::errors::Result;
use crate::storage::StorageEngine;
```

### Naming Conventions

- **Types (struct, enum, trait)**: PascalCase (e.g., `StorageEngine`, `RaftConsensus`)
- **Functions and variables**: snake_case (e.g., `get_key`, `storage_engine`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `MAX_CONNECTIONS`, `DEFAULT_PORT`)
- **Modules**: snake_case (e.g., `storage_engine`, `raft_consensus`)
- **Type parameters**: single uppercase letter or PascalCase (e.g., `T`, `K`, `V`, `Consensus`)

### Error Handling

**Use `thiserror` for library errors**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;
```

**Use `anyhow` for application errors**:
```rust
use anyhow::{Context, Result};

fn load_config() -> Result<Config> {
    let content = std::fs::read_to_string("config.toml")
        .context("Failed to read config file")?;
    let config: Config = toml::from_str(&content)
        .context("Failed to parse config")?;
    Ok(config)
}
```

### Async Code Patterns

**Use `async_trait` for async traits**:
```rust
use async_trait::async_trait;

#[async_trait]
pub trait StorageEngine: Send + Sync {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
}
```

**Prefer `tokio::spawn` over `std::thread::spawn`**:
```rust
// Good
tokio::spawn(async move {
    process_request(request).await;
});

// Avoid (unless necessary for blocking operations)
std::thread::spawn(|| {
    blocking_operation();
});
```

### Trait Design

**Use generics over `dyn Trait` for performance**:
```rust
// Preferred: Generic
pub struct Engine<C: Consensus, S: StorageEngine> {
    consensus: C,
    storage: S,
}

// Avoid: Dynamic dispatch (5-10% performance overhead)
pub struct Engine {
    consensus: Box<dyn Consensus>,
    storage: Box<dyn StorageEngine>,
}
```

## Architecture Layers

```
API Layer (src/api/)
    ↓ calls
Engine Layer (src/engine/)
    ↓ coordinates
Consensus Layer (src/consensus/) ← trait abstraction
    ↓ applies to
Storage Layer (src/storage/) ← trait abstraction
```

### Module Responsibilities

1. **api/**: HTTP (Axum) + gRPC (Tonic) handlers
2. **engine/**: Core business logic, command building
3. **consensus/**: Consensus trait + implementations (NoOp/Raft)
4. **storage/**: StorageEngine trait + implementations (Memory/Disk)
5. **network/**: Raft network transport (cluster mode only)
6. **config/**: Configuration structures and validation
7. **common/**: Shared types, constants, type aliases
8. **errors/**: Error types and conversions

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

### Consensus Trait

```rust
#[async_trait]
pub trait Consensus: Send + Sync + Clone + 'static {
    async fn propose(&self, cmd: Command) -> Result<Vec<u8>>;
    async fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    async fn read_local(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    async fn leader(&self) -> Option<u64>;
    async fn is_leader(&self) -> bool;
}
```

## Dependencies and Features

### Core Dependencies
- **tokio**: Async runtime (features: "full")
- **axum**: HTTP framework
- **tonic**: gRPC framework
- **async-raft**: Raft consensus implementation
- **serde/bincode**: Serialization
- **thiserror/anyhow**: Error handling
- **tracing**: Structured logging

### Optional Features
- `disk-storage` (default): Enable disk storage backend
- `rocksdb-backend`: Use RocksDB storage engine
- `sled-backend`: Use Sled storage engine

## Configuration

**Config files**: `config/single.toml` (single-node), `config/cluster.toml` (cluster)

**Runtime config**:
```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub consensus: ConsensusConfig,
    pub logging: LoggingConfig,
}
```

## Testing Strategy

1. **Unit tests**: In-module `#[cfg(test)] mod tests { ... }`
2. **Integration tests**: `tests/integration_test.rs`
3. **Benchmarks**: `benches/benchmark.rs` using Criterion
4. **Test coverage**: Use `cargo tarpaulin` (if installed)

**Testing patterns**:
- Use `tokio-test` for async tests
- Mock dependencies via trait objects in tests only
- Test both single-node and cluster modes
- Test failure scenarios (network partition, node failure)

## Development Workflow

1. **Check code**: `cargo check` (fast feedback)
2. **Format**: `cargo fmt`
3. **Lint**: `cargo clippy -- -W clippy::all`
4. **Test**: `cargo test`
5. **Document**: Update docs if API changes

## Important Files

- `Cargo.toml`: Project dependencies and features
- `docs/architecture/`: Architecture documentation (read first!)
- `proto/`: Protobuf definitions for gRPC
- `config/`: Example configuration files
- `.gitignore`: Ignore patterns (target/, *.log, data/, etc.)

## Common Patterns

### Running the Application

```bash
# Single-node mode
cargo run -- --config config/single.toml

# Cluster mode (run 3 nodes)
cargo run -- --config config/node1.toml
cargo run -- --config config/node2.toml
cargo run -- --config config/node3.toml
```

### Adding New Storage Backend

1. Implement `StorageEngine` trait in `src/storage/new_backend.rs`
2. Add feature flag in `Cargo.toml`
3. Add backend selection in `src/storage/mod.rs`
4. Update configuration structures in `src/config/`
5. Add integration tests in `tests/`

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Enable tracing for specific module
RUST_LOG=minikv::storage=trace cargo run

# Generate flamegraph (if installed)
cargo flamegraph --root
```

## References

- Architecture: `docs/architecture/README.md`
- API docs: `docs/api.md` (when created)
- Rust async book: https://rust-lang.github.io/async-book/
- Tokio tutorial: https://tokio.rs/tokio/tutorial

---

**Note**: This project follows strict Rust best practices. Avoid:
- `unwrap()` in production code (use `?` or `expect()` with context)
- `dyn Trait` in hot paths (use generics)
- Blocking operations in async context (use `spawn_blocking`)
- Adding unnecessary comments (code should be self-documenting)
- Type suppression (`as any`, `#[allow(type_complexity)]`)