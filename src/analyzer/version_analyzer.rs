use crate::config::ConventionalCommitsConfig;
use crate::domain::{ParsedCommit, VersionBump};
use crate::error::Result;
use crate::git::Repository;
use git2::Oid;

/// Analyzes commits to determine version bump type
pub struct VersionAnalyzer {
    config: ConventionalCommitsConfig,
}

impl VersionAnalyzer {
    /// Create a new version analyzer
    pub fn new(config: ConventionalCommitsConfig) -> Self {
        VersionAnalyzer { config }
    }

    /// Analyze commits from a repository between two OIDs to determine version bump
    pub fn analyze_repository_range<R: Repository>(
        &self,
        repo: &R,
        from_oid: Oid,
        to_oid: Oid,
    ) -> Result<VersionBump> {
        let commits = repo.get_commits_between(from_oid, to_oid)?;
        let messages: Vec<String> = commits.into_iter().map(|c| c.message).collect();
        Ok(self.analyze_messages(&messages))
    }

    /// Analyze commit messages and determine version bump
    pub fn analyze_messages(&self, messages: &[String]) -> VersionBump {
        let mut has_breaking = false;
        let mut has_features = false;
        let mut has_fixes = false;

        for message in messages {
            let parsed = ParsedCommit::parse(message);

            // Check for breaking changes (highest priority)
            if parsed.is_breaking_change {
                has_breaking = true;
            }

            // Check for major version indicators
            for keyword in &self.config.major_keywords {
                if message.to_lowercase().contains(keyword) {
                    has_features = true;
                }
            }

            // Check for minor version indicators
            for keyword in &self.config.minor_keywords {
                if message.to_lowercase().contains(keyword) {
                    has_features = true;
                }
            }

            // Check for commit types that might indicate features or fixes
            match parsed.r#type.as_str() {
                "feat" | "feature" => has_features = true,
                "fix" | "perf" | "refactor" => has_fixes = true,
                _ => {}
            }

            // If we found a breaking change, we can return early
            if has_breaking {
                return VersionBump::Major;
            }
        }

        if has_features {
            VersionBump::Minor
        } else if has_fixes {
            VersionBump::Patch
        } else {
            // If no conventional commits detected, default to patch
            VersionBump::Patch
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::ConventionalCommitsConfig;

    use super::*;

    #[test]
    fn test_analyze_major() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec![
            "feat: new feature".to_string(),
            "fix(api)!: breaking change".to_string(),
        ];

        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Major);
    }

    #[test]
    fn test_analyze_minor() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec!["feat: new feature".to_string(), "fix: bug fix".to_string()];

        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Minor);
    }

    #[test]
    fn test_analyze_patch() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec![
            "fix: bug fix".to_string(),
            "refactor: code cleanup".to_string(),
        ];

        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Patch);
    }

    #[test]
    fn test_analyze_empty() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec!["docs: update readme".to_string()];

        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Patch);
    }

    // Integration tests: real-world commit scenarios
    #[test]
    fn test_analyze_single_breaking_change() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec!["feat(api)!: redesign endpoint".to_string()];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Major);
    }

    #[test]
    fn test_analyze_single_feature() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec!["feat(auth): add oauth support".to_string()];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Minor);
    }

    #[test]
    fn test_analyze_single_fix() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec!["fix(ui): button styling".to_string()];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Patch);
    }

    #[test]
    fn test_analyze_mixed_commits_features_and_fixes() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec![
            "feat(api): add endpoint".to_string(),
            "fix(ui): button color".to_string(),
            "fix(db): connection pool".to_string(),
        ];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Minor);
    }

    #[test]
    fn test_analyze_multiple_fixes_no_features() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec![
            "fix: bug 1".to_string(),
            "fix: bug 2".to_string(),
            "perf: optimize".to_string(),
            "refactor: cleanup".to_string(),
        ];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Patch);
    }

    #[test]
    fn test_analyze_breaking_change_via_footer() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages =
            vec!["fix: rename API field\n\nBREAKING CHANGE: field changed from X to Y".to_string()];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Major);
    }

    #[test]
    fn test_analyze_priority_breaking_over_features() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec![
            "feat: new feature 1".to_string(),
            "feat: new feature 2".to_string(),
            "fix(core)!: breaking change".to_string(),
        ];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Major);
    }

    #[test]
    fn test_analyze_ignore_docs_and_chore() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec![
            "docs: update readme".to_string(),
            "chore: update deps".to_string(),
            "style: format code".to_string(),
            "test: add tests".to_string(),
        ];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Patch);
    }

    #[test]
    fn test_analyze_real_release_cycle() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        // Simulate a release cycle from v1.0.0 to v1.1.0
        let messages = vec![
            "feat(api): add user list endpoint".to_string(),
            "feat(auth): add role-based access".to_string(),
            "fix(ui): modal alignment".to_string(),
            "docs: update api docs".to_string(),
        ];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Minor);
    }

    #[test]
    fn test_analyze_major_release_scenario() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        // Simulate major version bump
        let messages = vec![
            "feat(core)!: rewrite core engine".to_string(),
            "feat(api)!: new response format".to_string(),
            "feat(auth): add oauth2".to_string(),
            "fix: various bugs".to_string(),
        ];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Major);
    }

    #[test]
    fn test_analyze_patch_release_scenario() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        // Simulate patch version bump (bug fixes only)
        let messages = vec![
            "fix(api): handle null values".to_string(),
            "fix(db): query optimization".to_string(),
            "perf: cache results".to_string(),
        ];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Patch);
    }

    #[test]
    fn test_analyze_many_commits() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec![
            "chore: bump deps".to_string(),
            "docs: add faq".to_string(),
            "style: format code".to_string(),
            "test: add unit tests".to_string(),
            "test: add e2e tests".to_string(),
            "refactor: extract module".to_string(),
            "fix: edge case handling".to_string(),
            "feat: new search feature".to_string(),
        ];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Minor);
    }

    #[test]
    fn test_analyze_empty_message() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec!["".to_string()];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Patch);
    }

    #[test]
    fn test_analyze_non_conventional_commits() {
        let config = ConventionalCommitsConfig::default();
        let analyzer = VersionAnalyzer::new(config);

        let messages = vec![
            "Updated stuff".to_string(),
            "Fixed things".to_string(),
            "Added more stuff".to_string(),
        ];
        assert_eq!(analyzer.analyze_messages(&messages), VersionBump::Patch);
    }
}
