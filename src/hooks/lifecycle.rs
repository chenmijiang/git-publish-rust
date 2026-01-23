use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of hooks available in git-publish workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookType {
    PreTagCreate,
    PostTagCreate,
    PostPush,
}

impl HookType {
    /// Get the hook name as a string
    pub fn name(&self) -> &'static str {
        match self {
            HookType::PreTagCreate => "pre-tag-create",
            HookType::PostTagCreate => "post-tag-create",
            HookType::PostPush => "post-push",
        }
    }
}

/// Context information passed to a hook
#[derive(Debug, Clone)]
pub struct HookContext {
    /// Type of hook being executed
    pub hook_type: HookType,
    /// Branch name that was tagged
    pub branch: String,
    /// Tag name being created or pushed
    pub tag: String,
    /// Remote repository name
    pub remote: String,
    /// Version bump type (Major, Minor, Patch) if applicable
    pub version_bump: Option<String>,
    /// Number of commits since last tag if applicable
    pub commit_count: Option<usize>,
}

impl HookContext {
    /// Convert context to environment variables for the hook script
    ///
    /// Maps context fields to GITPUBLISH_* environment variables
    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        env.insert("GITPUBLISH_BRANCH".to_string(), self.branch.clone());
        env.insert("GITPUBLISH_TAG_NAME".to_string(), self.tag.clone());
        env.insert("GITPUBLISH_REMOTE".to_string(), self.remote.clone());

        if let Some(ref bump) = self.version_bump {
            env.insert("GITPUBLISH_VERSION_BUMP".to_string(), bump.clone());
        }

        if let Some(count) = self.commit_count {
            env.insert("GITPUBLISH_COMMIT_COUNT".to_string(), count.to_string());
        }

        env
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_type_pre_tag_create() {
        assert_eq!(HookType::PreTagCreate.name(), "pre-tag-create");
    }

    #[test]
    fn test_hook_type_post_tag_create() {
        assert_eq!(HookType::PostTagCreate.name(), "post-tag-create");
    }

    #[test]
    fn test_hook_type_post_push() {
        assert_eq!(HookType::PostPush.name(), "post-push");
    }

    #[test]
    fn test_hook_context_to_env_vars_all_fields() {
        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "main".to_string(),
            tag: "v1.2.3".to_string(),
            remote: "origin".to_string(),
            version_bump: Some("Minor".to_string()),
            commit_count: Some(5),
        };

        let env = ctx.to_env_vars();
        assert_eq!(env.get("GITPUBLISH_BRANCH"), Some(&"main".to_string()));
        assert_eq!(env.get("GITPUBLISH_TAG_NAME"), Some(&"v1.2.3".to_string()));
        assert_eq!(env.get("GITPUBLISH_REMOTE"), Some(&"origin".to_string()));
        assert_eq!(
            env.get("GITPUBLISH_VERSION_BUMP"),
            Some(&"Minor".to_string())
        );
        assert_eq!(env.get("GITPUBLISH_COMMIT_COUNT"), Some(&"5".to_string()));
    }

    #[test]
    fn test_hook_context_to_env_vars_minimal() {
        let ctx = HookContext {
            hook_type: HookType::PostPush,
            branch: "develop".to_string(),
            tag: "v2.0.0".to_string(),
            remote: "upstream".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let env = ctx.to_env_vars();
        assert_eq!(env.len(), 3);
        assert_eq!(env.get("GITPUBLISH_BRANCH"), Some(&"develop".to_string()));
        assert_eq!(env.get("GITPUBLISH_TAG_NAME"), Some(&"v2.0.0".to_string()));
        assert_eq!(env.get("GITPUBLISH_REMOTE"), Some(&"upstream".to_string()));
        assert!(env.get("GITPUBLISH_VERSION_BUMP").is_none());
        assert!(env.get("GITPUBLISH_COMMIT_COUNT").is_none());
    }
}
