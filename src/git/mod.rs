//! Git operations abstraction layer

pub mod mock;
pub mod repository;

pub use mock::MockRepository;
pub use repository::Git2Repository;

use crate::error::Result;
use git2::Oid;

/// Commit information for analysis
#[derive(Debug, Clone, PartialEq)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
}

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
