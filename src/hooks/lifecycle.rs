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

    // Integration tests: hook workflows
    #[test]
    fn test_hook_context_all_hook_types() {
        let branches = vec!["main", "develop", "feature"];
        let hook_types = vec![
            HookType::PreTagCreate,
            HookType::PostTagCreate,
            HookType::PostPush,
        ];

        for hook_type in hook_types {
            for branch in &branches {
                let ctx = HookContext {
                    hook_type,
                    branch: branch.to_string(),
                    tag: "v1.0.0".to_string(),
                    remote: "origin".to_string(),
                    version_bump: None,
                    commit_count: None,
                };

                let env = ctx.to_env_vars();
                assert_eq!(env.get("GITPUBLISH_BRANCH"), Some(&branch.to_string()));
            }
        }
    }

    #[test]
    fn test_hook_context_prerelease_workflow() {
        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "develop".to_string(),
            tag: "v2.0.0-beta.1".to_string(),
            remote: "origin".to_string(),
            version_bump: Some("Minor".to_string()),
            commit_count: Some(3),
        };

        let env = ctx.to_env_vars();
        assert_eq!(
            env.get("GITPUBLISH_TAG_NAME"),
            Some(&"v2.0.0-beta.1".to_string())
        );
        assert_eq!(
            env.get("GITPUBLISH_VERSION_BUMP"),
            Some(&"Minor".to_string())
        );
    }

    #[test]
    fn test_hook_context_major_version_bump() {
        let ctx = HookContext {
            hook_type: HookType::PostTagCreate,
            branch: "main".to_string(),
            tag: "v2.0.0".to_string(),
            remote: "origin".to_string(),
            version_bump: Some("Major".to_string()),
            commit_count: Some(15),
        };

        let env = ctx.to_env_vars();
        assert_eq!(
            env.get("GITPUBLISH_VERSION_BUMP"),
            Some(&"Major".to_string())
        );
        assert_eq!(env.get("GITPUBLISH_COMMIT_COUNT"), Some(&"15".to_string()));
    }

    #[test]
    fn test_hook_context_various_remotes() {
        let remotes = vec!["origin", "upstream", "github", "gitlab"];

        for remote in remotes {
            let ctx = HookContext {
                hook_type: HookType::PostPush,
                branch: "main".to_string(),
                tag: "v1.0.0".to_string(),
                remote: remote.to_string(),
                version_bump: None,
                commit_count: None,
            };

            let env = ctx.to_env_vars();
            assert_eq!(env.get("GITPUBLISH_REMOTE"), Some(&remote.to_string()));
        }
    }

    #[test]
    fn test_hook_context_env_var_formats() {
        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "main".to_string(),
            tag: "v1.2.3".to_string(),
            remote: "origin".to_string(),
            version_bump: Some("Patch".to_string()),
            commit_count: Some(1),
        };

        let env = ctx.to_env_vars();

        // Verify all keys follow GITPUBLISH_ naming
        for (key, _value) in &env {
            assert!(
                key.starts_with("GITPUBLISH_"),
                "Key should start with GITPUBLISH_: {}",
                key
            );
        }

        // Verify all required fields are present
        assert!(env.contains_key("GITPUBLISH_BRANCH"));
        assert!(env.contains_key("GITPUBLISH_TAG_NAME"));
        assert!(env.contains_key("GITPUBLISH_REMOTE"));
    }

    #[test]
    fn test_hook_context_commit_count_display() {
        let counts = vec![0, 1, 5, 10, 100, 1000];

        for count in counts {
            let ctx = HookContext {
                hook_type: HookType::PostTagCreate,
                branch: "main".to_string(),
                tag: "v1.0.0".to_string(),
                remote: "origin".to_string(),
                version_bump: None,
                commit_count: Some(count),
            };

            let env = ctx.to_env_vars();
            assert_eq!(env.get("GITPUBLISH_COMMIT_COUNT"), Some(&count.to_string()));
        }
    }

    #[test]
    fn test_hook_type_all_variants() {
        let types = vec![
            HookType::PreTagCreate,
            HookType::PostTagCreate,
            HookType::PostPush,
        ];

        let names = vec!["pre-tag-create", "post-tag-create", "post-push"];

        for (hook_type, expected_name) in types.iter().zip(names.iter()) {
            assert_eq!(hook_type.name(), *expected_name);
        }
    }

    #[test]
    fn test_hook_context_with_special_characters_in_branch() {
        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "release/v1.0.0".to_string(),
            tag: "v1.0.0".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let env = ctx.to_env_vars();
        assert_eq!(
            env.get("GITPUBLISH_BRANCH"),
            Some(&"release/v1.0.0".to_string())
        );
    }

    #[test]
    fn test_hook_context_version_bump_all_types() {
        let bump_types = vec!["Major", "Minor", "Patch"];

        for bump_type in bump_types {
            let ctx = HookContext {
                hook_type: HookType::PostTagCreate,
                branch: "main".to_string(),
                tag: "v1.0.0".to_string(),
                remote: "origin".to_string(),
                version_bump: Some(bump_type.to_string()),
                commit_count: None,
            };

            let env = ctx.to_env_vars();
            assert_eq!(
                env.get("GITPUBLISH_VERSION_BUMP"),
                Some(&bump_type.to_string())
            );
        }
    }
}
