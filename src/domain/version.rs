use crate::domain::PreRelease;
use crate::error::{GitPublishError, Result};
use std::fmt;

/// Semantic version representation
///
/// Supports standard semantic versioning (MAJOR.MINOR.PATCH) with optional pre-release versions.
/// According to https://semver.org/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<PreRelease>,
}

impl Version {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version {
            major,
            minor,
            patch,
            prerelease: None,
        }
    }

    /// Create a new version with pre-release
    pub fn with_prerelease(
        major: u32,
        minor: u32,
        patch: u32,
        prerelease: Option<PreRelease>,
    ) -> Self {
        Version {
            major,
            minor,
            patch,
            prerelease,
        }
    }

    /// Parse version from a tag string (e.g., "v1.2.3" or "v1.2.3-beta.1")
    pub fn parse(tag: &str) -> Result<Self> {
        let clean_tag = tag.trim_start_matches('v').trim_start_matches('V');

        // Split on '-' to separate version from pre-release
        let (version_part, prerelease_part) = if let Some(pos) = clean_tag.find('-') {
            (&clean_tag[..pos], Some(&clean_tag[pos + 1..]))
        } else {
            (clean_tag, None)
        };

        // Parse semantic version part
        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() != 3 {
            return Err(GitPublishError::version(format!(
                "Invalid version format: '{}' - expected X.Y.Z or X.Y.Z-PRERELEASE",
                tag
            )));
        }

        let major = parts[0].parse::<u32>().map_err(|_| {
            GitPublishError::version(format!("Invalid major version: {}", parts[0]))
        })?;
        let minor = parts[1].parse::<u32>().map_err(|_| {
            GitPublishError::version(format!("Invalid minor version: {}", parts[1]))
        })?;
        let patch = parts[2].parse::<u32>().map_err(|_| {
            GitPublishError::version(format!("Invalid patch version: {}", parts[2]))
        })?;

        // Parse pre-release if present
        let prerelease = if let Some(pr_str) = prerelease_part {
            Some(PreRelease::parse(pr_str)?)
        } else {
            None
        };

        Ok(Version {
            major,
            minor,
            patch,
            prerelease,
        })
    }

    /// Bump version according to bump type
    pub fn bump(&self, bump_type: &VersionBump) -> Self {
        match bump_type {
            VersionBump::Major => Version {
                major: self.major + 1,
                minor: 0,
                patch: 0,
                prerelease: None,
            },
            VersionBump::Minor => Version {
                major: self.major,
                minor: self.minor + 1,
                patch: 0,
                prerelease: None,
            },
            VersionBump::Patch => Version {
                major: self.major,
                minor: self.minor,
                patch: self.patch + 1,
                prerelease: None,
            },
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pr) = self.prerelease {
            write!(f, "-{}", pr)?;
        }
        Ok(())
    }
}

/// Version bump type decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionBump {
    Major,
    Minor,
    Patch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::PreReleaseType;

    // Basic version tests
    #[test]
    fn test_version_parse() {
        let v = Version::parse("v1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.prerelease, None);
    }

    #[test]
    fn test_version_parse_without_v() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v, Version::new(1, 2, 3));
    }

    #[test]
    fn test_version_parse_uppercase_v() {
        let v = Version::parse("V1.2.3").unwrap();
        assert_eq!(v, Version::new(1, 2, 3));
    }

    #[test]
    fn test_version_parse_invalid() {
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("v1.2.3.4").is_err());
    }

    // Pre-release version tests
    #[test]
    fn test_version_parse_with_prerelease_alpha() {
        let v = Version::parse("v1.0.0-alpha").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
        assert!(v.prerelease.is_some());
        let pr = v.prerelease.unwrap();
        assert_eq!(pr.identifier, PreReleaseType::Alpha);
        assert_eq!(pr.iteration, None);
    }

    #[test]
    fn test_version_parse_with_prerelease_beta_iteration() {
        let v = Version::parse("v1.2.3-beta.1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        let pr = v.prerelease.unwrap();
        assert_eq!(pr.identifier, PreReleaseType::Beta);
        assert_eq!(pr.iteration, Some(1));
    }

    #[test]
    fn test_version_parse_with_rc() {
        let v = Version::parse("v2.1.3-rc.2").unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 3);
        let pr = v.prerelease.unwrap();
        assert_eq!(pr.identifier, PreReleaseType::ReleaseCandidate);
        assert_eq!(pr.iteration, Some(2));
    }

    #[test]
    fn test_version_parse_with_custom_prerelease() {
        let v = Version::parse("v1.0.0-staging.1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
        let pr = v.prerelease.unwrap();
        assert_eq!(pr.identifier, PreReleaseType::Custom("staging".to_string()));
        assert_eq!(pr.iteration, Some(1));
    }

    #[test]
    fn test_version_parse_prerelease_invalid() {
        assert!(Version::parse("v1.0.0-invalid!").is_err());
    }

    // Bump tests
    #[test]
    fn test_version_bump_major() {
        let v = Version::new(1, 2, 3);
        let bumped = v.bump(&VersionBump::Major);
        assert_eq!(bumped, Version::new(2, 0, 0));
        assert_eq!(bumped.prerelease, None);
    }

    #[test]
    fn test_version_bump_minor() {
        let v = Version::new(1, 2, 3);
        let bumped = v.bump(&VersionBump::Minor);
        assert_eq!(bumped, Version::new(1, 3, 0));
        assert_eq!(bumped.prerelease, None);
    }

    #[test]
    fn test_version_bump_patch() {
        let v = Version::new(1, 2, 3);
        let bumped = v.bump(&VersionBump::Patch);
        assert_eq!(bumped, Version::new(1, 2, 4));
        assert_eq!(bumped.prerelease, None);
    }

    #[test]
    fn test_version_bump_removes_prerelease() {
        let v = Version::parse("v1.0.0-beta.1").unwrap();
        let bumped = v.bump(&VersionBump::Patch);
        assert_eq!(bumped.prerelease, None);
    }

    // Display tests
    #[test]
    fn test_version_display() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_version_display_with_prerelease() {
        let v = Version::parse("v1.2.3-beta.1").unwrap();
        assert_eq!(v.to_string(), "1.2.3-beta.1");
    }

    #[test]
    fn test_version_display_with_alpha() {
        let v = Version::parse("v2.0.0-alpha").unwrap();
        assert_eq!(v.to_string(), "2.0.0-alpha");
    }

    // with_prerelease tests
    #[test]
    fn test_version_with_prerelease() {
        let pr = PreRelease::parse("rc.3").unwrap();
        let v = Version::with_prerelease(1, 5, 2, Some(pr));
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 5);
        assert_eq!(v.patch, 2);
        assert!(v.prerelease.is_some());
    }

    #[test]
    fn test_version_equality_without_prerelease() {
        let v1 = Version::parse("1.2.3").unwrap();
        let v2 = Version::new(1, 2, 3);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_version_inequality_different_prerelease() {
        let v1 = Version::parse("1.2.3-alpha").unwrap();
        let v2 = Version::parse("1.2.3-beta").unwrap();
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_version_inequality_prerelease_vs_none() {
        let v1 = Version::parse("1.2.3-alpha").unwrap();
        let v2 = Version::new(1, 2, 3);
        assert_ne!(v1, v2);
    }
}
