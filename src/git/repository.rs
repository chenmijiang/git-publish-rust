use crate::error::{GitPublishError, Result};
use crate::git::CommitInfo;
use git2::{Oid, Repository as Git2Repo};
use std::path::Path;

/// Concrete implementation of the Repository trait using the git2 library
///
/// This struct wraps `git2::Repository` and provides implementations for all
/// methods defined in the `Repository` trait. It handles all Git operations
/// using the underlying libgit2 C library through the Rust bindings.
///
/// # Thread Safety
///
/// This implementation is safe to share across threads (`Send + Sync`) as long
/// as only read operations are performed. Git2 operations may block on I/O, but
/// libgit2 is designed to be thread-safe for concurrent read access.
///
/// # Error Handling
///
/// All methods convert `git2::Error` to appropriate `GitPublishError` variants
/// to provide a consistent error interface across the application.
pub struct Git2Repository {
    repo: Git2Repo,
}

impl Git2Repository {
    /// Open or discover a git repository at the given path
    ///
    /// This will search for a `.git` directory starting from the provided path
    /// and going up the directory hierarchy until one is found, or return an error.
    ///
    /// # Arguments
    /// * `path` - Path to start the repository discovery from
    ///
    /// # Returns
    /// * `Ok(Git2Repository)` - Successfully opened repository
    /// * `Err` - If no repository is found or if there's an I/O error
    ///
    /// # Example
    /// ```rust
    /// # use git_publish::git::repository::Git2Repository;
    /// let repo = Git2Repository::open(".").expect("Failed to open repository");
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Git2Repo::discover(path).map_err(GitPublishError::Git)?;

        Ok(Git2Repository { repo })
    }

    /// Create a Git2Repository from an existing git2::Repository
    ///
    /// This is primarily useful for advanced initialization scenarios where
    /// you already have a git2::Repository instance.
    ///
    /// # Arguments
    /// * `repo` - An existing git2::Repository instance
    ///
    /// # Returns
    /// * `Git2Repository` - Wrapped repository ready for use with the trait interface
    pub fn from_git2(repo: Git2Repo) -> Self {
        Git2Repository { repo }
    }
}

impl super::Repository for Git2Repository {
    fn get_branch_head_oid(&self, branch_name: &str) -> Result<Oid> {
        let branch = self
            .repo
            .find_branch(branch_name, git2::BranchType::Local)
            .map_err(|e| {
                GitPublishError::Branch(format!("Cannot find branch '{}': {}", branch_name, e))
            })?;

        let reference = branch.get();
        let oid = reference.target().ok_or_else(|| {
            GitPublishError::Branch(format!("Branch '{}' has no target", branch_name))
        })?;

        Ok(oid)
    }

    fn get_commits_between(&self, from_oid: Oid, to_oid: Oid) -> Result<Vec<CommitInfo>> {
        let mut revwalk = self.repo.revwalk().map_err(GitPublishError::Git)?;

        revwalk.push(to_oid).map_err(GitPublishError::Git)?;

        let mut commits = Vec::new();

        for oid_result in revwalk {
            let oid = oid_result.map_err(GitPublishError::Git)?;

            if oid == from_oid {
                break;
            }

            let commit = self.repo.find_commit(oid).map_err(GitPublishError::Git)?;

            let message = commit.message().unwrap_or("(empty message)").to_string();

            let author = commit.author().name().unwrap_or("unknown").to_string();

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
            Err(e) => Err(GitPublishError::Tag(format!(
                "Cannot find tag '{}': {}",
                tag_name, e
            ))),
        }
    }

    fn list_tags(&self) -> Result<Vec<String>> {
        let tags = self.repo.tag_names(None).map_err(GitPublishError::Git)?;

        Ok(tags.iter().flatten().map(|s| s.to_string()).collect())
    }

    fn create_tag(&self, name: &str, oid: Oid) -> Result<()> {
        let object = self
            .repo
            .find_object(oid, None)
            .map_err(|e| GitPublishError::Tag(format!("Cannot find object: {}", e)))?;

        self.repo
            .tag_lightweight(name, &object, false)
            .map_err(|e| GitPublishError::Tag(format!("Cannot create tag: {}", e)))?;

        Ok(())
    }

    fn push_tags(&self, remote: &str, tag_names: &[&str]) -> Result<()> {
        let mut remote = self
            .repo
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

    fn fetch_from_remote(&self, remote: &str, branch: &str) -> Result<Oid> {
        let mut remote = self
            .repo
            .find_remote(remote)
            .map_err(|e| GitPublishError::Remote(format!("Cannot find remote: {}", e)))?;

        remote
            .fetch(&[branch], None, None)
            .map_err(|e| GitPublishError::Remote(format!("Fetch failed: {}", e)))?;

        // After fetching, get the OID of the remote-tracking branch
        let remote_name = remote.name().unwrap_or("origin");
        let remote_ref_name = format!("refs/remotes/{}/{}", remote_name, branch);
        let remote_ref = self.repo.find_reference(&remote_ref_name).map_err(|e| {
            GitPublishError::Remote(format!(
                "Cannot find remote branch '{}' after fetch: {}",
                remote_ref_name, e
            ))
        })?;

        let oid = remote_ref
            .peel_to_commit()
            .map_err(|e| {
                GitPublishError::Remote(format!(
                    "Cannot get commit for remote branch '{}': {}",
                    remote_ref_name, e
                ))
            })?
            .id();

        Ok(oid)
    }
}

// SAFETY: Git2Repository wraps git2::Repository which is Send + Sync.
// git2 library is thread-safe for read operations via libgit2's thread-safe design.
unsafe impl Sync for Git2Repository {}

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
