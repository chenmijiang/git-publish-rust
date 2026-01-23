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
}
