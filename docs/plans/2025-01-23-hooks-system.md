# Hooks System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Create a git hooks system that allows users to run custom scripts at specific lifecycle points (pre-tag-create, post-tag-create, post-push) in the git-publish workflow.

**Architecture:** The hooks system consists of three components: (1) lifecycle definitions (HookType enum, HookContext struct for environment variables), (2) hook executor for running scripts with proper environment setup, (3) configuration support in gitpublish.toml for specifying hook script paths. Environment variables allow hooks to access workflow state; exit codes control flow (0=continue, non-zero=abort or warn depending on hook type).

**Tech Stack:** Rust 2021, std::process::Command, serde/toml for config, anyhow/thiserror for errors.

---

## Task 1: Create hooks module root

**Files:**
- Create: `src/hooks/mod.rs`

**Step 1: Write the failing test**

Create a unit test in `src/hooks/mod.rs` that will fail because HookType doesn't exist:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_type_variants_exist() {
        // This test verifies that HookType can be created
        let _hook = lifecycle::HookType::PreTagCreate;
        let _hook = lifecycle::HookType::PostTagCreate;
        let _hook = lifecycle::HookType::PostPush;
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib hooks::tests::test_hook_type_variants_exist`

Expected: FAIL - "no module named `lifecycle`"

**Step 3: Write minimal module exports**

```rust
//! Git hooks system for extensibility
//!
//! Provides a flexible system for users to run custom scripts at key points
//! in the git-publish workflow: before tag creation, after tag creation, and
//! after successful push to remote.

pub mod lifecycle;
pub mod executor;

pub use lifecycle::{HookType, HookContext};
pub use executor::HookExecutor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_type_variants_exist() {
        // This test verifies that HookType can be created
        let _hook = lifecycle::HookType::PreTagCreate;
        let _hook = lifecycle::HookType::PostTagCreate;
        let _hook = lifecycle::HookType::PostPush;
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib hooks::tests::test_hook_type_variants_exist`

Expected: PASS (though other tests will fail due to missing modules)

**Step 5: Update lib.rs to include hooks module**

Edit `src/lib.rs` and add:

```rust
pub mod hooks;
```

After the existing module declarations (around line 10).

**Step 6: Run full test suite to verify nothing broke**

Run: `cargo test --lib`

Expected: Compilation should fail (missing lifecycle and executor modules) - this is expected and next tasks will fix it.

**Step 7: Commit**

```bash
git add src/hooks/mod.rs src/lib.rs
git commit -m "feat: add hooks module root structure"
```

---

## Task 2: Create lifecycle definitions

**Files:**
- Create: `src/hooks/lifecycle.rs`

**Step 1: Write the failing test for HookType**

Create a comprehensive test file that will fail:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_type_name_pre_tag_create() {
        assert_eq!(HookType::PreTagCreate.name(), "pre-tag-create");
    }

    #[test]
    fn test_hook_type_name_post_tag_create() {
        assert_eq!(HookType::PostTagCreate.name(), "post-tag-create");
    }

    #[test]
    fn test_hook_type_name_post_push() {
        assert_eq!(HookType::PostPush.name(), "post-push");
    }

    #[test]
    fn test_hook_context_to_env_vars_basic() {
        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "main".to_string(),
            tag: "v1.2.3".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let env = ctx.to_env_vars();
        assert_eq!(env.get("GITPUBLISH_BRANCH"), Some(&"main".to_string()));
        assert_eq!(env.get("GITPUBLISH_TAG_NAME"), Some(&"v1.2.3".to_string()));
        assert_eq!(env.get("GITPUBLISH_REMOTE"), Some(&"origin".to_string()));
    }

    #[test]
    fn test_hook_context_to_env_vars_with_optional_fields() {
        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "main".to_string(),
            tag: "v1.2.3".to_string(),
            remote: "origin".to_string(),
            version_bump: Some("Minor".to_string()),
            commit_count: Some(5),
        };

        let env = ctx.to_env_vars();
        assert_eq!(env.get("GITPUBLISH_VERSION_BUMP"), Some(&"Minor".to_string()));
        assert_eq!(env.get("GITPUBLISH_COMMIT_COUNT"), Some(&"5".to_string()));
    }

    #[test]
    fn test_hook_context_to_env_vars_without_optional_fields() {
        let ctx = HookContext {
            hook_type: HookType::PostPush,
            branch: "develop".to_string(),
            tag: "d1.0.0".to_string(),
            remote: "upstream".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let env = ctx.to_env_vars();
        assert!(!env.contains_key("GITPUBLISH_VERSION_BUMP"));
        assert!(!env.contains_key("GITPUBLISH_COMMIT_COUNT"));
        assert_eq!(env.len(), 3); // Only branch, tag, remote
    }

    #[test]
    fn test_hook_context_derive_debug() {
        let ctx = HookContext {
            hook_type: HookType::PostTagCreate,
            branch: "main".to_string(),
            tag: "v2.0.0".to_string(),
            remote: "origin".to_string(),
            version_bump: Some("Major".to_string()),
            commit_count: Some(10),
        };

        let debug_str = format!("{:?}", ctx);
        assert!(debug_str.contains("PostTagCreate"));
        assert!(debug_str.contains("main"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib hooks::lifecycle::tests`

Expected: FAIL - "struct HookType is not defined"

**Step 3: Write minimal lifecycle implementation**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of hooks in the workflow
///
/// Defines the three key extension points in the git-publish workflow
/// where users can run custom scripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookType {
    /// Runs before creating the tag
    PreTagCreate,
    /// Runs after tag created locally, before push
    PostTagCreate,
    /// Runs after tag pushed to remote
    PostPush,
}

