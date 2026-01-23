use crate::config::ConventionalCommitsConfig;
use crate::domain::{ParsedCommit, VersionBump};

/// Analyzes commits to determine version bump type
pub struct VersionAnalyzer {
    #[allow(dead_code)]
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
        }

        // Decision tree (priority order)
        if has_breaking {
            VersionBump::Major
        } else if has_features {
            VersionBump::Minor
        } else {
            VersionBump::Patch
        }
    }

    fn is_feature(&self, commit_type: &str) -> bool {
        commit_type == "feat" || commit_type == "feature"
    }

    #[allow(dead_code)]
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

        let messages = vec!["feat: new feature".to_string(), "fix: bug fix".to_string()];

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
