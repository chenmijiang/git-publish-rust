use git_publish::boundary::BoundaryWarning;
use git_publish::ui;

// ============================================================================
// BoundaryWarning Display Tests
// ============================================================================

#[test]
fn test_boundary_warning_no_new_commits_display() {
    let warning = BoundaryWarning::NoNewCommits {
        latest_tag: "v1.0.0".to_string(),
        current_commit_hash: "abc1234def5678".to_string(),
    };

    let display_msg = warning.to_string();
    assert!(
        display_msg.contains("No new commits"),
        "Message should contain 'No new commits', got: {}",
        display_msg
    );
    assert!(
        display_msg.contains("v1.0.0"),
        "Message should contain tag 'v1.0.0', got: {}",
        display_msg
    );
    assert!(
        display_msg.contains("abc1234"),
        "Message should contain shortened commit hash 'abc1234', got: {}",
        display_msg
    );
}

#[test]
fn test_boundary_warning_unparsable_tag_display() {
    let warning = BoundaryWarning::UnparsableTag {
        tag: "release-123".to_string(),
        reason: "Invalid format".to_string(),
    };

    let display_msg = warning.to_string();
    assert!(
        display_msg.contains("Cannot parse tag"),
        "Message should contain 'Cannot parse tag', got: {}",
        display_msg
    );
    assert!(
        display_msg.contains("release-123"),
        "Message should contain tag 'release-123', got: {}",
        display_msg
    );
    assert!(
        display_msg.contains("Invalid format"),
        "Message should contain reason 'Invalid format', got: {}",
        display_msg
    );
}

#[test]
fn test_boundary_warning_tag_mismatch_pattern_display() {
    let warning = BoundaryWarning::TagMismatchPattern {
        tag: "my-tag".to_string(),
        pattern: "v{version}".to_string(),
    };

    let display_msg = warning.to_string();
    assert!(
        display_msg.contains("does not match pattern"),
        "Message should contain 'does not match pattern', got: {}",
        display_msg
    );
    assert!(
        display_msg.contains("my-tag"),
        "Message should contain tag 'my-tag', got: {}",
        display_msg
    );
    assert!(
        display_msg.contains("v{version}"),
        "Message should contain pattern 'v{{version}}', got: {}",
        display_msg
    );
}

#[test]
fn test_boundary_warning_fetch_auth_failed_display() {
    let warning = BoundaryWarning::FetchAuthenticationFailed {
        remote: "origin".to_string(),
    };

    let display_msg = warning.to_string();
    assert!(
        display_msg.contains("Authentication failed"),
        "Message should contain 'Authentication failed', got: {}",
        display_msg
    );
    assert!(
        display_msg.contains("origin"),
        "Message should contain remote name 'origin', got: {}",
        display_msg
    );
}

// ============================================================================
// Tag Format Validation Tests
// ============================================================================

#[test]
fn test_validate_tag_format_valid_simple() {
    // Should pass: tag starts with "v" and matches "v{version}" pattern
    let result = ui::validate_tag_format("v1.2.3", "v{version}");
    assert!(
        result.is_ok(),
        "v1.2.3 should be valid for pattern v{{version}}, got: {:?}",
        result
    );
}

#[test]
fn test_validate_tag_format_valid_zero_version() {
    // Should pass: v0.0.1 is valid
    let result = ui::validate_tag_format("v0.0.1", "v{version}");
    assert!(
        result.is_ok(),
        "v0.0.1 should be valid for pattern v{{version}}, got: {:?}",
        result
    );
}

#[test]
fn test_validate_tag_format_invalid_missing_prefix() {
    // Should fail: tag doesn't have "v" prefix
    let result = ui::validate_tag_format("1.2.3", "v{version}");
    assert!(
        result.is_err(),
        "1.2.3 should be invalid for pattern v{{version}}, got: {:?}",
        result
    );
}

#[test]
fn test_validate_tag_format_invalid_wrong_prefix() {
    // Should fail: tag has wrong prefix
    let result = ui::validate_tag_format("release-1.2.3", "v{version}");
    assert!(
        result.is_err(),
        "release-1.2.3 should be invalid for pattern v{{version}}, got: {:?}",
        result
    );
}

#[test]
fn test_validate_tag_format_valid_with_suffix() {
    // Should pass: tag matches "v{version}-release" pattern
    let result = ui::validate_tag_format("v1.2.3-release", "v{version}-release");
    assert!(
        result.is_ok(),
        "v1.2.3-release should be valid for pattern v{{version}}-release, got: {:?}",
        result
    );
}

#[test]
fn test_validate_tag_format_invalid_missing_suffix() {
    // Should fail: tag missing required suffix
    let result = ui::validate_tag_format("v1.2.3", "v{version}-release");
    assert!(
        result.is_err(),
        "v1.2.3 should be invalid for pattern v{{version}}-release (missing -release), got: {:?}",
        result
    );
}

#[test]
fn test_validate_tag_format_no_version_constraint() {
    // Should pass: pattern without {version} means anything goes
    let result = ui::validate_tag_format("anything", "free-form");
    assert!(
        result.is_ok(),
        "Any tag should be valid when pattern has no {{version}}, got: {:?}",
        result
    );
}

#[test]
fn test_validate_tag_format_complex_suffix() {
    // Should pass: more complex pattern with version and custom suffix
    let result = ui::validate_tag_format("release-v1.2.3-final", "release-v{version}-final");
    assert!(
        result.is_ok(),
        "release-v1.2.3-final should be valid for pattern release-v{{version}}-final, got: {:?}",
        result
    );
}

// ============================================================================
// UI Interaction Tests
// ============================================================================

#[cfg(test)]
mod ui_interaction_tests {
    use git_publish::ui;

    #[test]
    fn test_validate_tag_format_patterns() {
        // Test various pattern matches
        assert!(ui::validate_tag_format("v1.0.0", "v{version}").is_ok());
        assert!(ui::validate_tag_format("release-v1.0.0", "release-v{version}").is_ok());
        assert!(ui::validate_tag_format("app-v1.0.0-rc1", "app-v{version}-rc1").is_ok());
    }

    #[test]
    fn test_validate_tag_format_invalid_patterns() {
        // Test mismatched patterns
        assert!(ui::validate_tag_format("1.0.0", "v{version}").is_err());
        assert!(ui::validate_tag_format("v1.0.0", "release-v{version}").is_err());
        assert!(ui::validate_tag_format("app-v1.0.0", "app-v{version}-rc1").is_err());
    }

    #[test]
    fn test_validate_tag_format_no_version_placeholder() {
        // Pattern without {version} placeholder should accept any tag
        assert!(ui::validate_tag_format("anything-123", "free-form").is_ok());
        assert!(ui::validate_tag_format("custom", "custom-tag-pattern").is_ok());
    }

    #[test]
    fn test_validate_tag_format_empty_strings() {
        // Test empty string handling
        assert!(ui::validate_tag_format("", "v{version}").is_err());
        assert!(ui::validate_tag_format("v1.0.0", "").is_ok()); // pattern empty means no constraint
    }
}
