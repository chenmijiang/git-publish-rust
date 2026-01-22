use crate::config;
pub use crate::version::VersionBump;
use regex;

#[derive(Debug, PartialEq)]
pub struct ParsedCommit {
    pub r#type: String,
    pub scope: Option<String>,
    pub description: String,
    pub is_breaking_change: bool,
}

pub fn parse_conventional_commit(message: &str) -> Option<ParsedCommit> {
    // Case 1: type(scope)!: description or type(scope): description
    if let Ok(re) = regex::Regex::new(r"^([a-z]+)\(([^)]+)\)(!?):\s*(.*)") {
        if let Some(captures) = re.captures(message) {
            if captures.len() >= 5 {
                if let (Some(type_match), Some(scope_match), Some(description_match)) =
                    (captures.get(1), captures.get(2), captures.get(4))
                {
                    let commit_type = type_match.as_str().to_string();
                    let scope = Some(scope_match.as_str().to_string());
                    let exclamation_marker = captures.get(3).map(|m| m.as_str()).unwrap_or("");
                    let description = description_match.as_str().to_string();

                    let is_breaking_change =
                        exclamation_marker == "!" || message.contains("BREAKING CHANGE:");

                    return Some(ParsedCommit {
                        r#type: commit_type,
                        scope,
                        description,
                        is_breaking_change,
                    });
                }
            }
        }
    }

    // Case 2: type!: description (breaking change without scope)
    if let Ok(re) = regex::Regex::new(r"^([a-z]+)!:\s*(.*)") {
        if let Some(captures) = re.captures(message) {
            if captures.len() >= 3 {
                if let (Some(type_match), Some(description_match)) =
                    (captures.get(1), captures.get(2))
                {
                    let commit_type = type_match.as_str().to_string();
                    let description = description_match.as_str().to_string();

                    return Some(ParsedCommit {
                        r#type: commit_type,
                        scope: None,
                        description,
                        is_breaking_change: true,
                    });
                }
            }
        }
    }

    // Case 3: type: description (no scope, non-breaking)
    if let Ok(re) = regex::Regex::new(r"^([a-z]+):\s*(.*)") {
        if let Some(captures) = re.captures(message) {
            if captures.len() >= 3 {
                if let (Some(type_match), Some(description_match)) =
                    (captures.get(1), captures.get(2))
                {
                    let commit_type = type_match.as_str().to_string();
                    let description = description_match.as_str().to_string();

                    // Check for breaking changes in the message body
                    let is_breaking_change = message.contains("BREAKING CHANGE:");

                    return Some(ParsedCommit {
                        r#type: commit_type,
                        scope: None,
                        description,
                        is_breaking_change,
                    });
                }
            }
        }
    }

    // Default to chore for non-conventional commits
    Some(ParsedCommit {
        r#type: "chore".to_string(),
        scope: None,
        description: message.to_string(),
        is_breaking_change: false,
    })
}

pub fn determine_version_bump(
    commit_messages: &[String],
    config: &config::ConventionalCommitsConfig,
) -> VersionBump {
    let mut has_breaking_changes = false;
    let mut has_features = false;
    let mut has_fixes = false;

    for message in commit_messages {
        let parsed_commit = parse_conventional_commit(message);

        if let Some(parsed) = parsed_commit {
            // Check for breaking changes
            if parsed.is_breaking_change {
                has_breaking_changes = true;
            }

            // Check for major version indicators
            for keyword in &config.major_keywords {
                if message.to_lowercase().contains(keyword) {
                    has_features = true;
                }
            }

            // Check for minor version indicators
            for keyword in &config.minor_keywords {
                if message.to_lowercase().contains(keyword) {
                    has_features = true;
                }
            }

            // Check for commit types that might indicate features or fixes
            match parsed.r#type.as_str() {
                "feat" | "feature" => has_features = true,
                "fix" | "perf" | "refactor" => has_fixes = true,
                _ => {}
            }
        }

        // If we found a breaking change, we can return early
        if has_breaking_changes {
            return VersionBump::Major;
        }
    }

    if has_features {
        VersionBump::Minor
    } else if has_fixes {
        VersionBump::Patch
    } else {
        // If no conventional commits detected, default to patch
        VersionBump::Patch
    }
}
