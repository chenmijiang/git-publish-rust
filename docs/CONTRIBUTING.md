# Contributing to git-publish

Thank you for your interest in contributing! This guide explains how to work with the git-publish codebase.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Setup](#development-setup)
3. [Code Organization](#code-organization)
4. [Development Workflow](#development-workflow)
5. [Code Style](#code-style)
6. [Testing](#testing)
7. [Commit Messages](#commit-messages)
8. [Pull Requests](#pull-requests)

## Getting Started

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs/)
- Git
- macOS, Linux, or Windows

### Clone the Repository

```bash
git clone https://github.com/anomalyco/git-publish.git
cd git-publish
```

### Verify Setup

```bash
# Build
cargo build

# Run tests
cargo test --lib

# Check format and lints
cargo fmt --check
cargo clippy -- -D warnings

# Run the tool
cargo run -- --help
```

## Development Setup

### IDE Setup (VS Code)

**Install extensions**:
- rust-analyzer
- Even Better TOML

**.vscode/settings.json**:
```json
{
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### IDE Setup (IntelliJ/CLion)

- Built-in Rust plugin works great
- Enable `Format code on Save`

## Code Organization

### Module Structure

```
src/
├── error.rs              # Unified error handling
├── domain/               # Pure business logic
│   ├── version.rs        # Semantic versioning
│   ├── commit.rs         # Conventional commits
│   ├── tag.rs            # Git tag patterns
│   ├── branch.rs         # Branch detection
│   └── prerelease.rs     # Pre-release versions
├── git/                  # Git abstraction
│   ├── repository.rs     # Repository trait
│   └── mock.rs           # MockRepository
├── analyzer/             # Version analysis
│   └── version_analyzer.rs
├── config.rs             # Configuration
├── ui/                   # User interface
│   ├── mod.rs            # Interactive functions
│   └── formatter.rs      # Formatting functions
└── hooks/                # Lifecycle hooks
    ├── lifecycle.rs      # Hook types
    └── executor.rs       # Hook execution
```

### Adding a New Module

1. **Create the file**: `src/new_module.rs`
2. **Declare in lib.rs**: `pub mod new_module;`
3. **Add public API**:
   ```rust
   pub struct MyType {
       pub field: String,
   }
   
   impl MyType {
       pub fn new(field: impl Into<String>) -> Self {
           MyType { field: field.into() }
       }
   }
   ```
4. **Add tests**: Inline `#[cfg(test)] mod tests { ... }`
5. **Add documentation**: Doc comments for all public items

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/my-feature
```

### 2. Make Changes

```bash
# Edit files
# Run tests frequently
cargo test --lib

# Format before committing
cargo fmt

# Check for issues
cargo clippy -- -D warnings
```

### 3. Write Tests First (TDD)

Follow the Red-Green-Refactor cycle:

```rust
#[test]
fn test_new_behavior() {
    // RED: Write test first (fails)
    let result = my_function(input);
    assert_eq!(result, expected);
}

// GREEN: Write minimal code to pass
fn my_function(input: &str) -> String {
    "result".to_string()
}

// REFACTOR: Clean up while keeping tests passing
```

### 4. Add Documentation

```rust
/// Brief description of what this does
///
/// Longer explanation if needed.
///
/// # Arguments
/// * `name` - What this parameter is for
///
/// # Returns
/// * `Ok(T)` - Success case
/// * `Err` - Error case
///
/// # Examples
/// ```
/// let result = my_function("input");
/// assert_eq!(result.unwrap(), "expected");
/// ```
pub fn my_function(name: &str) -> Result<String> {
    Ok("result".to_string())
}
```

### 5. Run Quality Checks

```bash
# Format
cargo fmt

# Lint
cargo clippy -- -D warnings

# Test
cargo test --lib

# Build
cargo build

# All at once
cargo fmt && cargo clippy -- -D warnings && cargo test --lib && cargo build
```

### 6. Commit and Push

```bash
git add .
git commit -m "feat: add new feature"
git push origin feature/my-feature
```

## Code Style

### Naming Conventions

```rust
// Modules and files: snake_case
mod version_analyzer { }

// Types and structs: PascalCase
pub struct VersionAnalyzer { }

// Functions and variables: snake_case
fn get_version() { }

// Constants: UPPER_SNAKE_CASE
const MAX_RETRIES: u32 = 3;
```

### Imports Organization

```rust
// Order:
// 1. Standard library
// 2. External crates
// 3. Internal modules

use std::fs;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::Version;
use crate::error::GitPublishError;
```

### Error Handling

Always use `Result<T>` instead of unwrap:

```rust
// ✗ Bad
fn load_version(path: &str) -> Version {
    let content = fs::read_to_string(path).unwrap();
    Version::parse(&content).unwrap()
}

// ✓ Good
fn load_version(path: &str) -> Result<Version> {
    let content = fs::read_to_string(path)
        .context("Failed to read version file")?;
    Version::parse(&content)
}
```

### Type Annotations

All public function signatures must have type annotations:

```rust
// ✗ Bad
pub fn parse(s) {
    // ...
}

// ✓ Good
pub fn parse(s: &str) -> Result<Version> {
    // ...
}
```

### Doc Comments

```rust
// ✗ Bad: No documentation
pub fn bump(&self, bump_type: &VersionBump) -> Self {
    // ...
}

// ✓ Good: Clear documentation
/// Bump the version according to the bump type
///
/// # Arguments
/// * `bump_type` - The type of version bump (Major/Minor/Patch)
///
/// # Returns
/// A new Version with the bump applied
pub fn bump(&self, bump_type: &VersionBump) -> Self {
    // ...
}
```

## Testing

### Unit Tests (Domain Layer)

Place in the module file with `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_parse() {
        let v = Version::parse("v1.2.3").unwrap();
        assert_eq!(v.major, 1);
    }
}
```

### Running Tests

```bash
# All tests
cargo test --lib

# Specific module
cargo test --lib domain::version::tests

# Single test
cargo test --lib test_version_parse -- --exact

# With output
cargo test --lib -- --nocapture
```

### Test Coverage

- All public functions must have tests
- Test both success and error cases
- Test edge cases (empty strings, boundary values, etc.)
- Test integration between modules

### MockRepository for Testing

```rust
#[test]
fn test_with_git_operations() {
    let mut repo = MockRepository::new();
    
    let oid = Oid::from_bytes(&[1; 20]).unwrap();
    repo.add_tag("v1.0.0", oid);
    repo.set_branch_head("main", oid);
    
    assert!(repo.find_tag_oid("v1.0.0").unwrap().is_some());
}
```

## Commit Messages

Follow conventional commits format:

```
type(scope): subject

body

footer
```

### Type
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Build, dependencies, etc.

### Examples

```
feat(prerelease): add iteration number support

Adds ability to track iterations like beta.1, beta.2
for pre-release versions following semver spec.

Fixes #123
```

```
fix(analyzer): handle non-conventional commits

Previously crashed when encountering commits that
don't follow conventional format. Now treats as chore.

Fixes #456
```

```
test(domain): add version bump integration tests

Adds comprehensive tests for version bumping workflow
with pre-release versions and various bump types.
```

## Pull Requests

### PR Title Format

Use conventional commits format:
```
feat: add new feature
fix: resolve issue with version parsing
docs: update hooks guide
```

### PR Checklist

Before submitting PR, ensure:

- [ ] All tests pass: `cargo test --lib`
- [ ] No compiler warnings: `cargo clippy -- -D warnings`
- [ ] Code formatted: `cargo fmt`
- [ ] Documentation added for public items
- [ ] Commit messages follow conventional commits
- [ ] Related issue linked in PR description

### PR Description Template

```markdown
## Summary
Brief description of changes.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation

## Testing
How was this tested? What scenarios were covered?

## Related Issue
Fixes #123

## Additional Context
Any other context or screenshots.
```

## Running Full Validation

Before pushing, run complete validation:

```bash
#!/bin/bash
# full-validation.sh

set -e

echo "🔍 Running format check..."
cargo fmt --check

echo "🔍 Running clippy..."
cargo clippy -- -D warnings

echo "🧪 Running tests..."
cargo test --lib

echo "🏗️  Running build..."
cargo build

echo "✅ All checks passed!"
```

Save as `.git/hooks/pre-commit` to run automatically:

```bash
chmod +x .git/hooks/pre-commit
```

## Troubleshooting

### Cargo Cache Issues

```bash
cargo clean
cargo build
```

### Test Failures After Code Changes

```bash
# Run tests with output
cargo test --lib -- --nocapture

# Run single failing test
cargo test --lib test_name -- --exact
```

### Format Issues

```bash
# Automatically fix
cargo fmt

# Check what would change
cargo fmt -- --check
```

### Clippy Warnings

```bash
# See detailed explanation
cargo clippy -- -D warnings

# Some can be auto-fixed
cargo fix --allow-dirty --allow-staged
```

---

**Contributing Guide Version**: 1.0  
**Last Updated**: 2025-01-23

Questions? Create an issue or ask in the repository discussions.
