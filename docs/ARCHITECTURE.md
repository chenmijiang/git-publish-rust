# git-publish Architecture Guide

This document describes the architecture of git-publish, a Rust CLI tool for automated semantic versioning via conventional commits analysis.

## Table of Contents

1. [Overview](#overview)
2. [Core Principles](#core-principles)
3. [Architecture Layers](#architecture-layers)
4. [Module Structure](#module-structure)
5. [Data Flow](#data-flow)
6. [Key Components](#key-components)
7. [Design Patterns](#design-patterns)
8. [Extension Points](#extension-points)

## Overview

git-publish automates semantic versioning (MAJOR.MINOR.PATCH) by:
1. Analyzing conventional commit messages
2. Determining version bump type (major/minor/patch)
3. Creating and pushing git tags with proper version numbers
4. Supporting pre-release versions (alpha, beta, rc, custom)
5. Executing hooks at key lifecycle points

### High-Level Architecture

```
┌─────────────────────────────────────────────────┐
│             CLI Entry Point (main.rs)           │
└──────────────────┬──────────────────────────────┘
                   │
         ┌─────────▼──────────┐
         │  CLI Orchestration │
         │  (orchestration.rs)│
         └─────────┬──────────┘
                   │
        ┌──────────┴──────────┐
        │                     │
   ┌────▼────┐         ┌─────▼──────┐
   │ Analyzer│         │ Git Ops    │
   │ (domain)│         │ (git/)     │
   └─────────┘         └────────────┘
        │                     │
   ┌────▼──────────────────────▼────┐
   │  Domain Model (domain/)         │
   │  - Version                      │
   │  - ParsedCommit                 │
   │  - Tag                          │
   │  - Branch                       │
   │  - PreRelease                   │
   └─────────────────────────────────┘
```

## Core Principles

### 1. **Separation of Concerns**
Each module has a single, well-defined responsibility:
- **Domain**: Pure business logic (no I/O, no external dependencies)
- **Git**: Git operations abstraction layer
- **Config**: Configuration loading and validation
- **Analyzer**: Version bump decision logic
- **UI**: User interactions and formatting
- **Hooks**: Lifecycle hook execution
- **Errors**: Unified error handling

### 2. **Pure Domain Logic**
The `domain/` module contains zero external dependencies:
- No git2 imports
- No I/O operations
- No configuration dependencies
- Testable in isolation with deterministic results

### 3. **Abstraction Layers**
- **Repository Trait**: Allows swapping git implementations (git2, GitLab API, etc.)
- **MockRepository**: Enables testing without real git operations
- **Configuration**: Externalized settings prevent hardcoding

### 4. **Error Handling**
- Single unified `GitPublishError` enum (thiserror)
- Automatic conversions from git2::Error and io::Error
- Contextual error messages with descriptive prefixes

### 5. **Type Safety**
- Strong typing prevents mixing version numbers, tag names, branch names
- No "stringly typed" APIs
- Compile-time guarantees for version format

## Architecture Layers

### Layer 1: Domain (Pure Business Logic)
**Location**: `src/domain/`
**Dependencies**: None (zero external crates)
**Responsibility**: Core version and commit logic

```rust
// Pure function, no side effects
pub fn parse(tag: &str) -> Result<Version>

// Deterministic, testable
pub fn bump(&self, bump_type: &VersionBump) -> Version
```

**Modules**:
- `version.rs`: Semantic versioning (MAJOR.MINOR.PATCH-PRERELEASE)
- `commit.rs`: Conventional commit parsing
- `tag.rs`: Git tag pattern matching and formatting
- `branch.rs`: Branch context detection
- `prerelease.rs`: Pre-release version handling

### Layer 2: Git Abstraction (Repository Pattern)
**Location**: `src/git/`
**Dependencies**: git2 (production), none (tests)
**Responsibility**: Git operations abstraction

```rust
pub trait Repository: Send {
    fn get_branch_head_oid(&self, branch: &str) -> Result<Oid>;
    fn get_commits_between(&self, from: Oid, to: Oid) -> Result<Vec<CommitInfo>>;
    fn find_tag_oid(&self, tag_name: &str) -> Result<Option<Oid>>;
    fn list_tags(&self) -> Result<Vec<String>>;
    fn create_tag(&self, name: &str, oid: Oid) -> Result<()>;
    fn push_tags(&self, remote: &str, tag_names: &[&str]) -> Result<()>;
}
```

**Implementations**:
- `Git2Repository`: Production implementation wrapping git2
- `MockRepository`: Test double with in-memory storage

### Layer 3: Analysis (Decision Logic)
**Location**: `src/analyzer/`
**Dependencies**: domain, config
**Responsibility**: Determine version bump from commits

```
Commits → Parse Types → Decision Tree → VersionBump
   ↓
  Breaking Changes? → Major
   ↓ No
  Features? → Minor
   ↓ No
  Fixes/Other? → Patch
```

### Layer 4: Configuration
**Location**: `src/config.rs`
**Format**: TOML
**Responsibility**: Load and validate settings

```toml
[branches]
main = "v{version}"
develop = "d{version}"

[prerelease]
enabled = true
default_identifier = "beta"
auto_increment = true

[hooks]
pre_tag_create = "./scripts/pre-tag-create.sh"
post_tag_create = "./scripts/post-tag-create.sh"
post_push = "./scripts/post-push.sh"
```

### Layer 5: Hooks (Extensibility)
**Location**: `src/hooks/`
**Responsibility**: Execute user scripts at lifecycle points

**Hook Types**:
- `pre-tag-create`: Before creating tag (can prevent creation)
- `post-tag-create`: After tag created locally
- `post-push`: After tags pushed to remote

**Environment Variables** passed to hooks:
```bash
GITPUBLISH_BRANCH="main"
GITPUBLISH_TAG_NAME="v1.2.3"
GITPUBLISH_REMOTE="origin"
GITPUBLISH_VERSION_BUMP="Minor"
GITPUBLISH_COMMIT_COUNT="5"
```

### Layer 6: UI (Presentation)
**Location**: `src/ui/`
**Responsibility**: Separate user interaction from logic

- `formatter.rs`: Pure formatting functions (no I/O)
- `mod.rs`: Interactive functions (prompts, selections)

### Layer 7: CLI Orchestration
**Location**: `src/cli/orchestration.rs`
**Responsibility**: Wire everything together, coordinate workflow

## Module Structure

```
src/
├── lib.rs                           # Public API exports
├── main.rs                          # CLI entry point
├── error.rs                         # Unified error handling (101 LOC)
│
├── domain/                          # Pure business logic (zero dependencies)
│   ├── mod.rs
│   ├── version.rs                   # Semantic versioning
│   ├── commit.rs                    # Conventional commit parsing
│   ├── tag.rs                       # Git tag patterns
│   ├── branch.rs                    # Branch detection
│   └── prerelease.rs                # Pre-release version support
│
├── git/                             # Git abstraction layer
│   ├── mod.rs
│   ├── repository.rs                # Repository trait + Git2Repository
│   └── mock.rs                      # MockRepository for testing
│
├── analyzer/                        # Version analysis
│   └── version_analyzer.rs          # Determines version bump
│
├── config.rs                        # Configuration (TOML)
│
├── ui/                              # User interface
│   ├── mod.rs                       # Interactive functions
│   └── formatter.rs                 # Formatting functions
│
├── hooks/                           # Lifecycle hooks
│   ├── mod.rs
│   ├── lifecycle.rs                 # Hook types and context
│   └── executor.rs                  # Hook execution
│
└── cli/
    └── orchestration.rs             # Workflow coordination
```

## Data Flow

### Version Bump Workflow

```
┌──────────────────────────┐
│ User runs: git-publish   │
└────────────┬─────────────┘
             │
┌────────────▼─────────────────────────┐
│ Load Config + Parse Arguments        │
└────────────┬─────────────────────────┘
             │
┌────────────▼─────────────────────────┐
│ Get Current Branch                   │
│ (Repository.get_branch_head_oid)     │
└────────────┬─────────────────────────┘
             │
┌────────────▼─────────────────────────┐
│ Find Last Tag on Branch              │
│ (Repository.list_tags, filter)       │
└────────────┬─────────────────────────┘
             │
┌────────────▼─────────────────────────┐
│ Get Commits Between Last Tag & HEAD  │
│ (Repository.get_commits_between)     │
└────────────┬─────────────────────────┘
             │
┌────────────▼─────────────────────────┐
│ Parse Commit Messages                │
│ (domain::ParsedCommit::parse)        │
└────────────┬─────────────────────────┘
             │
┌────────────▼─────────────────────────┐
│ Analyze Version Bump                 │
│ (VersionAnalyzer.analyze)            │
│ → VersionBump { Major | Minor | Patch}
└────────────┬─────────────────────────┘
             │
┌────────────▼─────────────────────────┐
│ Calculate New Version                │
│ (Version.bump)                       │
└────────────┬─────────────────────────┘
             │
┌────────────▼─────────────────────────┐
│ Format Tag Name                      │
│ (TagPattern.format)                  │
└────────────┬─────────────────────────┘
             │
┌────────────▼─────────────────────────┐
│ Execute pre-tag-create Hook          │
│ (hooks::HookExecutor.execute)        │
└────────────┬─────────────────────────┘
             │
┌────────────▼─────────────────────────┐
│ Create Tag                           │
│ (Repository.create_tag)              │
└────────────┬─────────────────────────┘
             │
┌────────────▼─────────────────────────┐
│ Execute post-tag-create Hook         │
└────────────┬─────────────────────────┘
             │
┌────────────▼─────────────────────────┐
│ Push Tags to Remote                  │
│ (Repository.push_tags)               │
└────────────┬─────────────────────────┘
             │
┌────────────▼─────────────────────────┐
│ Execute post-push Hook               │
└────────────┬─────────────────────────┘
             │
        ✅ Done
```

## Key Components

### 1. Version (domain::Version)
Represents semantic versioning: `MAJOR.MINOR.PATCH[-PRERELEASE]`

```rust
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<PreRelease>,
}

// Parsing: "v1.2.3-beta.1" → Version { major: 1, minor: 2, patch: 3, prerelease: Some(...) }
// Display: Version { major: 1, minor: 2, patch: 3, ... } → "1.2.3-beta.1"
// Bumping: bump(Major) → "2.0.0"
```

### 2. ParsedCommit (domain::ParsedCommit)
Extracts conventional commit structure

```rust
pub struct ParsedCommit {
    pub r#type: String,           // "feat", "fix", etc.
    pub scope: Option<String>,    // "(api)" → Some("api")
    pub description: String,      // What changed
    pub is_breaking_change: bool, // "!" or "BREAKING CHANGE:"
}

// Parses:
// - "feat(auth): add login" → { type: "feat", scope: Some("auth"), is_breaking: false }
// - "fix!: breaking change" → { type: "fix", is_breaking: true }
```

### 3. VersionAnalyzer (analyzer::VersionAnalyzer)
Decision tree for version bumping

```rust
pub fn analyze(&self, messages: &[String]) -> VersionBump {
    // Decision tree (priority order):
    // 1. Any breaking change? → Major
    // 2. Any feature (feat)? → Minor
    // 3. Otherwise → Patch
}
```

### 4. PreRelease (domain::PreRelease)
Supports alpha/beta/rc/custom identifiers with iteration

```rust
pub struct PreRelease {
    pub identifier: PreReleaseType,  // Alpha, Beta, ReleaseCandidate, Custom(...)
    pub iteration: Option<u32>,      // 1, 2, 3, ... (for beta.1, beta.2, etc.)
}

// Examples:
// - "alpha" → { identifier: Alpha, iteration: None }
// - "beta.1" → { identifier: Beta, iteration: Some(1) }
// - "staging.5" → { identifier: Custom("staging"), iteration: Some(5) }
```

### 5. Repository Trait (git::Repository)
Abstracts git operations for testability

```rust
pub trait Repository: Send {
    fn get_branch_head_oid(&self, branch_name: &str) -> Result<Oid>;
    fn get_commits_between(&self, from_oid: Oid, to_oid: Oid) -> Result<Vec<CommitInfo>>;
    fn find_tag_oid(&self, tag_name: &str) -> Result<Option<Oid>>;
    fn list_tags(&self) -> Result<Vec<String>>;
    fn create_tag(&self, name: &str, oid: Oid) -> Result<()>;
    fn push_tags(&self, remote: &str, tag_names: &[&str]) -> Result<()>;
    fn fetch_from_remote(&self, remote: &str, branch: &str) -> Result<()>;
}
```

**Why `Send` instead of `Sync`?**
- git2::Repository is `Send` but not `Sync` (thread-safe for move, not sharing)
- We create one Repository per thread/task, don't share across threads
- This respects git2's thread-safety model

### 6. HookContext (hooks::lifecycle::HookContext)
Context passed to hooks as environment variables

```rust
pub struct HookContext {
    pub hook_type: HookType,
    pub branch: String,
    pub tag: String,
    pub remote: String,
    pub version_bump: Option<String>,
    pub commit_count: Option<usize>,
}

// Maps to:
// GITPUBLISH_BRANCH="main"
// GITPUBLISH_TAG_NAME="v1.2.3"
// GITPUBLISH_REMOTE="origin"
// GITPUBLISH_VERSION_BUMP="Minor"
// GITPUBLISH_COMMIT_COUNT="5"
```

## Design Patterns

### 1. Repository Pattern
**Purpose**: Abstract git operations for testability

```rust
// Production
let repo = Git2Repository::open(".")?;

// Testing
let mut repo = MockRepository::new();
repo.add_tag("v1.0.0", oid);
```

### 2. Type Alias for Results
**Purpose**: Cleaner error handling

```rust
pub type Result<T> = std::result::Result<T, GitPublishError>;

// Instead of:
fn foo() -> std::result::Result<Version, GitPublishError>

// Use:
fn foo() -> Result<Version>
```

### 3. Builder Pattern (Config)
**Purpose**: Flexible configuration assembly

```toml
[branches]
main = "v{version}"

[prerelease]
enabled = true

[hooks]
pre_tag_create = "./scripts/pre.sh"
```

### 4. Strategy Pattern (VersionBump)
**Purpose**: Different version bumping strategies

```rust
pub enum VersionBump {
    Major,  // breaking changes
    Minor,  // new features
    Patch,  // bug fixes
}

version.bump(&strategy) → new_version
```

### 5. Command Pattern (Hooks)
**Purpose**: Execute actions at lifecycle points

```rust
HookExecutor::execute(HookType::PreTagCreate, context)
```

## Extension Points

### 1. Adding a New Repository Implementation
```rust
// src/git/github_api.rs
pub struct GitHubAPIRepository {
    client: GitHubClient,
}

impl Repository for GitHubAPIRepository {
    fn get_branch_head_oid(&self, branch: &str) -> Result<Oid> {
        // Call GitHub API
    }
    // ... implement other methods
}

// In main.rs:
let repo: Box<dyn Repository> = match source {
    GitSource::Local => Box::new(Git2Repository::open(".")?),
    GitSource::GitHub => Box::new(GitHubAPIRepository::new(...)?),
};
```

### 2. Adding a New Pre-Release Type
```rust
// In domain/prerelease.rs:
pub enum PreReleaseType {
    Alpha,
    Beta,
    ReleaseCandidate,
    Custom(String),
    // Add new type:
    Snapshot,
}

impl FromStr for PreReleaseType {
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            // ...
            "snapshot" => Ok(PreReleaseType::Snapshot),
            // ...
        }
    }
}
```

### 3. Adding a New Hook Type
```rust
// In hooks/lifecycle.rs:
pub enum HookType {
    PreTagCreate,
    PostTagCreate,
    PostPush,
    // Add new hook:
    PrePush,
}

// Add configuration support:
// In config.rs:
pub struct HooksConfig {
    pub pre_push: Option<String>,
    // ...
}
```

### 4. Custom Version Bump Logic
```rust
// Create alternate analyzer:
pub struct CustomVersionAnalyzer {
    rules: Vec<BumpRule>,
}

impl CustomVersionAnalyzer {
    pub fn analyze(&self, messages: &[String]) -> VersionBump {
        // Custom decision logic
    }
}
```

## Testing Strategy

### Unit Tests (Domain Layer)
- Pure functions with predictable inputs/outputs
- No mocks needed
- Fast execution
- High coverage

```rust
#[test]
fn test_version_parse() {
    let v = Version::parse("v1.2.3").unwrap();
    assert_eq!(v.major, 1);
}
```

### Integration Tests (Component Layer)
- Components working together
- Mock external dependencies (git, filesystem)
- Real algorithms running end-to-end

```rust
#[test]
fn test_analyzer_with_real_commits() {
    let analyzer = VersionAnalyzer::new(config);
    let messages = vec!["feat: new", "fix: bug"];
    assert_eq!(analyzer.analyze(&messages), VersionBump::Minor);
}
```

### Mock Objects
- `MockRepository`: In-memory git state
- `MockHookExecutor`: Tracks hook executions
- Deterministic test data

## Performance Considerations

### Algorithm Complexity
- **Tag listing**: O(n) where n = number of tags
- **Commit parsing**: O(m) where m = commits since last tag
- **Version bumping**: O(1)
- **Overall**: Linear in commit count between tags

### Memory Usage
- Commits stored in memory: ~1KB per commit
- Tags in memory: ~100 bytes per tag
- Suitable for repositories with thousands of commits

### Optimization Opportunities
- Cache tag listings (invalidate on update)
- Parallel commit parsing (per-commit operations are independent)
- Lazy loading of commit bodies

## Security Considerations

### Hook Execution
- Hooks run with full user permissions
- User controls hook paths (via gitpublish.toml)
- No input validation on hook script - it's user code
- Recommendation: Version control hook scripts with repository

### Error Information Leakage
- Error messages include file paths and git information
- Suitable for local development
- Consider sanitizing in server-side deployments

### Version Number Validation
- Accepts versions with large numbers (999.999.999)
- No integer overflow: u32 max is 4,294,967,295
- Reasonable for practical use cases

## Dependencies

### Core Dependencies
- **serde**: Configuration deserialization
- **toml**: TOML format support
- **git2**: Git operations
- **anyhow/thiserror**: Error handling
- **regex**: Conventional commit parsing
- **clap**: CLI argument parsing

### Dev Dependencies
- **tempfile**: Test fixtures
- **mockall** (optional): Mock generation

### Dependency Tree (Minimal)
```
git-publish
├── git2 (0.29)
├── serde (1.0)
├── toml (0.8)
├── thiserror (1.0)
├── regex (1.10)
├── anyhow (1.0)
└── clap (4.4)
```

## Future Architecture Improvements

### 1. Plugin System
- Dynamic loading of repository implementations
- User-provided version bump strategies
- Custom hook executors

### 2. Distributed Version State
- Track versions across multiple repositories
- Coordinated versioning for monorepos

### 3. Version History Database
- SQLite storage for version metadata
- Query version trends
- Audit trail

### 4. Interactive Mode
- Step-by-step version bumping
- Preview changes before committing
- Dry-run mode

### 5. Configuration Validation
- JSON Schema for gitpublish.toml
- Validation errors with suggestions
- Type checking on config values

---

**Document Version**: 1.0  
**Last Updated**: 2025-01-23  
**Architecture Quality**: Production-Ready (100+ tests, zero warnings)
