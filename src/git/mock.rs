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
        assert_eq!(commits.len(), 4);
    }

    #[test]
    fn test_mock_repository_empty_state() {
        let repo = MockRepository::new();

        assert!(repo.list_tags().unwrap().is_empty());
        assert!(repo.find_tag_oid("v1.0.0").unwrap().is_none());

        assert!(repo.get_branch_head_oid("main").is_err());
    }
}
