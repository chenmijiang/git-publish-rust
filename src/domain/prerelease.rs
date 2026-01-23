//! Pre-release version handling for semantic versioning
//!
//! Supports pre-release identifiers (alpha, beta, rc, and custom) with optional iteration numbers.
//! According to semver.org: https://semver.org/#spec-item-9

use crate::error::{GitPublishError, Result};
use std::fmt;
use std::str::FromStr;

/// Pre-release identifier type (alpha, beta, rc, or custom)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PreReleaseType {
    /// Alpha pre-release
    Alpha,
    /// Beta pre-release
    Beta,
    /// Release candidate
    ReleaseCandidate,
    /// Custom pre-release identifier
    Custom(String),
}

impl PreReleaseType {
    /// Parse a pre-release type from a string
    ///
    /// Accepts: "alpha", "a", "beta", "b", "rc", or any custom alphanumeric-hyphen string
    ///
    /// # Arguments
    /// * `s` - String to parse
    ///
    /// # Returns
    /// * `Ok(PreReleaseType)` - Parsed pre-release type
    /// * `Err` - If string contains invalid characters
    pub fn parse(s: &str) -> Result<Self> {
        s.parse()
    }
}

impl FromStr for PreReleaseType {
    type Err = GitPublishError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "alpha" | "a" => Ok(PreReleaseType::Alpha),
            "beta" | "b" => Ok(PreReleaseType::Beta),
            "rc" => Ok(PreReleaseType::ReleaseCandidate),
            other => {
                if other.chars().all(|c| c.is_alphanumeric() || c == '-') {
                    Ok(PreReleaseType::Custom(other.to_string()))
                } else {
                    Err(GitPublishError::version(format!(
                        "Invalid pre-release identifier: '{}'",
                        s
                    )))
                }
            }
        }
    }
}

impl fmt::Display for PreReleaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreReleaseType::Alpha => write!(f, "alpha"),
            PreReleaseType::Beta => write!(f, "beta"),
            PreReleaseType::ReleaseCandidate => write!(f, "rc"),
            PreReleaseType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Pre-release version with optional iteration number
///
/// Represents a pre-release version like "beta.1" or "alpha"
///
/// # Examples
/// - "alpha" -> PreRelease { identifier: Alpha, iteration: None }
/// - "beta.1" -> PreRelease { identifier: Beta, iteration: Some(1) }
/// - "rc.3" -> PreRelease { identifier: ReleaseCandidate, iteration: Some(3) }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreRelease {
    /// The pre-release identifier (alpha, beta, rc, or custom)
    pub identifier: PreReleaseType,
    /// Optional iteration number (incremented per release cycle)
    pub iteration: Option<u32>,
}

impl PreRelease {
    /// Create a new pre-release version
    ///
    /// # Arguments
    /// * `identifier` - Pre-release type
    /// * `iteration` - Optional iteration number
    ///
    /// # Returns
    /// A new PreRelease instance
    pub fn new(identifier: PreReleaseType, iteration: Option<u32>) -> Self {
        PreRelease {
            identifier,
            iteration,
        }
    }

    /// Parse a pre-release version from a string
    ///
    /// Accepts formats like "beta", "beta.1", "rc.2", or "custom-id.5"
    ///
    /// # Arguments
    /// * `s` - String to parse
    ///
    /// # Returns
    /// * `Ok(PreRelease)` - Parsed pre-release version
    /// * `Err` - If format is invalid
    ///
    /// # Examples
    /// ```ignore
    /// let pr = PreRelease::parse("beta.1")?;
    /// assert_eq!(pr.identifier, PreReleaseType::Beta);
    /// assert_eq!(pr.iteration, Some(1));
    /// ```
    pub fn parse(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(GitPublishError::version(
                "Empty pre-release identifier".to_string(),
            ));
        }

        let parts: Vec<&str> = s.split('.').collect();

        let identifier = PreReleaseType::parse(parts[0])?;

        let iteration = if parts.len() > 1 {
            Some(parts[1].parse::<u32>().map_err(|_| {
                GitPublishError::version(format!("Invalid iteration number: '{}'", parts[1]))
            })?)
        } else {
            None
        };

        Ok(PreRelease {
            identifier,
            iteration,
        })
    }

    /// Increment the iteration number
    ///
    /// If iteration is None, returns Some(1). Otherwise increments by 1.
    ///
    /// # Examples
    /// ```ignore
    /// let pr = PreRelease::parse("beta.1")?;
    /// let next = pr.increment_iteration();
    /// assert_eq!(next.iteration, Some(2));
    /// ```
    pub fn increment_iteration(&self) -> Self {
        let new_iteration = match self.iteration {
            Some(n) => Some(n + 1),
            None => Some(1),
        };

        PreRelease {
            identifier: self.identifier.clone(),
            iteration: new_iteration,
        }
    }
}

