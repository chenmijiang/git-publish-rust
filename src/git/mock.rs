use crate::error::Result;
use crate::git::{CommitInfo, Repository};
use git2::Oid;
use std::collections::HashMap;

/// Mock implementation of the Repository trait for testing
///
/// This implementation simulates Git operations without requiring an actual Git repository.
/// It stores commits, tags, and branch heads in memory and returns predefined values.
/// This enables fast, deterministic tests without external dependencies or file system operations.
///
/// # Usage
///
/// The mock repository starts empty. Tests should populate it with relevant data before
/// using it with the trait methods.
///
/// ```rust
/// # use git_publish::git::{MockRepository, Repository, CommitInfo};
/// # use git2::Oid;
/// let mut repo = MockRepository::new();
/// let oid = Oid::from_str("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2").unwrap();
///
/// repo.add_commit(
///     oid,
///     CommitInfo {
///         hash: "a1b2c3d4e5f6".to_string(),
///         message: "Test commit".to_string(),
///         author: "Test Author".to_string(),
///     }
/// );
/// repo.set_branch_head("main", oid);
///
/// assert_eq!(repo.get_branch_head_oid("main").unwrap(), oid);
/// ```
pub struct MockRepository {
    /// Map of OIDs to commit information
    commits: HashMap<Oid, CommitInfo>,
    /// Map of tag names to their OIDs
    tags: HashMap<String, Oid>,
    /// Map of branch names to their HEAD OIDs
    branch_heads: HashMap<String, Oid>,
}

impl MockRepository {
    /// Create a new empty mock repository
    ///
    /// The repository starts with no commits, tags, or branches configured.
    ///
    /// # Returns
    /// * `MockRepository` - A new empty mock repository instance
    ///
    /// # Example
    /// ```rust
    /// # use git_publish::git::{mock::MockRepository, Repository};
    /// let repo = MockRepository::new();
    /// assert!(repo.list_tags().unwrap().is_empty());
    /// ```
    pub fn new() -> Self {
        MockRepository {
            commits: HashMap::new(),
            tags: HashMap::new(),
            branch_heads: HashMap::new(),
        }
    }

    /// Add a commit to the mock repository
    ///
    /// This associates an OID with commit information in the mock. The commit becomes
    /// available when using methods like `get_commits_between`.
    ///
    /// # Arguments
    /// * `oid` - The object ID to associate with the commit
    /// * `info` - The commit information to store
    ///
    /// # Example
    /// ```rust
    /// # use git_publish::git::{mock::MockRepository, CommitInfo};
    /// # use git2::Oid;
    /// let mut repo = MockRepository::new();
    /// let oid = Oid::from_str("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2").unwrap();
    ///
    /// repo.add_commit(
    ///     oid,
    ///     CommitInfo {
    ///         hash: "a1b2c3d4e5f6".to_string(),
    ///         message: "Initial commit".to_string(),
    ///         author: "Alice".to_string(),
    ///     }
    /// );
    /// ```
    pub fn add_commit(&mut self, oid: Oid, info: CommitInfo) {
        self.commits.insert(oid, info);
    }

    /// Add a tag pointing to an OID
    ///
    /// This creates a mapping from a tag name to an object ID in the mock.
    ///
    /// # Arguments
    /// * `name` - The name of the tag (e.g., "v1.0.0", "release-1.2")
    /// * `oid` - The object ID the tag should point to
    ///
    /// # Example
    /// ```rust
    /// # use git_publish::git::{mock::MockRepository, Repository};
    /// # use git2::Oid;
    /// let mut repo = MockRepository::new();
    /// let oid = Oid::from_str("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2").unwrap();
    ///
    /// repo.add_tag("v1.0.0", oid);
    ///
    /// assert_eq!(repo.find_tag_oid("v1.0.0").unwrap(), Some(oid));
    /// ```
    pub fn add_tag(&mut self, name: impl Into<String>, oid: Oid) {
        self.tags.insert(name.into(), oid);
    }

