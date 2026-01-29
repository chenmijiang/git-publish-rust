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

    /// Parse version from a tag string (e.g., "v1.2.3", "g1.2.3", or "v1.2.3-beta.1")
    ///
    /// Supports any single alphabetic character prefix (v, g, d, etc.) followed by
    /// a semantic version number.
    pub fn parse(tag: &str) -> Result<Self> {
        // Strip any single alphabetic character prefix (v, g, d, etc.)
        let clean_tag = if tag.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
            && tag.chars().nth(1).is_some_and(|c| c.is_ascii_digit())
        {
            &tag[1..]
        } else {
            tag
        };

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

    // Integration tests: pre-release version workflows
    #[test]
    fn test_prerelease_bump_from_stable_to_beta() {
        let _stable = Version::new(1, 0, 0);
        let pr = PreRelease::new(PreReleaseType::Beta, None);
        let beta = Version::with_prerelease(1, 0, 0, Some(pr));
        assert_eq!(
            beta.prerelease.as_ref().unwrap().identifier,
            PreReleaseType::Beta
        );
        assert_eq!(beta.prerelease.as_ref().unwrap().iteration, None);
    }

    #[test]
    fn test_prerelease_iteration_increment_workflow() {
        // Start with v1.0.0-beta.1
        let v1 = Version::parse("v1.0.0-beta.1").unwrap();

        // Increment to v1.0.0-beta.2
        let incremented = v1.prerelease.as_ref().unwrap().increment_iteration();
        let v2 = Version::with_prerelease(1, 0, 0, Some(incremented));

        assert_eq!(v2.to_string(), "1.0.0-beta.2");
    }

    #[test]
    fn test_prerelease_rc_to_stable_release() {
        // Start with v1.2.0-rc.3
        let rc = Version::parse("v1.2.0-rc.3").unwrap();

        // Release as stable v1.2.0
        let stable = rc.bump(&VersionBump::Patch);

        assert_eq!(stable.to_string(), "1.2.1");
        assert_eq!(stable.prerelease, None);
    }

    #[test]
    fn test_prerelease_mixed_version_types() {
        // Parse various pre-release formats
        let alpha = Version::parse("v1.0.0-alpha").unwrap();
        let beta1 = Version::parse("v1.0.0-beta.1").unwrap();
        let rc2 = Version::parse("v1.0.0-rc.2").unwrap();
        let custom = Version::parse("v1.0.0-dev.5").unwrap();

        assert_eq!(alpha.to_string(), "1.0.0-alpha");
        assert_eq!(beta1.to_string(), "1.0.0-beta.1");
        assert_eq!(rc2.to_string(), "1.0.0-rc.2");
        assert_eq!(custom.to_string(), "1.0.0-dev.5");
    }

    #[test]
    fn test_prerelease_custom_identifier() {
        let staging = Version::parse("v2.1.0-staging.10").unwrap();

        assert_eq!(staging.major, 2);
        assert_eq!(staging.minor, 1);
        assert_eq!(staging.patch, 0);
        let pr = staging.prerelease.unwrap();
        assert_eq!(pr.identifier, PreReleaseType::Custom("staging".to_string()));
        assert_eq!(pr.iteration, Some(10));
    }

    #[test]
    fn test_prerelease_bump_from_beta_preserves_major_minor() {
        let beta = Version::parse("v1.5.0-beta.1").unwrap();
        let minor_bump = beta.bump(&VersionBump::Minor);

        assert_eq!(minor_bump.major, 1);
        assert_eq!(minor_bump.minor, 6);
        assert_eq!(minor_bump.patch, 0);
        assert_eq!(minor_bump.prerelease, None);
    }

    #[test]
    fn test_prerelease_major_bump_clears_prerelease() {
        let rc = Version::parse("v1.9.9-rc.5").unwrap();
        let major_bump = rc.bump(&VersionBump::Major);

        assert_eq!(major_bump.to_string(), "2.0.0");
        assert_eq!(major_bump.prerelease, None);
    }

    #[test]
    fn test_prerelease_roundtrip_parse_display() {
        let versions = vec![
            "1.0.0-alpha",
            "1.0.0-beta.1",
            "1.0.0-rc.2",
            "2.3.4-staging.10",
            "0.0.1-alpha",
        ];

        for version_str in versions {
            let parsed = Version::parse(version_str).unwrap();
            let displayed = parsed.to_string();
            assert_eq!(displayed, version_str);
        }
    }

    #[test]
    fn test_prerelease_multiple_iteration_increments() {
        let v0 = Version::parse("v1.0.0-beta").unwrap();
        let pr = v0.prerelease.unwrap();

        // Increment from None to 1
        let pr1 = pr.increment_iteration();
        assert_eq!(pr1.to_string(), "beta.1");

        // Increment from 1 to 2
        let pr2 = pr1.increment_iteration();
        assert_eq!(pr2.to_string(), "beta.2");

        // Increment from 2 to 3
        let pr3 = pr2.increment_iteration();
        assert_eq!(pr3.to_string(), "beta.3");
    }
}
