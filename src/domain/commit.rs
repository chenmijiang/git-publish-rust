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
        // Try format: type(scope)!: description
        if let Some(captures) = Regex::new(r"^([a-z]+)\(([^)]+)\)(!?):\s*(.*)")
            .ok()
            .and_then(|re| re.captures(message))
        {
            let r#type = captures
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let scope = captures.get(2).map(|m| m.as_str().to_string());
            let has_exclamation = captures.get(3).map(|m| m.as_str()) == Some("!");
            let description = captures
                .get(4)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let is_breaking = has_exclamation || message.contains("BREAKING CHANGE:");

            return ParsedCommit {
                r#type,
                scope,
                description,
                is_breaking_change: is_breaking,
            };
        }

        // Try format: type!: description
        if let Some(captures) = Regex::new(r"^([a-z]+)!:\s*(.*)")
            .ok()
            .and_then(|re| re.captures(message))
        {
            let r#type = captures
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let description = captures
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            return ParsedCommit {
                r#type,
                scope: None,
                description,
                is_breaking_change: true,
            };
        }

        // Try format: type: description
        if let Some(captures) = Regex::new(r"^([a-z]+):\s*(.*)")
            .ok()
            .and_then(|re| re.captures(message))
        {
            let r#type = captures
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let description = captures
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let is_breaking = message.contains("BREAKING CHANGE:");

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

    // Integration tests: commit parsing variations
    #[test]
    fn test_parse_feat_types_variations() {
        let commits = vec![
            ("feat: new feature", "feat"),
            ("feat(api): new endpoint", "feat"),
            ("feat(api)!: breaking change", "feat"),
            ("fix: bug fix", "fix"),
            ("fix(ui)!: redesign", "fix"),
            ("docs: update readme", "docs"),
            ("style: format code", "style"),
            ("refactor: restructure", "refactor"),
            ("perf: optimize", "perf"),
            ("test: add tests", "test"),
            ("chore: dependencies", "chore"),
        ];

        for (msg, expected_type) in commits {
            let parsed = ParsedCommit::parse(msg);
            assert_eq!(parsed.r#type, expected_type, "Failed for: {}", msg);
        }
    }

    #[test]
    fn test_parse_scope_extraction() {
        let scopes = vec![
            ("feat(auth): login", Some("auth".to_string())),
            ("feat(api-v2): endpoint", Some("api-v2".to_string())),
            ("feat(core/parser): update", Some("core/parser".to_string())),
            ("feat: no scope", None),
            ("feat!: no scope breaking", None),
        ];

        for (msg, expected_scope) in scopes {
            let parsed = ParsedCommit::parse(msg);
            assert_eq!(parsed.scope, expected_scope, "Failed for: {}", msg);
        }
    }

    #[test]
    fn test_parse_breaking_change_detection() {
        let breaking = vec![
            "feat!: breaking feature",
            "feat(api)!: breaking api",
            "fix: something\n\nBREAKING CHANGE: explanation",
            "feat(auth): auth\n\nBREAKING CHANGE: removed endpoint",
        ];

        for msg in breaking {
            let parsed = ParsedCommit::parse(msg);
            assert!(parsed.is_breaking_change, "Should be breaking: {}", msg);
        }
    }

    #[test]
    fn test_parse_non_breaking_commit_types() {
        let non_breaking = vec![
            "docs: add documentation",
            "style: format code",
            "test: add test",
            "chore: update deps",
            "refactor: cleanup",
            "perf: optimize loop",
        ];

        for msg in non_breaking {
            let parsed = ParsedCommit::parse(msg);
            assert!(
                !parsed.is_breaking_change,
                "Should not be breaking: {}",
                msg
            );
        }
    }

    #[test]
    fn test_parse_description_extraction() {
        let descriptions = vec![
            ("feat(api): add user endpoint", "add user endpoint"),
            ("fix(ui): correct typo", "correct typo"),
            ("docs: update api guide", "update api guide"),
            ("Random text without format", "Random text without format"),
        ];

        for (msg, expected_desc) in descriptions {
            let parsed = ParsedCommit::parse(msg);
            assert_eq!(parsed.description, expected_desc, "Failed for: {}", msg);
        }
    }

    #[test]
    fn test_parse_multiline_commit() {
        let multiline = r#"feat(api): add new endpoint

This is a longer description of the feature.
It can span multiple lines.

BREAKING CHANGE: changed response format"#;

        let parsed = ParsedCommit::parse(multiline);
        assert_eq!(parsed.r#type, "feat");
        assert_eq!(parsed.scope, Some("api".to_string()));
        assert!(parsed.is_breaking_change);
    }

    #[test]
    fn test_parse_edge_cases() {
        let edge_cases = vec![
            ("fix:", "fix"),
            ("feat(scope):", "feat"),
            ("feat: ", "feat"),
            ("fix(db/migrate)!: schema", "fix"),
            ("chore: release v1.0.0", "chore"),
        ];

        for (msg, expected_type) in edge_cases {
            let parsed = ParsedCommit::parse(msg);
            assert_eq!(parsed.r#type, expected_type, "Failed for: {}", msg);
        }
    }
}
