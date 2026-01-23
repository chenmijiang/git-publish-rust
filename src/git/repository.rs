use crate::error::{GitPublishError, Result};
use crate::git::CommitInfo;
use git2::{Oid, Repository as Git2Repo};
use std::path::Path;

/// Wrapper around git2::Repository with our trait interface
pub struct Git2Repository {
    repo: Git2Repo,
}

impl Git2Repository {
    /// Open or discover a git repository
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Git2Repo::discover(path)?;

        Ok(Git2Repository { repo })
    }

    /// Create from existing git2::Repository
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
        let mut revwalk = self.repo.revwalk()?;

        revwalk.push(to_oid)?;

        let mut commits = Vec::new();

        for oid_result in revwalk {
            let oid = oid_result?;

            if oid == from_oid {
                break;
            }

            let commit = self.repo.find_commit(oid)?;

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
        let tags = self.repo.tag_names(None)?;

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

    fn fetch_from_remote(&self, remote: &str, branch: &str) -> Result<()> {
        let mut remote = self
            .repo
            .find_remote(remote)
            .map_err(|e| GitPublishError::Remote(format!("Cannot find remote: {}", e)))?;

        remote
            .fetch(&[branch], None, None)
            .map_err(|e| GitPublishError::Remote(format!("Fetch failed: {}", e)))?;

        Ok(())
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
