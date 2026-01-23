# git-publish Refactoring Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform git-publish from a monolithic CLI tool into a well-architected, extensible codebase with clear separation of concerns, supporting pre-releases and git hooks while maintaining 100% backward compatibility.

**Architecture:** Layered architecture with domain logic (business rules) isolated from infrastructure (git operations), CLI orchestration separated from application logic, and trait-based abstractions enabling testability.

**Tech Stack:** Rust 2021, clap CLI, serde TOML, git2 library, thiserror for error handling, tempfile for testing.

---

## Phase 1: Infrastructure Setup (2-3 hours)

### Task 1: Create unified error type with thiserror

**Files:**
- Create: `src/error.rs`
- Modify: `src/lib.rs` (add error module export)
- Test: `tests/error_tests.rs`

**Step 1: Create error.rs with thiserror**

Create `src/error.rs`:

```rust
use std::fmt;
use thiserror::Error;

/// Unified error type for git-publish operations
#[derive(Error, Debug)]
pub enum GitPublishError {
    #[error("Git operation failed: {0}")]
    Git(#[from] git2::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Version parsing error: {0}")]
    Version(String),

    #[error("Tag error: {0}")]
    Tag(String),

    #[error("Hook execution failed: {0}")]
    Hook(String),

    #[error("Remote operation failed: {0}")]
    Remote(String),

    #[error("Branch error: {0}")]
    Branch(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

/// Convenience type alias for Results in git-publish
pub type Result<T> = std::result::Result<T, GitPublishError>;

impl GitPublishError {
    /// Create a configuration error with context
    pub fn config(msg: impl Into<String>) -> Self {
        GitPublishError::Config(msg.into())
    }

    /// Create a version error with context
    pub fn version(msg: impl Into<String>) -> Self {
        GitPublishError::Version(msg.into())
    }

    /// Create a tag error with context
    pub fn tag(msg: impl Into<String>) -> Self {
        GitPublishError::Tag(msg.into())
    }

    /// Create a hook error with context
    pub fn hook(msg: impl Into<String>) -> Self {
        GitPublishError::Hook(msg.into())
    }

    /// Create a branch error with context
    pub fn branch(msg: impl Into<String>) -> Self {
        GitPublishError::Branch(msg.into())
    }

    /// Create a remote error with context
    pub fn remote(msg: impl Into<String>) -> Self {
        GitPublishError::Remote(msg.into())
    }

    /// Create an invalid argument error
    pub fn invalid_arg(msg: impl Into<String>) -> Self {
        GitPublishError::InvalidArgument(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = GitPublishError::config("test config issue");
        assert_eq!(err.to_string(), "Configuration error: test config issue");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: GitPublishError = io_err.into();
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn test_error_constructors() {
        assert!(GitPublishError::version("test").to_string().contains("Version"));
        assert!(GitPublishError::tag("test").to_string().contains("Tag"));
        assert!(GitPublishError::branch("test").to_string().contains("Branch"));
    }
}
```

**Step 2: Update Cargo.toml to add thiserror dependency**

Modify `Cargo.toml`, in the `[dependencies]` section add:

```toml
thiserror = "1.0"
```

(Keep the rest of dependencies as-is)

**Step 3: Update lib.rs to export error module**

Modify `src/lib.rs`, replace the entire file with:

```rust
pub mod error;
pub mod config;
pub mod conventional;
pub mod version;
pub mod boundary;
pub mod ui;
pub mod git_ops;

pub use error::{GitPublishError, Result};
```

**Step 4: Run tests to verify error module**

Run:
```bash
cargo test error_tests --lib
```

Expected: All tests pass (3 tests)

**Step 5: Commit**

```bash
git add Cargo.toml src/error.rs src/lib.rs
git commit -m "feat: add unified error handling with thiserror"
```

---

### Task 2: Create domain module structure

**Files:**
- Create: `src/domain/mod.rs`
- Create: `src/domain/version.rs` (extracted from current version.rs)
- Create: `src/domain/commit.rs` (extracted from current conventional.rs)
- Create: `src/domain/tag.rs` (new, for tag logic)
- Create: `src/domain/branch.rs` (new, for branch context)

**Step 1: Create domain module directory structure**

```bash
mkdir -p src/domain
touch src/domain/mod.rs
```

**Step 2: Create domain/mod.rs**

Create `src/domain/mod.rs`:

