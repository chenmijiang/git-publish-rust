#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VersionBump {
    Major,
    Minor,
    Patch,
}

impl Version {
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
