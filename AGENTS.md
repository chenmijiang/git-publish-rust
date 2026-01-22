# AGENTS.md - Git Publish Development Guide

**git-publish** is a Rust CLI for creating and pushing git tags based on conventional commit analysis with semantic versioning.

**Tech Stack:** Rust 2021, clap (CLI), serde (TOML), git2, anyhow/thiserror (errors)

## Build, Lint, and Test

```bash
# Build & Run
cargo build && cargo run -- --help

# All tests
cargo test

# Single test by name
cargo test test_load_default_config

# Specific test file
cargo test --test config_test

# Full validation
cargo fmt && cargo clippy -- -D warnings && cargo test
```

## Code Style Guidelines

### Imports & Organization

- **Order**: Standard library → External crates → Internal modules
- Group imports with blank lines separating categories
- Use explicit imports (avoid glob imports `*`)
- Place `use` statements at module top before other code

**Example:**
```rust
use std::collections::HashMap;
use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config;
use crate::version;
```

### Formatting & Naming

- `snake_case` for functions and variables
- `PascalCase` for types, structs, enums
- `UPPER_SNAKE_CASE` for constants
- Descriptive names: `parse_conventional_commit` not `parse_cc`
- Use `cargo fmt` (enforced)

### Types & Generics

- Required type annotations on public functions/structs
- Derive traits: `#[derive(Debug, Clone, PartialEq)]` for all public types

**Example:**
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub branches: HashMap<String, String>,
    #[serde(default)]
    pub conventional_commits: ConventionalCommitsConfig,
}
```

### Error Handling

**Use anyhow for application errors:**
```rust
use anyhow::{Result, Context};

fn load_config(path: &str) -> Result<Config> {
    let contents = fs::read_to_string(path)
        .context("Failed to read config file")?;
    Ok(config)
}
```

**Never panic in production code** - return `Result<T>` instead.

### Testing

- Unit tests in `src/` files: `#[cfg(test)] mod tests { ... }`
- Integration tests in `tests/` directory as separate `.rs` files
- Use `tempfile::NamedTempFile` for fixtures

**Example:**
```rust
#[test]
fn test_load_default_config() {
    let config = Config::default();
    assert_eq!(config.branches.get("main"), Some(&"v{version}".to_string()));
}
```

### Documentation & Comments

- Doc comments for all public APIs
- Inline comments only for complex logic (explain why, not what)
- No obvious comments like `i += 1; // increment i`

### Module Organization

- **src/lib.rs** - Public API exports
- **src/main.rs** - CLI entry point, argument parsing
- **src/config.rs** - Configuration loading and structures
- **src/conventional.rs** - Conventional commit parsing
- **src/git_ops.rs** - Git operations (tagging, pushing)
- **src/version.rs** - Version parsing and bumping
- **src/ui.rs** - User interaction (prompts, formatting)

## Key Standards Summary

1. **Format before commit**: Run `cargo fmt && cargo clippy -- -D warnings && cargo test`
2. **Always test**: New code requires tests in `tests/` or inline `#[test]`
3. **Use Result<T>**: Never unwrap outside tests
4. **Document public APIs**: All pub fns/types need doc comments
5. **Handle errors gracefully**: Use anyhow context for application errors
6. **Follow Rust conventions**: The Rust Book and API Guidelines
