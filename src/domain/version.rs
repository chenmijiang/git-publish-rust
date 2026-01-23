use crate::error::{GitPublishError, Result};
use std::fmt;

/// Semantic version representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version {
            major,
            minor,
            patch,
        }
    }

    /// Parse version from a tag string (e.g., "v1.2.3" -> Version(1,2,3))
    pub fn parse(tag: &str) -> Result<Self> {
        // Remove 'v' or 'V' prefix
        let clean_tag = tag.trim_start_matches('v').trim_start_matches('V');

        // Split by '.' and parse
        let parts: Vec<&str> = clean_tag.split('.').collect();
        if parts.len() != 3 {
            return Err(GitPublishError::version(format!(
                "Invalid version format: '{}' - expected X.Y.Z",
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

        Ok(Version {
            major,
            minor,
            patch,
        })
    }

    /// Bump version according to bump type
    pub fn bump(&self, bump_type: &VersionBump) -> Self {
        match bump_type {
            VersionBump::Major => Version {
                major: self.major + 1,
                minor: 0,
                patch: 0,
            },
            VersionBump::Minor => Version {
                major: self.major,
                minor: self.minor + 1,
                patch: 0,
            },
            VersionBump::Patch => Version {
                major: self.major,
                minor: self.minor,
                patch: self.patch + 1,
            },
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
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

    #[test]
    fn test_version_parse() {
        let v = Version::parse("v1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
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

    #[test]
    fn test_version_bump_major() {
        let v = Version::new(1, 2, 3);
        let bumped = v.bump(&VersionBump::Major);
        assert_eq!(bumped, Version::new(2, 0, 0));
    }

    #[test]
    fn test_version_bump_minor() {
        let v = Version::new(1, 2, 3);
        let bumped = v.bump(&VersionBump::Minor);
        assert_eq!(bumped, Version::new(1, 3, 0));
    }

    #[test]
    fn test_version_bump_patch() {
        let v = Version::new(1, 2, 3);
        let bumped = v.bump(&VersionBump::Patch);
        assert_eq!(bumped, Version::new(1, 2, 4));
    }

    #[test]
    fn test_version_display() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }
}
