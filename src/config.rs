use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Configuration for git hooks
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct HooksConfig {
    /// Path to pre-tag-create hook script
    #[serde(default)]
    pub pre_tag_create: Option<String>,

    /// Path to post-tag-create hook script
    #[serde(default)]
    pub post_tag_create: Option<String>,

    /// Path to post-push hook script
    #[serde(default)]
    pub post_push: Option<String>,
}

/// Represents the complete configuration for git-publish.
///
/// Contains branch mappings, conventional commit settings, version formatting patterns, and behavior options.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub branches: HashMap<String, String>,

    #[serde(default)]
    pub conventional_commits: ConventionalCommitsConfig,

    #[serde(default)]
    pub patterns: PatternsConfig,

    #[serde(default)]
    pub behavior: BehaviorConfig,

    #[serde(default)]
    pub prerelease: PreReleaseConfig,

    #[serde(default)]
    pub hooks: HooksConfig,
}

/// Returns the default list of conventional commit types.
fn default_commit_types() -> Vec<String> {
    vec![
        "feat".to_string(),
        "fix".to_string(),
        "docs".to_string(),
        "style".to_string(),
        "refactor".to_string(),
        "test".to_string(),
        "chore".to_string(),
        "build".to_string(),
        "ci".to_string(),
        "perf".to_string(),
    ]
}

/// Returns the default list of breaking change indicators.
fn default_breaking_change_indicators() -> Vec<String> {
    vec![
        "BREAKING CHANGE:".to_string(),
        "BREAKING-CHANGE:".to_string(),
    ]
}

/// Returns the default list of keywords that trigger major version bumps.
fn default_major_keywords() -> Vec<String> {
    vec!["breaking".to_string(), "deprecate".to_string()]
}

/// Returns the default list of keywords that trigger minor version bumps.
fn default_minor_keywords() -> Vec<String> {
    vec![
        "feature".to_string(),
        "feat".to_string(),
        "enhancement".to_string(),
    ]
}

/// Configuration for conventional commit analysis.
///
/// Defines the types, breaking change indicators, and keywords used to analyze commits
/// and determine version bumping strategy.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConventionalCommitsConfig {
    #[serde(default = "default_commit_types")]
    pub types: Vec<String>,

    #[serde(default = "default_breaking_change_indicators")]
    pub breaking_change_indicators: Vec<String>,

    #[serde(default = "default_major_keywords")]
    pub major_keywords: Vec<String>,

    #[serde(default = "default_minor_keywords")]
    pub minor_keywords: Vec<String>,
}

impl Default for ConventionalCommitsConfig {
    fn default() -> Self {
        ConventionalCommitsConfig {
            types: default_commit_types(),
            breaking_change_indicators: default_breaking_change_indicators(),
            major_keywords: default_major_keywords(),
            minor_keywords: default_minor_keywords(),
        }
    }
}

/// Configuration for version formatting patterns.
///
/// Allows customization of how versions are formatted for different bump types.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PatternsConfig {
    #[serde(default = "default_version_format")]
    pub version_format: HashMap<String, String>,
}

/// Returns the default version format patterns.
fn default_version_format() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("major".to_string(), "{major}.{minor}.{patch}".to_string());
    map.insert("minor".to_string(), "{major}.{minor}.{patch}".to_string());
    map.insert("patch".to_string(), "{major}.{minor}.{patch}".to_string());
    map
}

impl Default for PatternsConfig {
    fn default() -> Self {
        PatternsConfig {
            version_format: default_version_format(),
        }
    }
}

/// Configuration for behavior customization.
///
/// Controls runtime behavior of git-publish without affecting version analysis.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct BehaviorConfig {
    #[serde(default)]
    pub skip_remote_selection: bool,
}

/// Configuration for pre-release version handling.
///
/// Controls how pre-release versions (alpha, beta, rc, custom) are managed.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PreReleaseConfig {
    /// Enable pre-release version support
    #[serde(default)]
    pub enabled: bool,

    /// Default pre-release identifier ("alpha", "beta", "rc", or custom)
    #[serde(default = "default_prerelease_identifier")]
    pub default_identifier: String,

    /// Auto-increment iteration number
    #[serde(default = "default_prerelease_auto_increment")]
    pub auto_increment: bool,
}

/// Returns the default pre-release identifier
fn default_prerelease_identifier() -> String {
    "alpha".to_string()
}

