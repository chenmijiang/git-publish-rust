use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub branches: HashMap<String, String>,

    #[serde(default)]
    pub conventional_commits: ConventionalCommitsConfig,

    #[serde(default)]
    pub patterns: PatternsConfig,
}

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

fn default_breaking_change_indicators() -> Vec<String> {
    vec![
        "BREAKING CHANGE:".to_string(),
        "BREAKING-CHANGE:".to_string(),
    ]
}

fn default_major_keywords() -> Vec<String> {
    vec!["breaking".to_string(), "deprecate".to_string()]
}

fn default_minor_keywords() -> Vec<String> {
    vec![
        "feature".to_string(),
        "feat".to_string(),
        "enhancement".to_string(),
    ]
}

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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PatternsConfig {
    #[serde(default = "default_version_format")]
    pub version_format: HashMap<String, String>,
}

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
        }
    }
}

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
