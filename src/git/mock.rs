use crate::error::Result;
use crate::git::{CommitInfo, Repository};
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
        self.branch_heads.get(branch_name).copied().ok_or_else(|| {
            crate::error::GitPublishError::branch(format!("Branch not found: {}", branch_name))
        })
    }

    fn get_commits_between(&self, _from_oid: Oid, _to_oid: Oid) -> Result<Vec<CommitInfo>> {
        // Simplified: return commits in order from hashmap
        let mut commits: Vec<_> = self.commits.values().cloned().collect();
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

        let commits = repo.get_commits_between(oid1, oid2).unwrap();
        assert_eq!(commits.len(), 2);
    }

    #[test]
    fn test_mock_repository_default() {
        let repo = MockRepository::default();
        assert!(repo.list_tags().unwrap().is_empty());
    }
}
