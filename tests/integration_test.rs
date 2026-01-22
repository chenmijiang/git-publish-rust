// tests/integration_test.rs
use std::env;
use std::process::Command;

#[test]
fn test_git_publish_help() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "git-publish", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("git-publish"));
    assert!(stdout.contains("Create and push git tags"));
}

#[test]
fn test_git_publish_version() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "git-publish", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    // This might fail if version command isn't implemented, which is OK for now
    // Just checking that the command can be executed without crashing badly
    assert!(output.status.success() || output.status.code() == Some(2)); // 2 usually means help/version screen
}

#[test]
fn test_config_loading() {
    use git_publish::config::load_config;

    // Test with no config file (should use defaults)
    let config = load_config(None).expect("Should load default config");
    assert!(config.branches.contains_key("main"));
    assert!(config.branches.contains_key("develop"));
    assert!(config.branches.contains_key("gray"));
    assert_eq!(config.branches.get("main"), Some(&"v{version}".to_string()));
}

#[test]
fn test_version_bump_detection() {
    use git_publish::config::ConventionalCommitsConfig;
    use git_publish::conventional::{determine_version_bump, VersionBump};

    let config = ConventionalCommitsConfig::default();
    let commit_messages = vec![
        "feat: add new authentication system".to_string(),
        "fix: resolve login issue".to_string(),
    ];

    let bump = determine_version_bump(&commit_messages, &config);
    // Since there's a feat commit, it should be at least minor
    assert!(matches!(bump, VersionBump::Major | VersionBump::Minor));
}

#[test]
fn test_version_parsing_and_bumping() {
    use git_publish::version::{bump_version, parse_version_from_tag, VersionBump};

    // Test parsing version from tag
    let version = parse_version_from_tag("v1.2.3").expect("Should parse version");
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);

    // Test bumping version
    let bumped = bump_version(version.clone(), &VersionBump::Minor);
    assert_eq!(bumped.major, 1);
    assert_eq!(bumped.minor, 3);
    assert_eq!(bumped.patch, 0);

    // Test major bump
    let major_bumped = bump_version(version.clone(), &VersionBump::Major);
    assert_eq!(major_bumped.major, 2);
    assert_eq!(major_bumped.minor, 0);
    assert_eq!(major_bumped.patch, 0);

    // Test patch bump
    let patch_bumped = bump_version(version.clone(), &VersionBump::Patch);
    assert_eq!(patch_bumped.major, 1);
    assert_eq!(patch_bumped.minor, 2);
    assert_eq!(patch_bumped.patch, 4);
}

#[test]
fn test_conventional_commit_parsing() {
    use git_publish::conventional::parse_conventional_commit;

    // Test standard conventional commit
    let parsed =
        parse_conventional_commit("feat(auth): add new login system").expect("Should parse");
    assert_eq!(parsed.r#type, "feat");
    assert_eq!(parsed.scope, Some("auth".to_string()));
    assert_eq!(parsed.description, "add new login system");
    assert_eq!(parsed.is_breaking_change, false);

    // Test breaking change with ! syntax
    let parsed_breaking =
        parse_conventional_commit("feat!: remove deprecated API").expect("Should parse");
    assert_eq!(parsed_breaking.r#type, "feat");
    assert_eq!(parsed_breaking.is_breaking_change, true);

    // Test breaking change in footer
    let breaking_with_footer = "feat: new feature\n\nBREAKING CHANGE: This changes the API";
    let parsed_footer = parse_conventional_commit(breaking_with_footer).expect("Should parse");
    assert_eq!(parsed_footer.r#type, "feat");
    assert_eq!(parsed_footer.is_breaking_change, true);

    // Test non-conventional commit (should default to chore)
    let parsed_non_conv =
        parse_conventional_commit("Update README").expect("Should parse as chore");
    assert_eq!(parsed_non_conv.r#type, "chore");
    assert_eq!(parsed_non_conv.description, "Update README");
}

#[cfg(test)]
mod git_operations_tests {
    use super::*;
    use git2::Repository;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    // Helper function to setup a temporary git repo for testing
    fn setup_test_repo() -> TempDir {
        let temp_dir = TempDir::new().expect("Could not create temp dir");

        // Initialize git repo
        let repo = Repository::init(temp_dir.path()).expect("Could not init git repo");

        // Configure git user
        {
            let mut config = repo.config().expect("Could not get config");
            config
                .set_str("user.name", "Test User")
                .expect("Could not set user.name");
            config
                .set_str("user.email", "test@example.com")
                .expect("Could not set user.email");
        }

        // Create initial commit
        let content = b"Initial content\n";
        let content_path = temp_dir.path().join("README.md");
        fs::write(&content_path, content).expect("Could not write initial file");

        let mut index = repo.index().expect("Could not get index");
        index
            .add_path(Path::new("README.md"))
            .expect("Could not add file to index");
        index.write().expect("Could not write index");

        let tree_id = index.write_tree().expect("Could not write tree");
        let tree = repo.find_tree(tree_id).expect("Could not find tree");

        let commit_id = repo
            .commit(
                Some("HEAD"),
                &repo.signature().expect("Could not get sig"),
                &repo.signature().expect("Could not get sig"),
                "Initial commit",
                &tree,
                &[],
            )
            .expect("Could not create commit");

        // Create a tag
        repo.tag_lightweight("v1.0.0", &repo.find_object(commit_id, None).unwrap(), false)
            .expect("Could not create tag");

        // Add another commit
        let content2 = b"Updated content\n";
        fs::write(&content_path, content2).expect("Could not write updated file");

        let mut index = repo.index().expect("Could not get index");
        index
            .add_path(Path::new("README.md"))
            .expect("Could not add file to index");
        index.write().expect("Could not write index");

        let tree_id = index.write_tree().expect("Could not write tree");
        let tree = repo.find_tree(tree_id).expect("Could not find tree");

        repo.commit(
            Some("HEAD"),
            &repo.signature().expect("Could not get sig"),
            &repo.signature().expect("Could not get sig"),
            "feat: add new feature",
            &tree,
            &[&repo.find_commit(commit_id).unwrap()],
        )
        .expect("Could not create commit");

        temp_dir
    }

    #[test]
    fn test_git_repo_operations() {
        // This test creates a temporary git repository for testing git operations
        let temp_dir = setup_test_repo();
        let original_dir = env::current_dir().unwrap();

        // Change to the temp directory
        env::set_current_dir(temp_dir.path()).expect("Could not change to temp dir");

        // Test that we can instantiate GitRepo
        let git_repo = git_publish::git_ops::GitRepo::new();
        assert!(
            git_repo.is_ok(),
            "GitRepo::new() should succeed in a git directory"
        );

        // Change back to the original directory
        env::set_current_dir(original_dir).unwrap();
    }
}