    /// Set a branch head to point to a specific OID
    ///
    /// This determines what OID will be returned when `get_branch_head_oid` is called
    /// for the given branch.
    ///
    /// # Arguments
    /// * `branch` - The name of the branch (e.g., "main", "develop", "feature/new-ui")
    /// * `oid` - The object ID the branch should point to
    ///
    /// # Example
    /// ```rust
    /// # use git_publish::git::{mock::MockRepository, Repository};
    /// # use git2::Oid;
    /// let mut repo = MockRepository::new();
    /// let main_oid = Oid::from_str("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2").unwrap();
    ///
    /// repo.set_branch_head("main", main_oid);
    ///
    /// assert_eq!(repo.get_branch_head_oid("main").unwrap(), main_oid);
    /// ```
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
        self.branch_heads.get(branch_name).copied().ok_or_else(|| {
            crate::error::GitPublishError::branch(format!("Branch not found: {}", branch_name))
        })
    }

    fn get_commits_between(&self, from_oid: Oid, to_oid: Oid) -> Result<Vec<CommitInfo>> {
        // Implement more realistic git semantics for the mock:
        // - If from_oid == to_oid: return that specific commit if it exists
        // - Otherwise: return all commits in the mock except the from_oid (git .. semantics)

        if from_oid == to_oid {
            // Git semantics: X..X should return no commits, but for this mock
            // returning the specific commit might be more useful for tests
            if let Some(commit) = self.commits.get(&from_oid) {
                Ok(vec![commit.clone()])
            } else {
                Ok(vec![])
            }
        } else {
            // Return all commits except the from_oid (git A..B semantics)
            let mut commits: Vec<_> = self
                .commits
                .iter()
                .filter(|(oid, _)| *oid != &from_oid)
                .map(|(_, info)| info.clone())
                .collect();

            commits.sort_by(|a, b| a.hash.cmp(&b.hash));
            Ok(commits)
        }
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

    fn fetch_from_remote(&self, _remote: &str, _branch: &str) -> Result<Oid> {
        // In the mock, return a dummy OID since we don't actually perform Git operations
        Ok(Oid::from_bytes(&[0; 20]).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_repository_basic() {
        let mut repo = MockRepository::new();
        let oid = Oid::from_bytes(&[1; 20]).unwrap();

        repo.add_commit(
            oid,
            CommitInfo {
                hash: "abc123".to_string(),
                message: "test commit".to_string(),
                author: "Test Author".to_string(),
            },
        );

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

    #[test]
    fn test_mock_repository_list_tags() {
        let mut repo = MockRepository::new();
        let oid1 = Oid::from_bytes(&[1; 20]).unwrap();
        let oid2 = Oid::from_bytes(&[2; 20]).unwrap();

        repo.add_tag("v1.0.0", oid1);
        repo.add_tag("v2.0.0", oid2);

        let tags = repo.list_tags().unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"v1.0.0".to_string()));
        assert!(tags.contains(&"v2.0.0".to_string()));
    }

    #[test]
    fn test_mock_repository_get_commits() {
        let mut repo = MockRepository::new();
        let oid1 = Oid::from_bytes(&[1; 20]).unwrap();
        let oid2 = Oid::from_bytes(&[2; 20]).unwrap();

        repo.add_commit(
            oid1,
            CommitInfo {
                hash: "abc123".to_string(),
                message: "first commit".to_string(),
                author: "Author 1".to_string(),
            },
        );

        repo.add_commit(
            oid2,
            CommitInfo {
                hash: "def456".to_string(),
                message: "second commit".to_string(),
                author: "Author 2".to_string(),
            },
        );

        // Test that get_commits_between returns all commits except from_oid (git .. semantics)
        let commits = repo.get_commits_between(oid1, oid2).unwrap();
        // Should return all commits in the repo except oid1 (the from_oid)
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].hash, "def456");
    }

    #[test]
    fn test_mock_repository_default() {
        let repo = MockRepository::default();
        assert!(repo.list_tags().unwrap().is_empty());
    }

    // Integration tests: advanced repository scenarios
    #[test]
    fn test_mock_repository_multiple_branches() {
        let mut repo = MockRepository::new();
        let main_oid = Oid::from_bytes(&[1; 20]).unwrap();
        let develop_oid = Oid::from_bytes(&[2; 20]).unwrap();
        let feature_oid = Oid::from_bytes(&[3; 20]).unwrap();

        repo.set_branch_head("main", main_oid);
        repo.set_branch_head("develop", develop_oid);
        repo.set_branch_head("feature/xyz", feature_oid);

        assert_eq!(repo.get_branch_head_oid("main").unwrap(), main_oid);
        assert_eq!(repo.get_branch_head_oid("develop").unwrap(), develop_oid);
        assert_eq!(
            repo.get_branch_head_oid("feature/xyz").unwrap(),
            feature_oid
        );
    }

    #[test]
    fn test_mock_repository_missing_branch() {
        let repo = MockRepository::new();
        let result = repo.get_branch_head_oid("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_repository_multiple_tags_on_commits() {
        let mut repo = MockRepository::new();
        let v1_oid = Oid::from_bytes(&[1; 20]).unwrap();
        let v2_oid = Oid::from_bytes(&[2; 20]).unwrap();
        let v3_oid = Oid::from_bytes(&[3; 20]).unwrap();

        repo.add_tag("v1.0.0", v1_oid);
        repo.add_tag("v1.1.0", v2_oid);
        repo.add_tag("v2.0.0-beta.1", v2_oid);
        repo.add_tag("v2.0.0", v3_oid);

        let tags = repo.list_tags().unwrap();
        assert_eq!(tags.len(), 4);

        // Verify tag lookups
        assert_eq!(repo.find_tag_oid("v1.0.0").unwrap(), Some(v1_oid));
        assert_eq!(repo.find_tag_oid("v1.1.0").unwrap(), Some(v2_oid));
        assert_eq!(repo.find_tag_oid("v2.0.0-beta.1").unwrap(), Some(v2_oid));
        assert_eq!(repo.find_tag_oid("v2.0.0").unwrap(), Some(v3_oid));
    }

    #[test]
    fn test_mock_repository_commit_with_metadata() {
        let mut repo = MockRepository::new();
        let commit_oid = Oid::from_bytes(&[5; 20]).unwrap();

        let commit = CommitInfo {
            hash: "abc1234567890def".to_string(),
            message: "feat(api): add new endpoint\n\nThis is a detailed description.".to_string(),
            author: "John Doe <john@example.com>".to_string(),
        };

        repo.add_commit(commit_oid, commit);
        repo.set_branch_head("main", commit_oid);

        let commits = repo.get_commits_between(commit_oid, commit_oid).unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].hash, "abc1234567890def");
    }

    #[test]
    fn test_mock_repository_workflow_scenario() {
        let mut repo = MockRepository::new();

        // Create commits
        let c1 = Oid::from_bytes(&[1; 20]).unwrap();
        let c2 = Oid::from_bytes(&[2; 20]).unwrap();
        let c3 = Oid::from_bytes(&[3; 20]).unwrap();
        let c4 = Oid::from_bytes(&[4; 20]).unwrap();

        repo.add_commit(
            c1,
            CommitInfo {
                hash: "commit1".to_string(),
                message: "chore: initial commit".to_string(),
                author: "Developer".to_string(),
            },
        );
        repo.add_commit(
            c2,
            CommitInfo {
                hash: "commit2".to_string(),
                message: "feat: feature 1".to_string(),
                author: "Developer".to_string(),
            },
        );
        repo.add_commit(
            c3,
            CommitInfo {
                hash: "commit3".to_string(),
                message: "fix: bug fix".to_string(),
                author: "Developer".to_string(),
            },
        );
        repo.add_commit(
            c4,
            CommitInfo {
                hash: "commit4".to_string(),
                message: "feat(api)!: breaking change".to_string(),
                author: "Developer".to_string(),
            },
        );

        // Set up branches
        repo.set_branch_head("main", c3);
        repo.set_branch_head("develop", c4);

        // Tag some releases
        repo.add_tag("v1.0.0", c1);
        repo.add_tag("v1.1.0", c3);

        // Verify state
        assert_eq!(repo.get_branch_head_oid("main").unwrap(), c3);
        assert_eq!(repo.get_branch_head_oid("develop").unwrap(), c4);
        assert_eq!(repo.find_tag_oid("v1.0.0").unwrap(), Some(c1));
        assert_eq!(repo.find_tag_oid("v1.1.0").unwrap(), Some(c3));
        assert!(repo.find_tag_oid("v2.0.0").unwrap().is_none());

        let tags = repo.list_tags().unwrap();
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_mock_repository_prerelease_tags() {
        let mut repo = MockRepository::new();

        let beta1_oid = Oid::from_bytes(&[1; 20]).unwrap();
        let beta2_oid = Oid::from_bytes(&[2; 20]).unwrap();
        let rc_oid = Oid::from_bytes(&[3; 20]).unwrap();
        let stable_oid = Oid::from_bytes(&[4; 20]).unwrap();

        repo.add_tag("v2.0.0-beta.1", beta1_oid);
        repo.add_tag("v2.0.0-beta.2", beta2_oid);
        repo.add_tag("v2.0.0-rc.1", rc_oid);
        repo.add_tag("v2.0.0", stable_oid);

        let tags = repo.list_tags().unwrap();
        assert_eq!(tags.len(), 4);

        // Verify lookups
        assert!(tags.contains(&"v2.0.0-beta.1".to_string()));
        assert!(tags.contains(&"v2.0.0-beta.2".to_string()));
        assert!(tags.contains(&"v2.0.0-rc.1".to_string()));
        assert!(tags.contains(&"v2.0.0".to_string()));
    }

    #[test]
    fn test_mock_repository_various_commit_types() {
        let mut repo = MockRepository::new();

        let oid1 = Oid::from_bytes(&[1; 20]).unwrap();
        let oid2 = Oid::from_bytes(&[2; 20]).unwrap();
        let oid3 = Oid::from_bytes(&[3; 20]).unwrap();
        let oid4 = Oid::from_bytes(&[4; 20]).unwrap();

        let commit_types = vec![
            ("feat(auth): add login", "feature"),
            ("fix(ui): button alignment", "bugfix"),
            ("docs: update readme", "documentation"),
            ("refactor(core): simplify", "refactoring"),
        ];

        for (i, (msg, _type_label)) in commit_types.iter().enumerate() {
            let oid = match i {
                0 => oid1,
                1 => oid2,
                2 => oid3,
                3 => oid4,
                _ => unreachable!(),
            };
            repo.add_commit(
                oid,
                CommitInfo {
                    hash: format!("hash{}", i),
                    message: msg.to_string(),
                    author: "Dev".to_string(),
                },
            );
        }

        repo.set_branch_head("main", oid4);

        let commits = repo.get_commits_between(oid1, oid4).unwrap();
        // Should return all commits except oid1 (git .. semantics)
        assert_eq!(commits.len(), 3);
    }

    #[test]
    fn test_mock_repository_empty_state() {
        let repo = MockRepository::new();

        assert!(repo.list_tags().unwrap().is_empty());
        assert!(repo.find_tag_oid("v1.0.0").unwrap().is_none());

        assert!(repo.get_branch_head_oid("main").is_err());
    }
}