impl fmt::Display for PreRelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.identifier)?;
        if let Some(iter) = self.iteration {
            write!(f, ".{}", iter)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // PreReleaseType tests
    #[test]
    fn test_prerelease_type_parse_alpha() {
        let pr = PreReleaseType::parse("alpha").unwrap();
        assert_eq!(pr, PreReleaseType::Alpha);
    }

    #[test]
    fn test_prerelease_type_parse_alpha_short() {
        let pr = PreReleaseType::parse("a").unwrap();
        assert_eq!(pr, PreReleaseType::Alpha);
    }

    #[test]
    fn test_prerelease_type_parse_beta() {
        let pr = PreReleaseType::parse("beta").unwrap();
        assert_eq!(pr, PreReleaseType::Beta);
    }

    #[test]
    fn test_prerelease_type_parse_beta_short() {
        let pr = PreReleaseType::parse("b").unwrap();
        assert_eq!(pr, PreReleaseType::Beta);
    }

    #[test]
    fn test_prerelease_type_parse_rc() {
        let pr = PreReleaseType::parse("rc").unwrap();
        assert_eq!(pr, PreReleaseType::ReleaseCandidate);
    }

    #[test]
    fn test_prerelease_type_parse_custom() {
        let pr = PreReleaseType::parse("custom-name").unwrap();
        assert_eq!(pr, PreReleaseType::Custom("custom-name".to_string()));
    }

    #[test]
    fn test_prerelease_type_parse_custom_numeric() {
        let pr = PreReleaseType::parse("test123").unwrap();
        assert_eq!(pr, PreReleaseType::Custom("test123".to_string()));
    }

    #[test]
    fn test_prerelease_type_parse_invalid() {
        assert!(PreReleaseType::parse("invalid!name").is_err());
        assert!(PreReleaseType::parse("invalid.name").is_err());
    }

    #[test]
    fn test_prerelease_type_display_alpha() {
        assert_eq!(PreReleaseType::Alpha.to_string(), "alpha");
    }

    #[test]
    fn test_prerelease_type_display_beta() {
        assert_eq!(PreReleaseType::Beta.to_string(), "beta");
    }

    #[test]
    fn test_prerelease_type_display_rc() {
        assert_eq!(PreReleaseType::ReleaseCandidate.to_string(), "rc");
    }

    #[test]
    fn test_prerelease_type_display_custom() {
        assert_eq!(PreReleaseType::Custom("dev".to_string()).to_string(), "dev");
    }

    // PreRelease tests
    #[test]
    fn test_prerelease_parse_with_iteration() {
        let pr = PreRelease::parse("beta.1").unwrap();
        assert_eq!(pr.identifier, PreReleaseType::Beta);
        assert_eq!(pr.iteration, Some(1));
    }

    #[test]
    fn test_prerelease_parse_no_iteration() {
        let pr = PreRelease::parse("alpha").unwrap();
        assert_eq!(pr.identifier, PreReleaseType::Alpha);
        assert_eq!(pr.iteration, None);
    }

    #[test]
    fn test_prerelease_parse_rc_with_iteration() {
        let pr = PreRelease::parse("rc.2").unwrap();
        assert_eq!(pr.identifier, PreReleaseType::ReleaseCandidate);
        assert_eq!(pr.iteration, Some(2));
    }

    #[test]
    fn test_prerelease_parse_custom_with_iteration() {
        let pr = PreRelease::parse("dev.5").unwrap();
        assert_eq!(pr.identifier, PreReleaseType::Custom("dev".to_string()));
        assert_eq!(pr.iteration, Some(5));
    }

    #[test]
    fn test_prerelease_parse_invalid_iteration() {
        assert!(PreRelease::parse("beta.abc").is_err());
    }

    #[test]
    fn test_prerelease_parse_empty() {
        assert!(PreRelease::parse("").is_err());
    }

    #[test]
    fn test_prerelease_increment_with_iteration() {
        let pr = PreRelease::parse("beta.1").unwrap();
        let incremented = pr.increment_iteration();
        assert_eq!(incremented.identifier, PreReleaseType::Beta);
        assert_eq!(incremented.iteration, Some(2));
    }

    #[test]
    fn test_prerelease_increment_from_none() {
        let pr = PreRelease::new(PreReleaseType::Alpha, None);
        let incremented = pr.increment_iteration();
        assert_eq!(incremented.identifier, PreReleaseType::Alpha);
        assert_eq!(incremented.iteration, Some(1));
    }

    #[test]
    fn test_prerelease_increment_high_number() {
        let pr = PreRelease::parse("rc.99").unwrap();
        let incremented = pr.increment_iteration();
        assert_eq!(incremented.iteration, Some(100));
    }

    #[test]
    fn test_prerelease_display_with_iteration() {
        let pr = PreRelease::parse("rc.2").unwrap();
        assert_eq!(pr.to_string(), "rc.2");
    }

    #[test]
    fn test_prerelease_display_without_iteration() {
        let pr = PreRelease::parse("alpha").unwrap();
        assert_eq!(pr.to_string(), "alpha");
    }

    #[test]
    fn test_prerelease_display_custom_with_iteration() {
        let pr = PreRelease::parse("staging.3").unwrap();
        assert_eq!(pr.to_string(), "staging.3");
    }

    #[test]
    fn test_prerelease_equality() {
        let pr1 = PreRelease::parse("beta.1").unwrap();
        let pr2 = PreRelease::parse("beta.1").unwrap();
        assert_eq!(pr1, pr2);
    }

    #[test]
    fn test_prerelease_inequality_different_iteration() {
        let pr1 = PreRelease::parse("beta.1").unwrap();
        let pr2 = PreRelease::parse("beta.2").unwrap();
        assert_ne!(pr1, pr2);
    }

    #[test]
    fn test_prerelease_inequality_different_type() {
        let pr1 = PreRelease::parse("alpha.1").unwrap();
        let pr2 = PreRelease::parse("beta.1").unwrap();
        assert_ne!(pr1, pr2);
    }
}