/// Returns the default auto-increment setting
fn default_prerelease_auto_increment() -> bool {
    true
}

impl Default for PreReleaseConfig {
    fn default() -> Self {
        PreReleaseConfig {
            enabled: false,
            default_identifier: default_prerelease_identifier(),
            auto_increment: default_prerelease_auto_increment(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut branches = HashMap::new();
        branches.insert("main".to_string(), "v{version}".to_string());
        branches.insert("develop".to_string(), "d{version}".to_string());
        branches.insert("gray".to_string(), "g{version}".to_string());

        Config {
            branches,
            conventional_commits: ConventionalCommitsConfig::default(),
            patterns: PatternsConfig::default(),
            behavior: BehaviorConfig::default(),
            prerelease: PreReleaseConfig::default(),
            hooks: HooksConfig::default(),
        }
    }
}

/// Loads configuration from file or returns defaults.
///
/// Attempts to load configuration in the following order:
/// 1. Custom path provided as parameter
/// 2. `gitpublish.toml` in current directory
/// 3. `~/.config/.gitpublish.toml` in user config directory
/// 4. Default configuration if no file found
///
/// # Arguments
/// * `config_path` - Optional path to custom configuration file
///
/// # Returns
/// * `Ok(Config)` - Loaded or default configuration
/// * `Err` - If file exists but cannot be read or parsed
pub fn load_config(config_path: Option<&str>) -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = if let Some(path) = config_path {
        fs::read_to_string(path)?
    } else if Path::new("./gitpublish.toml").exists() {
        fs::read_to_string("./gitpublish.toml")?
    } else if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join(".gitpublish.toml");
        if config_path.exists() {
            fs::read_to_string(config_path)?
        } else {
            return Ok(Config::default());
        }
    } else {
        return Ok(Config::default());
    };

    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests: configuration scenarios
    #[test]
    fn test_config_default() {
        let config = Config::default();

        assert!(config.branches.contains_key("main"));
        assert!(config.branches.contains_key("develop"));
        assert_eq!(config.branches.get("main"), Some(&"v{version}".to_string()));
    }

    #[test]
    fn test_config_conventional_commits_defaults() {
        let config = ConventionalCommitsConfig::default();

        assert!(config.types.contains(&"feat".to_string()));
        assert!(config.types.contains(&"fix".to_string()));
        assert!(config.types.contains(&"docs".to_string()));
        assert_eq!(config.types.len(), 10);
    }

    #[test]
    fn test_config_breaking_change_indicators() {
        let config = ConventionalCommitsConfig::default();

        assert!(config
            .breaking_change_indicators
            .contains(&"BREAKING CHANGE:".to_string()));
        assert!(config
            .breaking_change_indicators
            .contains(&"BREAKING-CHANGE:".to_string()));
    }

    #[test]
    fn test_config_major_keywords() {
        let config = ConventionalCommitsConfig::default();

        assert!(config.major_keywords.contains(&"breaking".to_string()));
        assert!(config.major_keywords.contains(&"deprecate".to_string()));
    }

    #[test]
    fn test_config_minor_keywords() {
        let config = ConventionalCommitsConfig::default();

        assert!(config.minor_keywords.contains(&"feat".to_string()));
        assert!(config.minor_keywords.contains(&"feature".to_string()));
        assert!(config.minor_keywords.contains(&"enhancement".to_string()));
    }

    #[test]
    fn test_config_patterns_default() {
        let config = PatternsConfig::default();

        assert!(config.version_format.contains_key("major"));
        assert!(config.version_format.contains_key("minor"));
        assert!(config.version_format.contains_key("patch"));
    }

    #[test]
    fn test_config_behavior_default() {
        let config = BehaviorConfig::default();

        assert!(!config.skip_remote_selection);
    }

    #[test]
    fn test_config_prerelease_default_disabled() {
        let config = PreReleaseConfig::default();

        assert!(!config.enabled);
        assert_eq!(config.default_identifier, "alpha");
        assert!(config.auto_increment);
    }

    #[test]
    fn test_config_hooks_default_empty() {
        let config = HooksConfig::default();

        assert!(config.pre_tag_create.is_none());
        assert!(config.post_tag_create.is_none());
        assert!(config.post_push.is_none());
    }

