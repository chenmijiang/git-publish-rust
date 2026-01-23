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

    #[error("Hook execution failed: {0}")]
    Hook(String),

    #[error("Remote operation failed: {0}")]
    Remote(String),

    #[error("Branch error: {0}")]
    Branch(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
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

    /// Create a hook error with context
    pub fn hook(msg: impl Into<String>) -> Self {
        GitPublishError::Hook(msg.into())
    }

    /// Create a branch error with context
    pub fn branch(msg: impl Into<String>) -> Self {
        GitPublishError::Branch(msg.into())
    }

    /// Create a remote error with context
    pub fn remote(msg: impl Into<String>) -> Self {
        GitPublishError::Remote(msg.into())
    }

    /// Create an invalid argument error
    pub fn invalid_arg(msg: impl Into<String>) -> Self {
        GitPublishError::InvalidArgument(msg.into())
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
        assert!(GitPublishError::branch("test")
            .to_string()
            .contains("Branch"));
    }
}