impl HookType {
    /// Get the canonical name of this hook type
    ///
    /// Returns the hook name as it appears in configuration files.
    pub fn name(&self) -> &'static str {
        match self {
            HookType::PreTagCreate => "pre-tag-create",
            HookType::PostTagCreate => "post-tag-create",
            HookType::PostPush => "post-push",
        }
    }
}

/// Context passed to a hook script
///
/// Contains all information the hook needs to make decisions,
/// which is passed as environment variables to the script.
#[derive(Debug, Clone)]
pub struct HookContext {
    /// Type of hook being executed
    pub hook_type: HookType,
    /// Git branch being tagged
    pub branch: String,
    /// Tag name being created/pushed
    pub tag: String,
    /// Remote name to push to
    pub remote: String,
    /// Version bump type (Major/Minor/Patch), if applicable
    pub version_bump: Option<String>,
    /// Number of commits since last tag, if applicable
    pub commit_count: Option<usize>,
}

impl HookContext {
    /// Convert hook context to environment variables
    ///
    /// Returns a HashMap of environment variable names and values
    /// that will be passed to the hook script. Always includes:
    /// - GITPUBLISH_BRANCH
    /// - GITPUBLISH_TAG_NAME
    /// - GITPUBLISH_REMOTE
    ///
    /// Optionally includes (if Some):
    /// - GITPUBLISH_VERSION_BUMP
    /// - GITPUBLISH_COMMIT_COUNT
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
    fn test_hook_type_name_pre_tag_create() {
        assert_eq!(HookType::PreTagCreate.name(), "pre-tag-create");
    }

    #[test]
    fn test_hook_type_name_post_tag_create() {
        assert_eq!(HookType::PostTagCreate.name(), "post-tag-create");
    }

    #[test]
    fn test_hook_type_name_post_push() {
        assert_eq!(HookType::PostPush.name(), "post-push");
    }

    #[test]
    fn test_hook_context_to_env_vars_basic() {
        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "main".to_string(),
            tag: "v1.2.3".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let env = ctx.to_env_vars();
        assert_eq!(env.get("GITPUBLISH_BRANCH"), Some(&"main".to_string()));
        assert_eq!(env.get("GITPUBLISH_TAG_NAME"), Some(&"v1.2.3".to_string()));
        assert_eq!(env.get("GITPUBLISH_REMOTE"), Some(&"origin".to_string()));
    }

    #[test]
    fn test_hook_context_to_env_vars_with_optional_fields() {
        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "main".to_string(),
            tag: "v1.2.3".to_string(),
            remote: "origin".to_string(),
            version_bump: Some("Minor".to_string()),
            commit_count: Some(5),
        };

