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
}
