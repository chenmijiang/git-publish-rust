use std::fmt;

/// Warnings that occur when processing git tags near repository boundaries.
/// These are non-fatal issues that should be reported to the user.
#[derive(Debug, Clone, PartialEq)]
pub enum BoundaryWarning {
    /// No new commits since the latest tag
    NoNewCommits {
        latest_tag: String,
        current_commit_hash: String,
    },
    /// Tag exists but cannot be parsed as a semantic version
    UnparsableTag { tag: String, reason: String },
    /// Tag exists but doesn't match the configured pattern
    #[allow(dead_code)]
    TagMismatchPattern { tag: String, pattern: String },
    /// Fetch operation failed due to authentication issues
    FetchAuthenticationFailed { remote: String },
}

impl fmt::Display for BoundaryWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoundaryWarning::NoNewCommits {
                latest_tag,
                current_commit_hash,
            } => {
                let short_hash = if current_commit_hash.len() > 7 {
                    &current_commit_hash[..7]
                } else {
                    current_commit_hash.as_str()
                };
                write!(
                    f,
                    "No new commits since tag '{}' (current: {})",
                    latest_tag, short_hash
                )
            }
            BoundaryWarning::UnparsableTag { tag, reason } => {
                write!(f, "Cannot parse tag '{}': {}", tag, reason)
            }
            BoundaryWarning::TagMismatchPattern { tag, pattern } => {
                write!(f, "Tag '{}' does not match pattern '{}'", tag, pattern)
            }
            BoundaryWarning::FetchAuthenticationFailed { remote } => {
                write!(
                    f,
                    "Authentication failed when fetching from remote '{}'",
                    remote
                )
            }
        }
    }
}
