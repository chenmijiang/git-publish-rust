//! Git operations abstraction layer
//!
//! This module provides a trait-based abstraction over Git operations,
//! allowing for multiple implementations including real Git repositories
//! and mock implementations for testing.
//!
//! # Overview
//!
//! The primary abstraction is the [Repository] trait, which defines common
//! Git operations that git-publish needs. The concrete implementations include:
//!
//! - [repository::Git2Repository]: A real implementation using the `git2` crate
//! - [mock::MockRepository]: A mock implementation for testing
//!
//! # Usage
//!
//! Most code should depend on the [Repository] trait rather than concrete
//! implementations to enable easy testing and flexibility.
//!
//! ```rust
//! # use git_publish::git::Repository;
//! # use git2::Oid;
//! # fn example<R: Repository>(repo: &R) -> Result<(), Box<dyn std::error::Error>> {
//! let head_oid = repo.get_branch_head_oid("main")?;
//! let commits = repo.get_commits_between(Oid::zero(), head_oid)?;
//! # Ok(())
//! # }
//! ```

pub mod mock;
pub mod repository;

pub use mock::MockRepository;
pub use repository::Git2Repository;

use crate::error::Result;
use git2::Oid;

/// Commit information for analysis
#[derive(Debug, Clone, PartialEq)]
pub struct CommitInfo {
    /// The commit hash (shortened)
    pub hash: String,
    /// The commit message
    pub message: String,
    /// The commit author
    pub author: String,
}

/// Common git operation trait for abstraction
///
/// This trait abstracts Git operations to allow for multiple implementations
/// including real Git repositories and mock implementations for testing.
///
/// ## Thread Safety
///
/// All implementors must be `Send + Sync` to allow safe sharing across threads.
///
/// ## Error Handling
///
/// All methods return [crate::error::Result<T>] which handles Git-specific and
/// application errors uniformly. Implementation should map underlying errors
/// (like `git2::Error`) to the appropriate [crate::error::GitPublishError] variants.
///
/// ## Implementations
///
/// - [Git2Repository](repository::Git2Repository): Real Git implementation using the `git2` crate
/// - [MockRepository](mock::MockRepository): Test implementation for mocking Git operations
pub trait Repository: Send + Sync {
    /// Get the OID of a branch's HEAD
    ///
    /// Returns the object ID (OID) of the commit at the tip of the specified branch.
    ///
    /// # Arguments
    /// * `branch_name` - The name of the branch (e.g., "main", "develop")
    ///
    /// # Returns
    /// * `Ok(Oid)` - Object ID of the branch's HEAD commit
    /// * `Err` - If the branch doesn't exist or if there's a Git error
    ///
    /// # Example
    /// ```rust
    /// # use git_publish::git::Repository;
    /// # use git2::Oid;
    /// # fn example<R: Repository>(repo: &R) -> Result<(), Box<dyn std::error::Error>> {
    /// let oid = repo.get_branch_head_oid("main")?;
    /// println!("Main branch HEAD: {}", oid);
    /// # Ok(())
    /// # }
    /// ```
    fn get_branch_head_oid(&self, branch_name: &str) -> Result<Oid>;

    /// Get commits between two OIDs
    ///
    /// Returns commits in the range from `from_oid` (exclusive) to `to_oid` (inclusive),
    /// in chronological order (oldest first).
    ///
    /// # Arguments
    /// * `from_oid` - Starting commit (exclusive - not included in results)
    /// * `to_oid` - Ending commit (inclusive - included in results)
    ///
    /// # Returns
    /// * `Ok(Vec<CommitInfo>)` - List of commits in chronological order (oldest first)
    /// * `Err` - If either OID doesn't exist or if there's a Git error
    ///
    /// # Example
    /// ```rust
    /// # use git_publish::git::Repository;
    /// # use git2::Oid;
    /// # fn example<R: Repository>(repo: &R) -> Result<(), Box<dyn std::error::Error>> {
    /// # let main_oid = repo.get_branch_head_oid("main")?;
    /// # let develop_oid = repo.get_branch_head_oid("develop")?;
    /// let commits = repo.get_commits_between(main_oid, develop_oid)?;
    /// for commit in commits {
    ///     println!("{}: {}", commit.hash, commit.message);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn get_commits_between(&self, from_oid: Oid, to_oid: Oid) -> Result<Vec<CommitInfo>>;

