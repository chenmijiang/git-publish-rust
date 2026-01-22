/// Represents a semantic version with major, minor, and patch components.
///
/// Follows semantic versioning specification (major.minor.patch).
#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// Represents the type of semantic version bump to apply.
///
/// Used to determine how to increment version numbers based on commit analysis.
#[derive(Debug, Clone, PartialEq)]
pub enum VersionBump {
    Major,
    Minor,
    Patch,
}

impl Version {
    /// Creates a new Version with the specified major, minor, and patch components.
    ///
    /// # Arguments
    /// * `major` - Major version number
    /// * `minor` - Minor version number
    /// * `patch` - Patch version number
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version {
            major,
            minor,
            patch,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Parses a version from a git tag string.
///
/// Extracts version numbers from tag names by removing common prefixes ('v' or 'V')
/// and splitting by dots. Expects exactly three version components (major.minor.patch).
///
/// # Arguments
/// * `tag` - Tag string to parse (e.g., "v1.2.3" or "release-1.2.3")
///
/// # Returns
/// * `Some(Version)` - Successfully parsed version
/// * `None` - If tag doesn't match the pattern or has wrong number of components
///
/// # Example
/// ```ignore
/// assert_eq!(parse_version_from_tag("v1.2.3").unwrap(), Version::new(1, 2, 3));
/// assert_eq!(parse_version_from_tag("V0.1.0").unwrap(), Version::new(0, 1, 0));
/// assert_eq!(parse_version_from_tag("1.2"), None); // Too few components
/// ```
pub fn parse_version_from_tag(tag: &str) -> Option<Version> {
    // Remove common prefixes like 'v', 'V', etc.
    let clean_tag = tag.trim_start_matches('v').trim_start_matches('V');

    // Split by dots and try to parse numbers
    let parts: Vec<&str> = clean_tag.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;

    Some(Version::new(major, minor, patch))
}

/// Bumps a version according to the specified bump type.
///
/// Increments the appropriate version component and resets lower components to 0:
/// - **Major**: major += 1, minor = 0, patch = 0
/// - **Minor**: minor += 1, patch = 0
/// - **Patch**: patch += 1
///
/// # Arguments
/// * `version` - Current version to bump
/// * `bump_type` - Type of bump to apply
///
/// # Returns
/// New version with appropriate component incremented
///
/// # Example
/// ```ignore
/// let v = Version::new(1, 2, 3);
/// assert_eq!(bump_version(v, &VersionBump::Major), Version::new(2, 0, 0));
/// assert_eq!(bump_version(v, &VersionBump::Minor), Version::new(1, 3, 0));
/// assert_eq!(bump_version(v, &VersionBump::Patch), Version::new(1, 2, 4));
/// ```
pub fn bump_version(mut version: Version, bump_type: &VersionBump) -> Version {
    match bump_type {
        VersionBump::Major => {
            version.major += 1;
            version.minor = 0;
            version.patch = 0;
        }
        VersionBump::Minor => {
            version.minor += 1;
            version.patch = 0;
        }
        VersionBump::Patch => {
            version.patch += 1;
        }
    }
    version
}