```rust
//! Domain logic - pure business rules independent of git operations

pub mod version;
pub mod commit;
pub mod tag;
pub mod branch;

pub use version::{Version, VersionBump};
pub use commit::ParsedCommit;
pub use tag::{Tag, TagPattern};
pub use branch::BranchContext;
```

**Step 3: Create domain/version.rs (extract from src/version.rs)**

Create `src/domain/version.rs`:

```rust
use crate::error::{GitPublishError, Result};
use std::fmt;

/// Semantic version representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version { major, minor, patch }
    }

    /// Parse version from a tag string (e.g., "v1.2.3" -> Version(1,2,3))
    pub fn parse(tag: &str) -> Result<Self> {
        // Remove 'v' or 'V' prefix
        let clean_tag = tag.trim_start_matches('v').trim_start_matches('V');

        // Split by '.' and parse
        let parts: Vec<&str> = clean_tag.split('.').collect();
        if parts.len() != 3 {
            return Err(GitPublishError::version(format!(
                "Invalid version format: '{}' - expected X.Y.Z",
                tag
            )));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| GitPublishError::version(format!("Invalid major version: {}", parts[0])))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| GitPublishError::version(format!("Invalid minor version: {}", parts[1])))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| GitPublishError::version(format!("Invalid patch version: {}", parts[2])))?;

        Ok(Version { major, minor, patch })
    }

    /// Bump version according to bump type
    pub fn bump(&self, bump_type: &VersionBump) -> Self {
        match bump_type {
            VersionBump::Major => Version {
                major: self.major + 1,
                minor: 0,
                patch: 0,
            },
            VersionBump::Minor => Version {
                major: self.major,
                minor: self.minor + 1,
                patch: 0,
            },
            VersionBump::Patch => Version {
                major: self.major,
                minor: self.minor,
                patch: self.patch + 1,
            },
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Version bump type decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionBump {
    Major,
    Minor,
    Patch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::parse("v1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_parse_without_v() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v, Version::new(1, 2, 3));
    }

    #[test]
    fn test_version_parse_uppercase_v() {
        let v = Version::parse("V1.2.3").unwrap();
        assert_eq!(v, Version::new(1, 2, 3));
    }

    #[test]
    fn test_version_parse_invalid() {
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("v1.2.3.4").is_err());
    }

    #[test]
    fn test_version_bump_major() {
        let v = Version::new(1, 2, 3);
        let bumped = v.bump(&VersionBump::Major);
        assert_eq!(bumped, Version::new(2, 0, 0));
    }

    #[test]
    fn test_version_bump_minor() {
        let v = Version::new(1, 2, 3);
        let bumped = v.bump(&VersionBump::Minor);
        assert_eq!(bumped, Version::new(1, 3, 0));
    }

    #[test]
    fn test_version_bump_patch() {
        let v = Version::new(1, 2, 3);
        let bumped = v.bump(&VersionBump::Patch);
        assert_eq!(bumped, Version::new(1, 2, 4));
    }

    #[test]
    fn test_version_display() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }
}
```

**Step 4: Create domain/commit.rs (extract from src/conventional.rs)**

Create `src/domain/commit.rs`:

```rust
use crate::error::Result;
use regex::Regex;

/// Parsed representation of a conventional commit message
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommit {
    pub r#type: String,
    pub scope: Option<String>,
    pub description: String,
    pub is_breaking_change: bool,
}

impl ParsedCommit {
    /// Parse a commit message according to conventional commits spec
    /// Supports formats:
    /// - type(scope)!: description
    /// - type(scope): description
    /// - type!: description
    /// - type: description
    /// - non-conventional text
    pub fn parse(message: &str) -> Self {
        let mut is_breaking = false;

        // Try format: type(scope)!: description
        if let Some(captures) = Regex::new(r"^([a-z]+)\(([^)]+)\)(!?):\s*(.*)").ok()
            .and_then(|re| re.captures(message))
        {
            let r#type = captures.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let scope = captures.get(2).map(|m| m.as_str().to_string());
            let has_exclamation = captures.get(3).map(|m| m.as_str()) == Some("!");
            let description = captures.get(4).map(|m| m.as_str().to_string()).unwrap_or_default();

            is_breaking = has_exclamation || message.contains("BREAKING CHANGE:");

            return ParsedCommit {
                r#type,
                scope,
                description,
                is_breaking_change: is_breaking,
            };
        }

        // Try format: type!: description
        if let Some(captures) = Regex::new(r"^([a-z]+)!:\s*(.*)").ok()
            .and_then(|re| re.captures(message))
        {
            let r#type = captures.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let description = captures.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();

            return ParsedCommit {
                r#type,
                scope: None,
                description,
                is_breaking_change: true,
            };
        }

        // Try format: type: description
        if let Some(captures) = Regex::new(r"^([a-z]+):\s*(.*)").ok()
            .and_then(|re| re.captures(message))
        {
            let r#type = captures.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let description = captures.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();

            is_breaking = message.contains("BREAKING CHANGE:");

            return ParsedCommit {
                r#type,
                scope: None,
                description,
                is_breaking_change: is_breaking,
            };
        }

        // Default: non-conventional commit
        ParsedCommit {
            r#type: "chore".to_string(),
            scope: None,
            description: message.to_string(),
            is_breaking_change: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_scope() {
        let commit = ParsedCommit::parse("feat(auth): add login");
        assert_eq!(commit.r#type, "feat");
        assert_eq!(commit.scope, Some("auth".to_string()));
        assert_eq!(commit.description, "add login");
        assert!(!commit.is_breaking_change);
    }

    #[test]
    fn test_parse_with_breaking_marker() {
        let commit = ParsedCommit::parse("feat(auth)!: redesign login");
        assert_eq!(commit.r#type, "feat");
        assert!(commit.is_breaking_change);
    }

    #[test]
    fn test_parse_breaking_without_scope() {
        let commit = ParsedCommit::parse("feat!: redesign");
        assert_eq!(commit.r#type, "feat");
        assert_eq!(commit.scope, None);
        assert!(commit.is_breaking_change);
    }

    #[test]
    fn test_parse_non_conventional() {
        let commit = ParsedCommit::parse("Random commit message");
        assert_eq!(commit.r#type, "chore");
        assert!(!commit.is_breaking_change);
    }

    #[test]
    fn test_parse_breaking_change_footer() {
        let commit = ParsedCommit::parse("fix: something\n\nBREAKING CHANGE: desc");
        assert!(commit.is_breaking_change);
    }
}
```

**Step 5: Create domain/tag.rs (new)**

Create `src/domain/tag.rs`:

```rust
use crate::error::{GitPublishError, Result};

/// Represents a git tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
}

impl Tag {
    /// Create a new tag from a string
    pub fn new(name: impl Into<String>) -> Self {
        Tag {
            name: name.into(),
        }
    }

    /// Extract version number from tag (e.g., "v1.2.3" -> "1.2.3")
    pub fn version_part(&self) -> Result<String> {
        let trimmed = self.name.trim_start_matches('v').trim_start_matches('V');
        Ok(trimmed.to_string())
    }
}

/// Tag naming pattern (e.g., "v{version}", "release-{version}")
#[derive(Debug, Clone)]
pub struct TagPattern {
    pub pattern: String,
}

impl TagPattern {
    /// Create a new tag pattern
    pub fn new(pattern: impl Into<String>) -> Self {
        TagPattern {
            pattern: pattern.into(),
        }
    }

    /// Format a version according to pattern
    /// Example: pattern="v{version}", version="1.2.3" -> "v1.2.3"
    pub fn format(&self, version: &str) -> String {
        self.pattern.replace("{version}", version)
    }

    /// Validate if a tag matches this pattern
    pub fn matches(&self, tag: &str) -> Result<bool> {
        // Extract the placeholder pattern part
        if !self.pattern.contains("{version}") {
            return Err(GitPublishError::tag("Pattern must contain {version} placeholder"));
        }

        // Create regex pattern: escape everything, replace {version} with regex
        let escaped = regex::escape(&self.pattern);
        let regex_pattern = escaped.replace(r"\{version\}", r"(\d+\.\d+\.\d+)");

        if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            Ok(re.is_match(tag))
        } else {
            Err(GitPublishError::tag("Invalid pattern"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_new() {
        let tag = Tag::new("v1.2.3");
        assert_eq!(tag.name, "v1.2.3");
    }

    #[test]
    fn test_tag_version_part() {
        let tag = Tag::new("v1.2.3");
        assert_eq!(tag.version_part().unwrap(), "1.2.3");
    }

    #[test]
    fn test_pattern_format() {
        let pattern = TagPattern::new("v{version}");
        assert_eq!(pattern.format("1.2.3"), "v1.2.3");
    }

    #[test]
    fn test_pattern_format_with_suffix() {
        let pattern = TagPattern::new("release-{version}");
        assert_eq!(pattern.format("1.2.3"), "release-1.2.3");
    }

    #[test]
    fn test_pattern_matches() {
        let pattern = TagPattern::new("v{version}");
        assert!(pattern.matches("v1.2.3").unwrap());
        assert!(!pattern.matches("release-1.2.3").unwrap());
    }
}
```

