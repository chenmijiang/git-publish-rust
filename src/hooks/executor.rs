use crate::error::{GitPublishError, Result};
use crate::hooks::HookContext;
use std::path::Path;
use std::process::Command;

/// Executes git-publish hook scripts
pub struct HookExecutor;

impl HookExecutor {
    /// Execute a hook script with the given context
    ///
    /// The script is executed with environment variables set from the context.
    /// If the script exits with code 0, the hook succeeds. Any non-zero exit code
    /// is treated as a failure.
    ///
    /// # Arguments
    /// * `script_path` - Path to the hook script (must be executable)
    /// * `context` - Hook context with environment variables
    ///
    /// # Returns
    /// * `Ok(())` if hook succeeds (exit code 0)
    /// * `Err` if script not found, not executable, or returns non-zero exit code
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

        // Add environment variables to the command
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        let output = cmd.output().map_err(|e| {
            GitPublishError::hook(format!("Failed to execute hook {}: {}", script_path, e))
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

    /// Try to execute a hook, logging errors but not failing
    ///
    /// Used for post-push hooks where the push has already succeeded and we
    /// don't want a hook failure to retroactively fail the operation.
    ///
    /// # Arguments
    /// * `script_path` - Path to the hook script
    /// * `context` - Hook context
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
    use crate::hooks::HookType;

    #[test]
    fn test_nonexistent_hook_fails() {
        let ctx = HookContext {
            hook_type: HookType::PreTagCreate,
            branch: "main".to_string(),
            tag: "v1.0.0".to_string(),
            remote: "origin".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let result = HookExecutor::execute("/nonexistent/path/to/hook.sh", &ctx);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Hook script not found"));
    }

    #[test]
    fn test_hook_directory_fails() {
        let ctx = HookContext {
            hook_type: HookType::PostTagCreate,
            branch: "develop".to_string(),
            tag: "v2.0.0".to_string(),
            remote: "upstream".to_string(),
            version_bump: None,
            commit_count: None,
        };

        let result = HookExecutor::execute("/tmp", &ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a file"));
    }
}
