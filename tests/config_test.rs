// tests/config_test.rs
use git_publish::config::{load_config, Config};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_load_default_config() {
    let config = Config::default();
    assert_eq!(config.branches.get("main"), Some(&"v{version}".to_string()));
    assert_eq!(
        config.branches.get("develop"),
        Some(&"d{version}".to_string())
    );
    assert_eq!(config.branches.get("gray"), Some(&"g{version}".to_string()));
}

#[test]
fn test_load_from_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let toml_content = r#"
[branches]
"main" = "v{version}"
"develop" = "dev-{version}"

[conventional_commits]
types = ["feat", "fix", "chore"]
major_keywords = ["breaking"]
"#;
    temp_file.write_all(toml_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let config = load_config(Some(temp_file.path().to_str().unwrap())).unwrap();
    assert_eq!(config.branches.get("main"), Some(&"v{version}".to_string()));
    assert_eq!(
        config.branches.get("develop"),
        Some(&"dev-{version}".to_string())
    );
    assert!(config
        .conventional_commits
        .types
        .contains(&"feat".to_string()));
}

#[test]
fn test_default_values() {
    let config = Config::default();
    // Test that defaults are properly set in the Default implementation
    assert!(config
        .conventional_commits
        .types
        .contains(&"feat".to_string()));
    assert!(config
        .conventional_commits
        .types
        .contains(&"fix".to_string()));
    assert!(config
        .conventional_commits
        .breaking_change_indicators
        .contains(&"BREAKING CHANGE:".to_string()));
    assert!(config
        .conventional_commits
        .major_keywords
        .contains(&"breaking".to_string()));
    assert!(config
        .conventional_commits
        .minor_keywords
        .contains(&"feature".to_string()));
}

#[test]
fn test_behavior_config_defaults() {
    let config = Config::default();
    assert_eq!(config.behavior.skip_remote_selection, false);
}

#[test]
fn test_behavior_config_skip_remote_selection_from_file() {
    let config = load_config(Some("tests/fixtures/config_with_behavior.toml"))
        .expect("Failed to load test config");
    assert_eq!(config.behavior.skip_remote_selection, true);
}
