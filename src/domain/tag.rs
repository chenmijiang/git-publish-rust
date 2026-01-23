use crate::error::{GitPublishError, Result};

/// Represents a git tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
}

impl Tag {
    /// Create a new tag from a string
    pub fn new(name: impl Into<String>) -> Self {
        Tag { name: name.into() }
    }

    /// Extract version number from tag (e.g., "v1.2.3" -> "1.2.3")
    pub fn version_part(&self) -> Result<String> {
        let trimmed = self.name.trim_start_matches('v').trim_start_matches('V');
        Ok(trimmed.to_string())
    }
}

/// Tag naming pattern (e.g., "v{version}", "release-{version}")
#[derive(Debug, Clone)]
pub struct TagPattern {
    pub pattern: String,
}

impl TagPattern {
    /// Create a new tag pattern
    pub fn new(pattern: impl Into<String>) -> Self {
        TagPattern {
            pattern: pattern.into(),
        }
    }

    /// Format a version according to pattern
    /// Example: pattern="v{version}", version="1.2.3" -> "v1.2.3"
    pub fn format(&self, version: &str) -> String {
        self.pattern.replace("{version}", version)
    }

    /// Validate if a tag matches this pattern
    pub fn matches(&self, tag: &str) -> Result<bool> {
        // Extract the placeholder pattern part
        if !self.pattern.contains("{version}") {
            return Err(GitPublishError::tag(
                "Pattern must contain {version} placeholder",
            ));
        }

        // Create regex pattern: escape everything, replace {version} with regex
        let escaped = regex::escape(&self.pattern);
        let regex_pattern = escaped.replace(r"\{version\}", r"(\d+\.\d+\.\d+)");

        if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            Ok(re.is_match(tag))
        } else {
            Err(GitPublishError::tag("Invalid pattern"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_new() {
        let tag = Tag::new("v1.2.3");
        assert_eq!(tag.name, "v1.2.3");
    }

    #[test]
    fn test_tag_version_part() {
        let tag = Tag::new("v1.2.3");
        assert_eq!(tag.version_part().unwrap(), "1.2.3");
    }

    #[test]
    fn test_pattern_format() {
        let pattern = TagPattern::new("v{version}");
        assert_eq!(pattern.format("1.2.3"), "v1.2.3");
    }

    #[test]
    fn test_pattern_format_with_suffix() {
        let pattern = TagPattern::new("release-{version}");
        assert_eq!(pattern.format("1.2.3"), "release-1.2.3");
    }

    #[test]
    fn test_pattern_matches() {
        let pattern = TagPattern::new("v{version}");
        assert!(pattern.matches("v1.2.3").unwrap());
        assert!(!pattern.matches("release-1.2.3").unwrap());
    }

    // Integration tests: tag patterns and versions
    #[test]
    fn test_pattern_format_multiple_versions() {
        let pattern = TagPattern::new("v{version}");

        assert_eq!(pattern.format("0.0.1"), "v0.0.1");
        assert_eq!(pattern.format("1.0.0"), "v1.0.0");
        assert_eq!(pattern.format("10.20.30"), "v10.20.30");
        assert_eq!(pattern.format("999.999.999"), "v999.999.999");
    }

    #[test]
    fn test_pattern_format_with_different_prefixes() {
        let patterns = vec![
            ("v{version}", "1.2.3", "v1.2.3"),
            ("release-{version}", "1.2.3", "release-1.2.3"),
            ("app-{version}", "2.0.0", "app-2.0.0"),
            ("{version}", "1.0.0", "1.0.0"),
            ("tag-{version}-final", "1.5.0", "tag-1.5.0-final"),
        ];

        for (pattern_str, version, expected) in patterns {
            let pattern = TagPattern::new(pattern_str);
            assert_eq!(pattern.format(version), expected);
        }
    }

    #[test]
    fn test_pattern_matches_multiple_versions() {
        let pattern = TagPattern::new("v{version}");

        assert!(pattern.matches("v0.0.0").unwrap());
        assert!(pattern.matches("v1.0.0").unwrap());
        assert!(pattern.matches("v999.999.999").unwrap());
        assert!(!pattern.matches("v1.0.0.0").unwrap());
        assert!(!pattern.matches("1.0.0").unwrap());
    }

    #[test]
    fn test_pattern_custom_format_matches() {
        let patterns = vec![
            ("v{version}", "v1.2.3"),
            ("release-{version}", "release-1.2.3"),
            ("app-{version}", "app-2.0.0"),
            ("{version}", "1.0.0"),
            ("v{version}-stable", "v1.5.0-stable"),
        ];

        for (pattern_str, tag) in patterns {
            let pattern = TagPattern::new(pattern_str);
            assert!(
                pattern.matches(tag).unwrap(),
                "Pattern {} should match {}",
                pattern_str,
                tag
            );
        }
    }

    #[test]
    fn test_tag_version_part_variations() {
        let tags = vec![
            ("v1.2.3", "1.2.3"),
            ("V1.2.3", "1.2.3"),
            ("release-1.2.3", "release-1.2.3"),
            ("1.0.0", "1.0.0"),
        ];

        for (tag_name, expected_version) in tags {
            let tag = Tag::new(tag_name);
            assert_eq!(tag.version_part().unwrap(), expected_version);
        }
    }

    #[test]
    fn test_pattern_roundtrip_format_and_match() {
        let pattern = TagPattern::new("v{version}");
        let version = "1.5.3";

        // Format version into tag
        let formatted = pattern.format(version);
        assert_eq!(formatted, "v1.5.3");

        // Verify formatted tag matches pattern
        assert!(pattern.matches(&formatted).unwrap());
    }

    #[test]
    fn test_pattern_mismatch_detection() {
        let pattern = TagPattern::new("v{version}");

        let non_matching = vec![
            "1.2.3",         // missing v prefix
            "release-1.2.3", // wrong prefix
            "v1.2",          // incomplete version
            "v1.2.3.4",      // too many version parts
            "version-1.2.3", // wrong format
        ];

        for tag in non_matching {
            assert!(!pattern.matches(tag).unwrap(), "Should not match: {}", tag);
        }
    }

    #[test]
    fn test_pattern_invalid_missing_placeholder() {
        let pattern = TagPattern::new("v-release");
        let result = pattern.matches("v-release");

        assert!(result.is_err());
    }
}