**Step 6: Create domain/branch.rs (new)**

Create `src/domain/branch.rs`:

```rust
use crate::error::Result;

/// Represents a git branch with context
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchContext {
    pub name: String,
    pub is_main: bool,
}

impl BranchContext {
    /// Create a new branch context
    pub fn new(name: impl Into<String>) -> Self {
        let name_str = name.into();
        let is_main = matches!(name_str.as_str(), "main" | "master");

        BranchContext {
            name: name_str,
            is_main,
        }
    }

    /// Check if this is a release branch (main/master)
    pub fn is_release_branch(&self) -> bool {
        self.is_main
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_branch() {
        let branch = BranchContext::new("main");
        assert!(branch.is_main);
        assert!(branch.is_release_branch());
    }

    #[test]
    fn test_master_branch() {
        let branch = BranchContext::new("master");
        assert!(branch.is_main);
    }

    #[test]
    fn test_develop_branch() {
        let branch = BranchContext::new("develop");
        assert!(!branch.is_main);
        assert!(!branch.is_release_branch());
    }
}
```

**Step 7: Run all domain tests**

```bash
cargo test domain --lib
```

Expected: All tests pass (21+ tests)

**Step 8: Commit**

```bash
git add src/domain/
git commit -m "feat: create domain module with pure business logic"
```

---

### Task 3: Create git operations abstraction layer

**Files:**
- Create: `src/git/mod.rs`
- Create: `src/git/repository.rs` (trait abstraction)
- Create: `src/git/mock.rs` (mock implementation for tests)

**Step 1: Create git module directory**

```bash
mkdir -p src/git
touch src/git/mod.rs
```

**Step 2: Create git/mod.rs**

Create `src/git/mod.rs`:

```rust
//! Git operations abstraction layer

pub mod repository;
pub mod mock;

pub use repository::{Repository, Git2Repository};
pub use mock::MockRepository;

use crate::error::Result;
use crate::domain::Tag;
use git2::Oid;

/// Common git operation trait for abstraction
pub trait Repository: Send + Sync {
    /// Get the OID of a branch's HEAD
    fn get_branch_head_oid(&self, branch_name: &str) -> Result<Oid>;

    /// Get commits between two OIDs
    fn get_commits_between(&self, from_oid: Oid, to_oid: Oid) -> Result<Vec<CommitInfo>>;

    /// Find a tag by name and get its OID
    fn find_tag_oid(&self, tag_name: &str) -> Result<Option<Oid>>;

    /// Get all tags in the repository
    fn list_tags(&self) -> Result<Vec<String>>;

    /// Create a lightweight tag at given OID
    fn create_tag(&self, name: &str, oid: Oid) -> Result<()>;

    /// Push tags to remote
    fn push_tags(&self, remote: &str, tag_names: &[&str]) -> Result<()>;

    /// Fetch from remote
    fn fetch_from_remote(&self, remote: &str, branch: &str) -> Result<()>;
}

/// Commit information for analysis
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
}
```

**Step 3: Create git/repository.rs**

Create `src/git/repository.rs`:

```rust
use crate::error::{GitPublishError, Result};
use crate::git::CommitInfo;
use git2::{Oid, Repository as Git2Repo, Direction};
use std::path::Path;

/// Wrapper around git2::Repository with our trait interface
pub struct Git2Repository {
    repo: Git2Repo,
}

impl Git2Repository {
    /// Open or discover a git repository
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Git2Repo::discover(path)
            .map_err(|e| GitPublishError::Git(e))?;

        Ok(Git2Repository { repo })
    }

    /// Create from existing git2::Repository
    pub fn from_git2(repo: Git2Repo) -> Self {
        Git2Repository { repo }
    }
}

impl super::Repository for Git2Repository {
    fn get_branch_head_oid(&self, branch_name: &str) -> Result<Oid> {
        let branch = self.repo
            .find_branch(branch_name, git2::BranchType::Local)
            .map_err(|e| GitPublishError::Branch(format!("Cannot find branch '{}': {}", branch_name, e)))?;

        let reference = branch.get();
        let oid = reference
            .target()
            .ok_or_else(|| GitPublishError::Branch(format!("Branch '{}' has no target", branch_name)))?;

        Ok(oid)
    }

    fn get_commits_between(&self, from_oid: Oid, to_oid: Oid) -> Result<Vec<CommitInfo>> {
        let mut revwalk = self.repo
            .revwalk(to_oid)
            .map_err(|e| GitPublishError::Git(e))?;

        let mut commits = Vec::new();

        for oid_result in revwalk {
            let oid = oid_result.map_err(|e| GitPublishError::Git(e))?;

            if oid == from_oid {
                break;
            }

            let commit = self.repo
                .find_commit(oid)
                .map_err(|e| GitPublishError::Git(e))?;

            let message = commit
                .message()
                .unwrap_or("(empty message)")
                .to_string();

            let author = commit
                .author()
                .name()
                .unwrap_or("unknown")
                .to_string();

            commits.push(CommitInfo {
                hash: oid.to_string(),
                message,
                author,
            });
        }

        commits.reverse();
        Ok(commits)
    }

    fn find_tag_oid(&self, tag_name: &str) -> Result<Option<Oid>> {
        let reference_name = format!("refs/tags/{}", tag_name);

        match self.repo.find_reference(&reference_name) {
            Ok(reference) => {
                let oid = reference
                    .peel(git2::ObjectType::Any)
                    .map_err(|e| GitPublishError::Tag(format!("Cannot peel tag: {}", e)))?
                    .id();

                Ok(Some(oid))
            }
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(GitPublishError::Tag(format!("Cannot find tag '{}': {}", tag_name, e))),
        }
    }

    fn list_tags(&self) -> Result<Vec<String>> {
        let tags = self.repo
            .tag_names(None)
            .map_err(|e| GitPublishError::Git(e))?;

        Ok(tags.iter()
            .flatten()
            .map(|s| s.to_string())
            .collect())
    }

    fn create_tag(&self, name: &str, oid: Oid) -> Result<()> {
        let object = self.repo
            .find_object(oid, None)
            .map_err(|e| GitPublishError::Tag(format!("Cannot find object: {}", e)))?;

        self.repo
            .tag_lightweight(name, &object, false)
            .map_err(|e| GitPublishError::Tag(format!("Cannot create tag: {}", e)))?;

        Ok(())
    }

    fn push_tags(&self, remote: &str, tag_names: &[&str]) -> Result<()> {
        let mut remote = self.repo
            .find_remote(remote)
            .map_err(|e| GitPublishError::Remote(format!("Cannot find remote: {}", e)))?;

        let refspecs: Vec<String> = tag_names
            .iter()
            .map(|tag| format!("refs/tags/{}:refs/tags/{}", tag, tag))
            .collect();

        let refspec_strs: Vec<&str> = refspecs.iter().map(|s| s.as_str()).collect();

        remote
            .push(&refspec_strs, None)
            .map_err(|e| GitPublishError::Remote(format!("Push failed: {}", e)))?;

        Ok(())
    }

    fn fetch_from_remote(&self, remote: &str, branch: &str) -> Result<()> {
        let mut remote = self.repo
            .find_remote(remote)
            .map_err(|e| GitPublishError::Remote(format!("Cannot find remote: {}", e)))?;

        remote
            .fetch(&[branch], None, None)
            .map_err(|e| GitPublishError::Remote(format!("Fetch failed: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git2_repository_open() {
        // This will test in actual integration context
        // Unit test would need a real repo or mock
        let result = Git2Repository::open(".");
        // Should either succeed or fail gracefully
        let _ = result;
    }
}
```

**Step 4: Create git/mock.rs for testing**

Create `src/git/mock.rs`:

```rust
use crate::error::Result;
use crate::git::{Repository, CommitInfo};
use git2::Oid;
use std::collections::HashMap;

/// Mock repository for testing without actual git operations
pub struct MockRepository {
    commits: HashMap<Oid, CommitInfo>,
    tags: HashMap<String, Oid>,
    branch_heads: HashMap<String, Oid>,
}

impl MockRepository {
    /// Create a new empty mock repository
    pub fn new() -> Self {
        MockRepository {
            commits: HashMap::new(),
            tags: HashMap::new(),
            branch_heads: HashMap::new(),
        }
    }

    /// Add a commit to the mock repository
    pub fn add_commit(&mut self, oid: Oid, info: CommitInfo) {
        self.commits.insert(oid, info);
    }

    /// Add a tag pointing to an OID
    pub fn add_tag(&mut self, name: impl Into<String>, oid: Oid) {
        self.tags.insert(name.into(), oid);
    }

    /// Set a branch head
    pub fn set_branch_head(&mut self, branch: impl Into<String>, oid: Oid) {
        self.branch_heads.insert(branch.into(), oid);
    }
}

impl Default for MockRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository for MockRepository {
    fn get_branch_head_oid(&self, branch_name: &str) -> Result<Oid> {
        self.branch_heads
            .get(branch_name)
            .copied()
            .ok_or_else(|| crate::error::GitPublishError::branch(format!("Branch not found: {}", branch_name)))
    }

    fn get_commits_between(&self, from_oid: Oid, to_oid: Oid) -> Result<Vec<CommitInfo>> {
        // Simplified: return commits in order from hashmap
        let mut commits: Vec<_> = self.commits
            .values()
            .cloned()
            .collect();
        commits.sort_by_key(|c| c.hash.clone());
        Ok(commits)
    }

    fn find_tag_oid(&self, tag_name: &str) -> Result<Option<Oid>> {
        Ok(self.tags.get(tag_name).copied())
    }

    fn list_tags(&self) -> Result<Vec<String>> {
        Ok(self.tags.keys().cloned().collect())
    }

    fn create_tag(&self, _name: &str, _oid: Oid) -> Result<()> {
        Ok(())
    }

    fn push_tags(&self, _remote: &str, _tag_names: &[&str]) -> Result<()> {
        Ok(())
    }

    fn fetch_from_remote(&self, _remote: &str, _branch: &str) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_repository_basic() {
        let mut repo = MockRepository::new();
        let oid = Oid::from_bytes(&[1; 20]).unwrap();

        repo.add_commit(oid, CommitInfo {
            hash: "abc123".to_string(),
            message: "test commit".to_string(),
            author: "Test Author".to_string(),
        });

        repo.set_branch_head("main", oid);

        assert_eq!(repo.get_branch_head_oid("main").unwrap(), oid);
    }

    #[test]
    fn test_mock_repository_tags() {
        let mut repo = MockRepository::new();
        let oid = Oid::from_bytes(&[2; 20]).unwrap();

        repo.add_tag("v1.0.0", oid);

        assert_eq!(repo.find_tag_oid("v1.0.0").unwrap(), Some(oid));
        assert_eq!(repo.find_tag_oid("v2.0.0").unwrap(), None);
    }
}
```

**Step 5: Update lib.rs to export git module**

Modify `src/lib.rs`:

```rust
pub mod error;
pub mod domain;
pub mod git;
pub mod config;
pub mod conventional;
pub mod version;
pub mod boundary;
pub mod ui;
pub mod git_ops;

pub use error::{GitPublishError, Result};
```

**Step 6: Run all tests**

```bash
cargo test git --lib
cargo test mock --lib
```

Expected: All tests pass (5+ tests)

**Step 7: Commit**

```bash
git add src/git/ src/lib.rs
git commit -m "feat: create git operations abstraction with Repository trait"
```

---

## End of Phase 1

**Checkpoint:** All infrastructure is in place
- ✅ Unified error handling with thiserror
- ✅ Domain module with pure business logic
- ✅ Git abstraction with Repository trait
- ✅ Mock repository for testing
- ✅ All tests passing

**Files changed:** 14
**Tests added:** 30+
**Lines of code:** ~1000 (all focused, well-tested)

---

## Phase 2: Migrate Core Business Logic (2-3 hours)

### Task 4: Create analyzer module and migrate version bump logic

**Files:**
- Create: `src/analyzer/mod.rs`
- Create: `src/analyzer/version_analyzer.rs`
- Modify: Tests for new analyzer

**Step 1: Create analyzer module**

```bash
mkdir -p src/analyzer
touch src/analyzer/mod.rs
```

Create `src/analyzer/mod.rs`:

```rust
//! Analysis engine for determining version bumps from commits

pub mod version_analyzer;

pub use version_analyzer::VersionAnalyzer;
```

**Step 2: Create analyzer/version_analyzer.rs**

Create `src/analyzer/version_analyzer.rs`:

```rust
use crate::config::ConventionalCommitsConfig;
use crate::domain::{ParsedCommit, VersionBump};

/// Analyzes commits to determine version bump type
pub struct VersionAnalyzer {
    config: ConventionalCommitsConfig,
}

impl VersionAnalyzer {
    /// Create a new version analyzer
    pub fn new(config: ConventionalCommitsConfig) -> Self {
        VersionAnalyzer { config }
    }

    /// Analyze commit messages and determine version bump
    pub fn analyze(&self, messages: &[String]) -> VersionBump {
        let mut has_breaking = false;
        let mut has_features = false;
        let mut has_fixes = false;

        for message in messages {
            let parsed = ParsedCommit::parse(message);

            // Check for breaking changes (highest priority)
            if parsed.is_breaking_change {
                has_breaking = true;
            }

            // Check for features
            if self.is_feature(&parsed.r#type) {
                has_features = true;
            }

            // Check for fixes
            if self.is_fix(&parsed.r#type) {
                has_fixes = true;
            }
        }

        // Decision tree (priority order)
        if has_breaking {
            VersionBump::Major
        } else if has_features {
            VersionBump::Minor
        } else if has_fixes {
            VersionBump::Patch
        } else {
            VersionBump::Patch // default
        }
    }

    fn is_feature(&self, commit_type: &str) -> bool {
        commit_type == "feat" || commit_type == "feature"
    }

    fn is_fix(&self, commit_type: &str) -> bool {
        matches!(commit_type, "fix" | "perf" | "refactor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_major() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec![
            "feat: new feature".to_string(),
            "fix(api)!: breaking change".to_string(),
        ];

        assert_eq!(analyzer.analyze(&messages), VersionBump::Major);
    }

    #[test]
    fn test_analyze_minor() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec![
            "feat: new feature".to_string(),
            "fix: bug fix".to_string(),
        ];

        assert_eq!(analyzer.analyze(&messages), VersionBump::Minor);
    }

    #[test]
    fn test_analyze_patch() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec![
            "fix: bug fix".to_string(),
            "refactor: code cleanup".to_string(),
        ];

        assert_eq!(analyzer.analyze(&messages), VersionBump::Patch);
    }

    #[test]
    fn test_analyze_empty() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec!["docs: update readme".to_string()];

        assert_eq!(analyzer.analyze(&messages), VersionBump::Patch);
    }
}
```

**Step 3: Update lib.rs**

Modify `src/lib.rs`:

```rust
pub mod error;
pub mod domain;
pub mod git;
pub mod analyzer;
pub mod config;
pub mod conventional;
pub mod version;
pub mod boundary;
pub mod ui;
pub mod git_ops;

pub use error::{GitPublishError, Result};
```

**Step 4: Run tests**

```bash
cargo test analyzer --lib
```

Expected: 4 tests pass

**Step 5: Commit**

```bash
git add src/analyzer/ src/lib.rs
git commit -m "feat: create analyzer module for version bump logic"
```

---

## Summary So Far

You've now completed the infrastructure and domain logic extraction. The next tasks would be:

### Remaining Tasks (outline for Phase 2-3):

5. **Migrate main.rs orchestration** → Split into CLI parsing + orchestration
6. **Extract UI formatting** → Separate presentation from interaction
7. **Add pre-release feature** → domain/prerelease.rs + logic
8. **Add hooks system** → hooks/ module with execution
9. **Comprehensive testing** → Unit + integration tests
10. **Documentation** → Architecture guides and examples
11. **Final validation** → Build, test, commit

---

## Execution Handoff

Plan complete and saved to `docs/plans/2026-01-23-git-publish-refactoring-implementation.md`.

**Two execution options:**

**Option 1: Subagent-Driven (this session)**
- I dispatch a fresh subagent per task
- I review and validate between tasks
- Fast iteration with immediate feedback
- Best for: Real-time collaboration and course correction

**Option 2: Parallel Session (separate)**
- Open new session with executing-plans skill
- Batch multiple tasks with checkpoints
- Best for: Deep focus and rapid execution

**Which approach would you prefer?**
