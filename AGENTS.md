# AGENTS.md - Development Guide for Agentic Coding

This guide provides essential commands, code style guidelines, and patterns for agentic coding in git-publish.

## Quick Commands

### Running Tests
```bash
# Run all unit tests
cargo test --lib

# Run a single test
cargo test --lib test_name -- --exact

# Run tests in a specific module
cargo test --lib domain::version::tests

# Run with output
cargo test --lib -- --nocapture

# Run tests sequentially (required for tests with serial_test)
cargo test --lib -- --test-threads=1
```

### Code Quality
```bash
# Format code (auto-fixes)
cargo fmt

# Check formatting without changes
cargo fmt --check

# Run linter (strict: denies all warnings)
cargo clippy -- -D warnings

# Build project
cargo build

# Complete validation pipeline
cargo fmt && cargo clippy -- -D warnings && cargo test --lib && cargo build
```

## Code Style Guidelines

### Module & Import Organization
1. **Standard library imports first**, then external crates, then internal modules
2. **Use absolute paths** for internal imports: `use crate::domain::Version;`
3. **Order imports**: std → external → crate
4. **Group related imports** together

Example:
```rust
use std::fmt;
use regex::Regex;
use thiserror::Error;

use crate::domain::PreRelease;
use crate::error::{GitPublishError, Result};
```

### Naming Conventions
- **Modules & files**: `snake_case` (e.g., `version_analyzer.rs`)
- **Types & structs**: `PascalCase` (e.g., `VersionAnalyzer`)
- **Functions & variables**: `snake_case` (e.g., `analyze_messages()`)
- **Constants**: `UPPER_SNAKE_CASE` (e.g., `MAX_RETRIES`)

### Type Annotations
- **All public functions** must have explicit type annotations
- **Return types** always annotated, even for `()`
- Use `Result<T>` type alias instead of `std::result::Result<T, GitPublishError>`

Example:
```rust
pub fn parse(tag: &str) -> Result<Version> {
    // implementation
}

fn analyze_messages(&self, messages: &[String]) -> VersionBump {
    // implementation
}
```

### Error Handling
- **Never use `unwrap()` or `panic!()`** in library code
- **Use `Result<T>` for fallible operations**
- **Use `?` operator** for error propagation
- **Use `GitPublishError` helper methods**: `.version()`, `.config()`, `.tag()`, `.remote()`
- **Provide context** in error messages

Good:
```rust
fn load_version(path: &str) -> Result<Version> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| GitPublishError::Io(e))?;
    Version::parse(&content)
}
```

Bad:
```rust
fn load_version(path: &str) -> Version {
    let content = std::fs::read_to_string(path).unwrap();
    Version::parse(&content).unwrap()
}
```

### Documentation
- **All public items** must have doc comments (`///`)
- **Include descriptions** of what, not just how
- **Add examples** for complex functions
- **Note panics** if any exist (though they shouldn't in library code)

Example:
```rust
/// Bump the version according to the bump type
///
/// Applies the specified bump to create a new version.
/// Pre-release versions are cleared on major/minor/patch bumps.
pub fn bump(&self, bump_type: &VersionBump) -> Self {
    // implementation
}
```

### Testing
- **Tests live inline** with `#[cfg(test)] mod tests { ... }`
- **Test both success and error cases**
- **Use descriptive test names**: `test_<function>_<scenario>`
- **Test public functions** at minimum; private functions rarely need tests
- **Use `assert_eq!` for equality**, other assertions for boolean checks

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_version() {
        let v = Version::parse("v1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_parse_invalid_version_fails() {
        let result = Version::parse("invalid");
        assert!(result.is_err());
    }
}
```

## Key Patterns

### Result Type Alias
```rust
// Always use this in git-publish
pub type Result<T> = std::result::Result<T, GitPublishError>;
```

### Conventional Commits Parsing
- Type must be lowercase
- Scope is optional in `()`
- `!` or `BREAKING CHANGE:` indicates breaking changes
- Format: `type(scope)!: description`

### Version Representation
- Stored as `MAJOR.MINOR.PATCH-PRERELEASE` (semver compliant)
- Pre-release versions handled via `PreRelease` struct
- Parsing and display via `Version` impl

### Configuration
- Load from `gitpublish.toml` or defaults
- Uses `serde` for deserialization
- Validate on load, not later

## Project Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Public API exports
├── error.rs             # Unified error type
├── config.rs            # Configuration loading
├── domain/              # Pure business logic (zero dependencies)
├── analyzer/            # Version analysis logic
├── git_ops.rs           # Git abstraction
├── ui/                  # User interface
└── boundary.rs          # Boundary warnings

tests/                   # Integration tests
```

## Linting & Formatting Rules

- **Rust Edition**: 2021
- **Clippy**: All warnings denied (`-D warnings`)
- **Rustfmt**: Default settings (auto-run with `cargo fmt`)
- **No unsafe code** without explicit justification
- **No external build scripts**

---

Generated: 2025-01-23
