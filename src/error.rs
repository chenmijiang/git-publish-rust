use thiserror::Error;

/// Unified error type for git-publish operations
#[derive(Error, Debug)]
pub enum GitPublishError {
    #[error("Git operation failed: {0}")]
    Git(#[from] git2::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Version parsing error: {0}")]
    Version(String),

    #[error("Tag error: {0}")]
    Tag(String),

    #[error("Remote operation failed: {0}")]
    Remote(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience type alias for Results in git-publish
pub type Result<T> = std::result::Result<T, GitPublishError>;

impl GitPublishError {
    /// Create a configuration error with context
    pub fn config(msg: impl Into<String>) -> Self {
        GitPublishError::Config(msg.into())
    }

    /// Create a version error with context
    pub fn version(msg: impl Into<String>) -> Self {
        GitPublishError::Version(msg.into())
    }

    /// Create a tag error with context
    pub fn tag(msg: impl Into<String>) -> Self {
        GitPublishError::Tag(msg.into())
    }

    /// Create a remote error with context
    pub fn remote(msg: impl Into<String>) -> Self {
        GitPublishError::Remote(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = GitPublishError::config("test config issue");
        assert_eq!(err.to_string(), "Configuration error: test config issue");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: GitPublishError = io_err.into();
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn test_error_constructors() {
        assert!(GitPublishError::version("test")
            .to_string()
            .contains("Version"));
        assert!(GitPublishError::tag("test").to_string().contains("Tag"));
    }

    // Integration tests: edge cases and error scenarios
    #[test]
    fn test_error_all_variants() {
        let errors = vec![
            GitPublishError::config("config issue"),
            GitPublishError::version("version issue"),
            GitPublishError::tag("tag issue"),
            GitPublishError::remote("remote issue"),
        ];

        for err in errors {
            let msg = err.to_string();
            assert!(!msg.is_empty());
        }
    }

    #[test]
    fn test_error_empty_messages() {
        let errors = vec![
            GitPublishError::config(""),
            GitPublishError::version(""),
            GitPublishError::tag(""),
        ];

        for err in errors {
            let msg = err.to_string();
            // Even with empty message, the error type prefix should be present
            assert!(!msg.is_empty());
        }
    }

    #[test]
    fn test_error_long_messages() {
        let long_msg = "a".repeat(1000);
        let err = GitPublishError::version(&long_msg);
        let msg = err.to_string();
        assert!(msg.contains(&long_msg));
    }

    #[test]
    fn test_error_special_characters_in_messages() {
        let special_chars = vec![
            "message with\nnewline",
            "message with\ttab",
            "message with 'quotes'",
            "message with \"double quotes\"",
            "message with \\ backslash",
            "message with émojis 🚀",
            "message with unicode: ñ",
        ];

        for msg in special_chars {
            let err = GitPublishError::version(msg);
            let err_msg = err.to_string();
            assert!(err_msg.contains("Version"));
        }
    }

    #[test]
    fn test_io_error_conversion() {
        let io_errors = vec![
            std::io::Error::new(std::io::ErrorKind::NotFound, "Not found"),
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied"),
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data"),
        ];

        for io_err in io_errors {
            let err: GitPublishError = io_err.into();
            let msg = err.to_string();
            assert!(msg.contains("I/O error"));
        }
    }

    #[test]
    fn test_error_messages_are_descriptive() {
        let error_pairs = vec![
            (GitPublishError::config("x"), "Configuration error"),
            (GitPublishError::version("x"), "Version parsing error"),
            (GitPublishError::tag("x"), "Tag error"),
            (GitPublishError::remote("x"), "Remote operation failed"),
        ];

        for (err, expected_prefix) in error_pairs {
            let msg = err.to_string();
            assert!(
                msg.starts_with(expected_prefix),
                "Error message should start with '{}', but got '{}'",
                expected_prefix,
                msg
            );
        }
    }

    #[test]
    fn test_error_into_string() {
        let err = GitPublishError::tag("test tag issue");
        let s: String = err.to_string();
        assert!(!s.is_empty());
        assert!(s.contains("tag"));
    }

    #[test]
    fn test_multiple_error_creations_same_type() {
        for i in 0..10 {
            let err = GitPublishError::version(&format!("error {}", i));
            let msg = err.to_string();
            assert!(msg.contains(&format!("error {}", i)));
        }
    }
}