    /// Find a tag by name and get its OID
    ///
    /// Returns the object ID of a tag if it exists, or `None` if the tag doesn't exist.
    /// Handles both lightweight and annotated tags.
    ///
    /// # Arguments
    /// * `tag_name` - Name of the tag (e.g., "v1.0.0", "release-1")
    ///
    /// # Returns
    /// * `Ok(Some(Oid))` - Object ID of the tag if it exists
    /// * `Ok(None)` - If the tag doesn't exist
    /// * `Err` - If there's a Git error
    ///
    /// # Example
    /// ```rust
    /// # use git_publish::git::Repository;
    /// # fn example<R: Repository>(repo: &R) -> Result<(), Box<dyn std::error::Error>> {
    /// match repo.find_tag_oid("v1.0.0")? {
    ///     Some(oid) => println!("Tag v1.0.0 exists at: {}", oid),
    ///     None => println!("Tag v1.0.0 does not exist"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn find_tag_oid(&self, tag_name: &str) -> Result<Option<Oid>>;

    /// Get all tags in the repository
    ///
    /// Returns a list of all tag names in the repository, sorted alphabetically.
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - Sorted list of tag names
    /// * `Err` - If there's a Git error
    ///
    /// # Example
    /// ```rust
    /// # use git_publish::git::Repository;
    /// # fn example<R: Repository>(repo: &R) -> Result<(), Box<dyn std::error::Error>> {
    /// let tags = repo.list_tags()?;
    /// for tag in tags {
    ///     println!("Tag: {}", tag);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn list_tags(&self) -> Result<Vec<String>>;

    /// Create a lightweight tag at given OID
    ///
    /// Creates a lightweight tag (not annotated) pointing to the specified commit.
    ///
    /// # Arguments
    /// * `name` - Name for the new tag
    /// * `oid` - Object ID of the commit to tag
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err` - If the tag already exists, OID doesn't exist, or Git error occurs
    ///
    /// # Example
    /// ```rust
    /// # use git_publish::git::Repository;
    /// # use git2::Oid;
    /// # fn example<R: Repository>(repo: &R) -> Result<(), Box<dyn std::error::Error>> {
    /// # let commit_oid = repo.get_branch_head_oid("main")?;
    /// repo.create_tag("v1.0.0", commit_oid)?;
    /// println!("Created tag v1.0.0");
    /// # Ok(())
    /// # }
    /// ```
    fn create_tag(&self, name: &str, oid: Oid) -> Result<()>;

    /// Push tags to remote
    ///
    /// Pushes the specified tags to a remote repository.
    ///
    /// # Arguments
    /// * `remote` - Name of the remote (e.g., "origin", "upstream")
    /// * `tag_names` - Slice of tag names to push
    ///
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err` - If the remote doesn't exist, tags don't exist, or Git error occurs
    ///
    /// # Example
    /// ```rust
    /// # use git_publish::git::Repository;
    /// # fn example<R: Repository>(repo: &R) -> Result<(), Box<dyn std::error::Error>> {
    /// repo.push_tags("origin", &["v1.0.0", "v1.0.1"])?;
    /// println!("Tags pushed to origin");
    /// # Ok(())
    /// # }
    /// ```
    fn push_tags(&self, remote: &str, tag_names: &[&str]) -> Result<()>;

    /// Fetch from remote
    ///
    /// Fetches the latest changes from a remote repository for a specific branch.
    ///
    /// # Arguments
    /// * `remote` - Name of the remote (e.g., "origin", "upstream")
    /// * `branch` - Name of the branch to fetch (e.g., "main", "develop")
    ///
    /// # Returns
    /// * `Ok(Oid)` - The object ID of the fetched remote branch head
    /// * `Err` - If the remote or branch doesn't exist, or Git error occurs
    ///
    /// # Example
    /// ```rust
    /// # use git_publish::git::Repository;
    /// # fn example<R: Repository>(repo: &R) -> Result<(), Box<dyn std::error::Error>> {
    /// let remote_head = repo.fetch_from_remote("origin", "main")?;
    /// println!("Fetched origin/main with head: {}", remote_head);
    /// # Ok(())
    /// # }
    /// ```
    fn fetch_from_remote(&self, remote: &str, branch: &str) -> Result<Oid>;
}