    #[test]
    fn test_config_full_default_structure() {
        let config = Config::default();

        // Verify all sections exist and are initialized
        assert!(!config.branches.is_empty());
        assert!(!config.conventional_commits.types.is_empty());
        assert!(!config.patterns.version_format.is_empty());
        assert!(!config.prerelease.enabled); // disabled by default
        assert!(config.hooks.pre_tag_create.is_none());
    }

    #[test]
    fn test_config_toml_parsing_simple() {
        let toml_str = r#"
[branches]
main = "v{version}"
develop = "d{version}"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();

        assert_eq!(config.branches.get("main"), Some(&"v{version}".to_string()));
        assert_eq!(
            config.branches.get("develop"),
            Some(&"d{version}".to_string())
        );
    }

    #[test]
    fn test_config_toml_parsing_with_prerelease() {
        let toml_str = r#"
[prerelease]
enabled = true
default_identifier = "beta"
auto_increment = true
"#;
        let config: Config = toml::from_str(toml_str).unwrap();

        assert!(config.prerelease.enabled);
        assert_eq!(config.prerelease.default_identifier, "beta");
        assert!(config.prerelease.auto_increment);
    }

    #[test]
    fn test_config_toml_parsing_with_hooks() {
        let toml_str = r#"
[hooks]
pre_tag_create = "./scripts/pre-tag-create.sh"
post_tag_create = "./scripts/post-tag-create.sh"
post_push = "./scripts/post-push.sh"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();

        assert_eq!(
            config.hooks.pre_tag_create,
            Some("./scripts/pre-tag-create.sh".to_string())
        );
        assert_eq!(
            config.hooks.post_tag_create,
            Some("./scripts/post-tag-create.sh".to_string())
        );
        assert_eq!(
            config.hooks.post_push,
            Some("./scripts/post-push.sh".to_string())
        );
    }

    #[test]
    fn test_config_toml_parsing_complete() {
        let toml_str = r#"
[branches]
main = "v{version}"
develop = "d{version}"
staging = "s{version}"

[conventional_commits]
types = ["feat", "fix", "docs"]

[behavior]
skip_remote_selection = true

[prerelease]
enabled = true
default_identifier = "rc"
auto_increment = false

[hooks]
pre_tag_create = "./hooks/pre.sh"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();

        // Verify branches
        assert_eq!(config.branches.len(), 3);
        assert_eq!(
            config.branches.get("staging"),
            Some(&"s{version}".to_string())
        );

        // Verify conventional commits
        assert_eq!(config.conventional_commits.types.len(), 3);

        // Verify behavior
        assert!(config.behavior.skip_remote_selection);

        // Verify prerelease
        assert!(config.prerelease.enabled);
        assert_eq!(config.prerelease.default_identifier, "rc");
        assert!(!config.prerelease.auto_increment);

        // Verify hooks
        assert_eq!(
            config.hooks.pre_tag_create,
            Some("./hooks/pre.sh".to_string())
        );
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let original = Config::default();

        // Serialize to TOML
        let toml_str = toml::to_string(&original).unwrap();

        // Deserialize back
        let restored: Config = toml::from_str(&toml_str).unwrap();

        // Verify key values match
        assert_eq!(original.branches, restored.branches);
        assert_eq!(original.prerelease.enabled, restored.prerelease.enabled);
    }

    #[test]
    fn test_conventional_commits_config_clone() {
        let config1 = ConventionalCommitsConfig::default();
        let config2 = config1.clone();

        assert_eq!(config1.types, config2.types);
        assert_eq!(
            config1.breaking_change_indicators,
            config2.breaking_change_indicators
        );
    }

    #[test]
    fn test_prerelease_config_equality() {
        let pr1 = PreReleaseConfig {
            enabled: true,
            default_identifier: "beta".to_string(),
            auto_increment: true,
        };

        let pr2 = PreReleaseConfig {
            enabled: true,
            default_identifier: "beta".to_string(),
            auto_increment: true,
        };

        assert_eq!(pr1, pr2);
    }

    #[test]
    fn test_config_multiple_branch_patterns() {
        let toml_str = r#"
[branches]
main = "v{version}"
develop = "d{version}"
staging = "staging-{version}"
release = "release/{version}"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();

        assert_eq!(config.branches.len(), 4);
        assert_eq!(
            config.branches.get("release"),
            Some(&"release/{version}".to_string())
        );
    }
}