#[cfg(test)]
mod mock_filtering_tests {
    use super::*;

    #[test]
    fn test_mock_repository_commit_filtering() {
        let mut repo = MockRepository::new();
        let oid1 = Oid::from_bytes(&[1; 20]).unwrap();
        let oid2 = Oid::from_bytes(&[2; 20]).unwrap();
        let oid3 = Oid::from_bytes(&[3; 20]).unwrap();

        repo.add_commit(
            oid1,
            CommitInfo {
                hash: "aaa".to_string(),
                message: "commit 1".to_string(),
                author: "Author".to_string(),
            },
        );
        repo.add_commit(
            oid2,
            CommitInfo {
                hash: "bbb".to_string(),
                message: "commit 2".to_string(),
                author: "Author".to_string(),
            },
        );
        repo.add_commit(
            oid3,
            CommitInfo {
                hash: "ccc".to_string(),
                message: "commit 3".to_string(),
                author: "Author".to_string(),
            },
        );

        // Test the commit filtering behavior - should return all commits except from_oid
        let commits = repo.get_commits_between(oid1, oid3).unwrap();

        // Should return all commits in the mock except oid1 (the from_oid)
        assert_eq!(commits.len(), 2);
        // The result includes both oid2 ("bbb") and oid3 ("ccc"), excluding oid1
    }
}
