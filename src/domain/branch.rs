/// Represents a git branch with context
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchContext {
    pub name: String,
    pub is_main: bool,
}

impl BranchContext {
    /// Create a new branch context
    pub fn new(name: impl Into<String>) -> Self {
        let name_str = name.into();
        let is_main = matches!(name_str.as_str(), "main" | "master");

        BranchContext {
            name: name_str,
            is_main,
        }
    }

    /// Check if this is a release branch (main/master)
    pub fn is_release_branch(&self) -> bool {
        self.is_main
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_branch() {
        let branch = BranchContext::new("main");
        assert!(branch.is_main);
        assert!(branch.is_release_branch());
    }

    #[test]
    fn test_master_branch() {
        let branch = BranchContext::new("master");
        assert!(branch.is_main);
    }

    #[test]
    fn test_develop_branch() {
        let branch = BranchContext::new("develop");
        assert!(!branch.is_main);
        assert!(!branch.is_release_branch());
    }
}