        let env = ctx.to_env_vars();
        assert_eq!(env.get("GITPUBLISH_VERSION_BUMP"), Some(&"Minor".to_string()));
        assert_eq!(env.get("GITPUBLISH_COMMIT_COUNT"), Some(&"5".to_string()));
    }

    #[test]
    fn test_hook_context_to_env_vars_without_optional_fields() {
        let ctx = HookContext {
            hook_type: HookType::PostPush,
            branch: "develop".to_string(),
            tag: "d1.0.0".to_string(),
            remote: "upstream".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let env = ctx.to_env_vars();
        assert!(!env.contains_key("GITPUBLISH_VERSION_BUMP"));
        assert!(!env.contains_key("GITPUBLISH_COMMIT_COUNT"));
        assert_eq!(env.len(), 3); // Only branch, tag, remote
    }

    #[test]
    fn test_hook_context_derive_debug() {
        let ctx = HookContext {
            hook_type: HookType::PostTagCreate,
            branch: "main".to_string(),
            tag: "v2.0.0".to_string(),
            remote: "origin".to_string(),
            version_bump: Some("Major".to_string()),
            commit_count: Some(10),
        };

        let debug_str = format!("{:?}", ctx);
        assert!(debug_str.contains("PostTagCreate"));
        assert!(debug_str.contains("main"));
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib hooks::lifecycle::tests`

Expected: PASS - All 6 tests passing

**Step 5: Run full test suite**

Run: `cargo test --lib`

Expected: Compilation should still fail (missing executor module) but lifecycle tests pass

**Step 6: Commit**

```bash
git add src/hooks/lifecycle.rs
git commit -m "feat: add HookType and HookContext definitions"
```

---

## Task 3: Create hook executor

**Files:**
- Create: `src/hooks/executor.rs`

**Step 1: Write the failing tests**

Create comprehensive tests that will guide implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::lifecycle::{HookContext, HookType};
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_execute_nonexistent_hook_fails() {
        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "main".to_string(),
            tag: "v1.0.0".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let result = HookExecutor::execute("/nonexistent/hook.sh", &ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not found"));
    }

    #[test]
    fn test_execute_directory_instead_of_file_fails() {
        let ctx = HookContext {
            hook_type: HookType::PostTagCreate,
            branch: "main".to_string(),
            tag: "v1.0.0".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let result = HookExecutor::execute("/tmp", &ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not a file"));
    }

    #[test]
    fn test_execute_successful_hook() {
        // Create a temporary script that succeeds
        let mut script = NamedTempFile::new().unwrap();
        writeln!(script, "#!/bin/bash").unwrap();
        writeln!(script, "exit 0").unwrap();
        script.flush().unwrap();

        // Make it executable (Unix only, but test is platform-specific anyway)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(script.path(), std::fs::Permissions::from_mode(0o755))
                .unwrap();
        }

        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "main".to_string(),
            tag: "v1.0.0".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let result = HookExecutor::execute(script.path().to_str().unwrap(), &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_hook_with_failing_exit_code() {
        let mut script = NamedTempFile::new().unwrap();
        writeln!(script, "#!/bin/bash").unwrap();
        writeln!(script, "exit 1").unwrap();
        script.flush().unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(script.path(), std::fs::Permissions::from_mode(0o755))
                .unwrap();
        }

        let ctx = HookContext {
            hook_type: HookType::PostPush,
            branch: "main".to_string(),
            tag: "v1.0.0".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let result = HookExecutor::execute(script.path().to_str().unwrap(), &ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("failed"));
    }

    #[test]
    fn test_execute_hook_receives_environment_variables() {
        let mut script = NamedTempFile::new().unwrap();
        writeln!(script, "#!/bin/bash").unwrap();
        writeln!(script, "[ \"$GITPUBLISH_BRANCH\" = \"main\" ] || exit 1").unwrap();
        writeln!(script, "[ \"$GITPUBLISH_TAG_NAME\" = \"v1.2.3\" ] || exit 1").unwrap();
        writeln!(script, "[ \"$GITPUBLISH_REMOTE\" = \"origin\" ] || exit 1").unwrap();
        writeln!(script, "exit 0").unwrap();
        script.flush().unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(script.path(), std::fs::Permissions::from_mode(0o755))
                .unwrap();
        }

        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "main".to_string(),
            tag: "v1.2.3".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let result = HookExecutor::execute(script.path().to_str().unwrap(), &ctx);
        assert!(result.is_ok());
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib hooks::executor::tests`

Expected: FAIL - "struct HookExecutor is not defined"

**Step 3: Write minimal executor implementation**

```rust
use crate::error::GitPublishError;
use crate::hooks::lifecycle::HookContext;
use crate::Result;
use std::path::Path;
use std::process::Command;

/// Executes git-publish hooks
///
/// HookExecutor runs hook scripts with proper environment variable setup
/// and error handling. It validates that scripts exist before execution
/// and interprets exit codes according to hook semantics.
pub struct HookExecutor;

impl HookExecutor {
    /// Execute a hook script
    ///
    /// Runs the script at the specified path with environment variables
    /// from the hook context. The script must be executable.
    ///
    /// # Arguments
    /// * `script_path` - Path to the hook script to execute
    /// * `context` - Hook context with environment variables
    ///
    /// # Returns
    /// * `Ok(())` if hook succeeds (exit code 0)
    /// * `Err` if hook fails (non-zero exit code), script not found, or not a file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use git_publish::hooks::{HookExecutor, HookContext, HookType};
    ///
    /// let ctx = HookContext {
    ///     hook_type: HookType::PreTagCreate,
    ///     branch: "main".to_string(),
    ///     tag: "v1.0.0".to_string(),
    ///     remote: "origin".to_string(),
    ///     version_bump: None,
    ///     commit_count: None,
    /// };
    ///
    /// let result = HookExecutor::execute("./hooks/pre-tag-create.sh", &ctx);
    /// ```
    pub fn execute(script_path: &str, context: &HookContext) -> Result<()> {
        let path = Path::new(script_path);

        if !path.exists() {
            return Err(GitPublishError::hook(format!(
                "Hook script not found: {}",
                script_path
            )));
        }

        if !path.is_file() {
            return Err(GitPublishError::hook(format!(
                "Hook path is not a file: {}",
                script_path
            )));
        }

        let env_vars = context.to_env_vars();

        let mut cmd = Command::new(script_path);

        // Add environment variables from context
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        let output = cmd.output().map_err(|e| {
            GitPublishError::hook(format!(
                "Failed to execute hook {}: {}",
                script_path, e
            ))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(GitPublishError::hook(format!(
                "Hook {} failed with exit code {}\nStdout: {}\nStderr: {}",
                script_path,
                output.status.code().unwrap_or(-1),
                stdout,
                stderr
            )));
        }

        Ok(())
    }

    /// Try to execute a hook, logging warnings but not failing
    ///
    /// Used for post-push hooks where the push has already succeeded
    /// and we don't want hook failures to affect the overall success.
    /// Logs warnings to stderr on failure but doesn't return an error.
    ///
    /// # Arguments
    /// * `script_path` - Path to the hook script to execute
    /// * `context` - Hook context with environment variables
    pub fn execute_permissive(script_path: &str, context: &HookContext) {
        match Self::execute(script_path, context) {
            Ok(()) => {
                println!("✓ Hook executed successfully: {}", script_path);
            }
            Err(e) => {
                eprintln!("⚠ Hook warning: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::lifecycle::{HookContext, HookType};
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_execute_nonexistent_hook_fails() {
        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "main".to_string(),
            tag: "v1.0.0".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let result = HookExecutor::execute("/nonexistent/hook.sh", &ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not found"));
    }

    #[test]
    fn test_execute_directory_instead_of_file_fails() {
        let ctx = HookContext {
            hook_type: HookType::PostTagCreate,
            branch: "main".to_string(),
            tag: "v1.0.0".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let result = HookExecutor::execute("/tmp", &ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not a file"));
    }

    #[test]
    fn test_execute_successful_hook() {
        let mut script = NamedTempFile::new().unwrap();
        writeln!(script, "#!/bin/bash").unwrap();
        writeln!(script, "exit 0").unwrap();
        script.flush().unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(script.path(), std::fs::Permissions::from_mode(0o755))
                .unwrap();
        }

        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "main".to_string(),
            tag: "v1.0.0".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let result = HookExecutor::execute(script.path().to_str().unwrap(), &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_hook_with_failing_exit_code() {
        let mut script = NamedTempFile::new().unwrap();
        writeln!(script, "#!/bin/bash").unwrap();
        writeln!(script, "exit 1").unwrap();
        script.flush().unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(script.path(), std::fs::Permissions::from_mode(0o755))
                .unwrap();
        }

        let ctx = HookContext {
            hook_type: HookType::PostPush,
            branch: "main".to_string(),
            tag: "v1.0.0".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let result = HookExecutor::execute(script.path().to_str().unwrap(), &ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("failed"));
    }

    #[test]
    fn test_execute_hook_receives_environment_variables() {
        let mut script = NamedTempFile::new().unwrap();
        writeln!(script, "#!/bin/bash").unwrap();
        writeln!(script, "[ \"$GITPUBLISH_BRANCH\" = \"main\" ] || exit 1").unwrap();
        writeln!(script, "[ \"$GITPUBLISH_TAG_NAME\" = \"v1.2.3\" ] || exit 1").unwrap();
        writeln!(script, "[ \"$GITPUBLISH_REMOTE\" = \"origin\" ] || exit 1").unwrap();
        writeln!(script, "exit 0").unwrap();
        script.flush().unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(script.path(), std::fs::Permissions::from_mode(0o755))
                .unwrap();
        }

        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "main".to_string(),
            tag: "v1.2.3".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let result = HookExecutor::execute(script.path().to_str().unwrap(), &ctx);
        assert!(result.is_ok());
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib hooks::executor::tests`

Expected: PASS - All 5 tests passing

**Step 5: Run full test suite**

Run: `cargo test --lib`

Expected: All tests pass OR remaining failures are due to config tests needing HooksConfig

**Step 6: Format and lint**

Run: `cargo fmt && cargo clippy -- -D warnings`

Expected: No warnings or errors

**Step 7: Commit**

```bash
git add src/hooks/executor.rs
git commit -m "feat: add HookExecutor for running hook scripts"
```

---

## Task 4: Add hook configuration

**Files:**
- Modify: `src/config.rs`

**Step 1: Write the failing test**

Edit `src/config.rs` and add a test that will fail:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_config_default() {
        let hooks = HooksConfig::default();
        assert_eq!(hooks.pre_tag_create, None);
        assert_eq!(hooks.post_tag_create, None);
        assert_eq!(hooks.post_push, None);
    }

    #[test]
    fn test_config_with_hooks() {
        let mut config = Config::default();
        config.hooks = HooksConfig {
            pre_tag_create: Some("./hooks/pre-tag-create.sh".to_string()),
            post_tag_create: Some("./hooks/post-tag-create.sh".to_string()),
            post_push: Some("./hooks/post-push.sh".to_string()),
        };
        assert_eq!(config.hooks.pre_tag_create, Some("./hooks/pre-tag-create.sh".to_string()));
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib config::tests::test_hooks_config_default`

Expected: FAIL - "struct HooksConfig is not defined"

**Step 3: Write minimal HooksConfig struct and update Config**

Add to `src/config.rs` after the PreReleaseConfig struct (before the Config struct definition):

```rust
/// Configuration for git hooks.
///
/// Specifies paths to optional hook scripts that will be executed at
/// key points in the git-publish workflow.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct HooksConfig {
    /// Path to pre-tag-create hook script
    ///
    /// Executed before the tag is created. Can inspect commits and abort
    /// tag creation if needed (exit code non-zero).
    pub pre_tag_create: Option<String>,

    /// Path to post-tag-create hook script
    ///
    /// Executed after tag is created locally but before push. Can validate
    /// tag format or prepare for push (exit code non-zero aborts push).
    pub post_tag_create: Option<String>,

    /// Path to post-push hook script
    ///
    /// Executed after successful push to remote. Can trigger deployments
    /// or update documentation. Failures logged but don't fail overall
    /// operation (push already succeeded).
    pub post_push: Option<String>,
}
```

Then update the Config struct definition to include hooks:

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub branches: HashMap<String, String>,

    #[serde(default)]
    pub conventional_commits: ConventionalCommitsConfig,

    #[serde(default)]
    pub patterns: PatternsConfig,

    #[serde(default)]
    pub behavior: BehaviorConfig,

    #[serde(default)]
    pub prerelease: PreReleaseConfig,

    #[serde(default)]
    pub hooks: HooksConfig,  // NEW
}
```

Then update the Config::default() implementation:

```rust
impl Default for Config {
    fn default() -> Self {
        let mut branches = HashMap::new();
        branches.insert("main".to_string(), "v{version}".to_string());
        branches.insert("develop".to_string(), "d{version}".to_string());
        branches.insert("gray".to_string(), "g{version}".to_string());

        Config {
            branches,
            conventional_commits: ConventionalCommitsConfig::default(),
            patterns: PatternsConfig::default(),
            behavior: BehaviorConfig::default(),
            prerelease: PreReleaseConfig::default(),
            hooks: HooksConfig::default(),  // NEW
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib config::tests::test_hooks_config_default`

Expected: PASS

Run: `cargo test --lib config::tests::test_config_with_hooks`

Expected: PASS

**Step 5: Run full test suite**

Run: `cargo test --lib`

Expected: All tests pass

**Step 6: Format and lint**

Run: `cargo fmt && cargo clippy -- -D warnings`

Expected: No warnings or errors

**Step 7: Commit**

```bash
git add src/config.rs
git commit -m "feat: add HooksConfig to support hook script paths"
```

---

## Task 5: Add hooks to integration tests

**Files:**
- Modify: `tests/config_test.rs`

**Step 1: Write the failing test**

Add a test to `tests/config_test.rs` that will fail:

```rust
#[test]
fn test_load_config_with_hooks() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let toml_content = r#"
[branches]
"main" = "v{version}"

[hooks]
pre-tag-create = "./hooks/pre-tag-create.sh"
post-tag-create = "./hooks/post-tag-create.sh"
post-push = "./hooks/post-push.sh"
"#;
    temp_file.write_all(toml_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let config = load_config(Some(temp_file.path().to_str().unwrap())).unwrap();
    assert_eq!(
        config.hooks.pre_tag_create,
        Some("./hooks/pre-tag-create.sh".to_string())
    );
    assert_eq!(
        config.hooks.post_tag_create,
        Some("./hooks/post-tag-create.sh".to_string())
    );
    assert_eq!(
        config.hooks.post_push,
        Some("./hooks/post-push.sh".to_string())
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test config_test test_load_config_with_hooks`

Expected: FAIL - "unknown field `pre_tag_create`" or similar

**Step 3: Update TOML field names**

The issue is that TOML uses kebab-case but Rust uses snake_case. Add `#[serde(rename = "...")]` attributes to HooksConfig:

```rust
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct HooksConfig {
    /// Path to pre-tag-create hook script
    #[serde(rename = "pre-tag-create")]
    pub pre_tag_create: Option<String>,

    /// Path to post-tag-create hook script
    #[serde(rename = "post-tag-create")]
    pub post_tag_create: Option<String>,

    /// Path to post-push hook script
    #[serde(rename = "post-push")]
    pub post_push: Option<String>,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test config_test test_load_config_with_hooks`

Expected: PASS

**Step 5: Run full integration test suite**

Run: `cargo test --test config_test`

Expected: All tests pass

**Step 6: Run full test suite**

Run: `cargo test`

Expected: All tests pass

**Step 7: Commit**

```bash
git add src/config.rs tests/config_test.rs
git commit -m "test: add integration test for hook configuration loading"
```

---

## Task 6: Format, lint, and test all

**Step 1: Format code**

Run: `cargo fmt`

Expected: Code formatted to Rust style

**Step 2: Lint code**

Run: `cargo clippy -- -D warnings`

Expected: No warnings or errors (clean output)

**Step 3: Run all tests**

Run: `cargo test`

Expected: All tests pass, output shows test count

**Step 4: Build project**

Run: `cargo build`

Expected: Build succeeds with no warnings

**Step 5: Verify documentation is correct**

Check that:
- All public items have doc comments
- Examples in doc comments are accurate
- No "TODO" or "FIXME" comments left

Run: `cargo test --doc`

Expected: All doc tests pass

**Step 6: Final commit**

Run: `git status`

Expected: Clean working directory (no uncommitted changes)

If any files are unstaged, commit them:

```bash
git add .
git commit -m "style: format and lint hooks system"
```

---

## Completion Checklist

- [ ] `src/hooks/mod.rs` created and exports HookType, HookContext, HookExecutor
- [ ] `src/hooks/lifecycle.rs` created with HookType enum and HookContext struct
- [ ] `src/hooks/executor.rs` created with HookExecutor for running scripts
- [ ] `src/config.rs` updated with HooksConfig struct
- [ ] `Config` struct includes `hooks: HooksConfig` field
- [ ] All tests pass (`cargo test`)
- [ ] Code compiles without warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is complete and accurate
- [ ] TOML field names use kebab-case (pre-tag-create, not pre_tag_create)
- [ ] Rust struct fields use snake_case (pre_tag_create)
- [ ] Environment variables passed correctly (GITPUBLISH_*)
- [ ] Hook errors are handled with GitPublishError::hook()
- [ ] All commits created with descriptive messages
