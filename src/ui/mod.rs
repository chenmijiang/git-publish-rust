//! User interface module - interaction (prompts) and formatting.
//!
//! Separates concerns:
//! - `formatter` - Pure formatting functions
//! - This module - Interactive prompts and user input handling

use std::io::{self, Write};

use anyhow::Result;

pub mod formatter;

// Re-export formatter functions for convenience
pub use formatter::{
    display_available_branches, display_boundary_warning, display_commit_analysis, display_error,
    display_manual_push_instruction, display_proposed_tag, display_status, display_success,
};

/// Prompts user to select a branch from available options.
///
/// If only one branch is available, returns it directly without prompting.
/// Otherwise displays numbered list and accepts 1-based index selection.
/// Default selection is the first branch (index 1) if user presses Enter.
///
/// # Arguments
/// * `available_branches` - List of branch names to choose from
///
/// # Returns
/// * `Ok(String)` - The selected branch name
/// * `Err` - If selection is invalid
pub fn select_branch(available_branches: &[String]) -> Result<String> {
    if available_branches.len() == 1 {
        return Ok(available_branches[0].clone());
    }

    println!("\n\x1b[1mAvailable branches for tagging:\x1b[0m");
    for (i, branch) in available_branches.iter().enumerate() {
        println!("  {}. {}", i + 1, branch);
    }

    print!(
        "\nSelect a branch (1-{}) [default: 1]: ",
        available_branches.len()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selection = input.trim();

    // If empty input, default to first branch (index 1)
    let index = if selection.is_empty() {
        1
    } else {
        selection.parse::<usize>().unwrap_or(0)
    };

    if index > 0 && index <= available_branches.len() {
        Ok(available_branches[index - 1].clone())
    } else {
        Err(anyhow::anyhow!("Invalid selection"))
    }
}

/// Prompts user to select a remote for fetch/push operations.
///
/// If only one remote exists, returns it directly without prompting.
/// Displays all available remotes and allows selection, with first remote as default.
///
/// # Arguments
/// * `available_remotes` - List of remote names (preferably sorted with "origin" first)
///
/// # Returns
/// * `Ok(String)` - The selected remote name
/// * `Err` - If selection is invalid
pub fn select_remote(available_remotes: &[String]) -> Result<String> {
    if available_remotes.len() == 1 {
        return Ok(available_remotes[0].clone());
    }

    println!("\n\x1b[1mAvailable remotes:\x1b[0m");
    for (i, remote) in available_remotes.iter().enumerate() {
        println!("  {}. {}", i + 1, remote);
    }

    print!(
        "\nSelect a remote for fetch/push (1-{}) [default: 1]: ",
        available_remotes.len()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selection = input.trim();

    // If empty input, default to first remote (index 1)
    let index = if selection.is_empty() {
        1
    } else {
        selection.parse::<usize>().unwrap_or(0)
    };

    if index > 0 && index <= available_remotes.len() {
        Ok(available_remotes[index - 1].clone())
    } else {
        Err(anyhow::anyhow!("Invalid remote selection"))
    }
}

/// Prompts user to confirm an action with a yes/no prompt.
///
/// Displays the given prompt and accepts "y" or "yes" (case-insensitive) as confirmation.
/// Default is "no" if user presses Enter.
///
/// # Arguments
/// * `prompt` - The prompt message to display (without the "(y/N): " suffix)
///
/// # Returns
/// * `Ok(true)` - If user entered "y" or "yes"
/// * `Ok(false)` - Otherwise (including Enter, or "n"/"no")
/// * `Err` - If input error occurs
pub fn confirm_action(prompt: &str) -> Result<bool> {
    print!("\n{} (y/N): ", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let response = input.trim().to_lowercase();
    Ok(response == "y" || response == "yes")
}

/// Validates that a tag matches the configured pattern.
///
/// Checks if the tag conforms to the pattern (e.g., "v{version}" -> "v1.2.3").
/// If pattern has no {version} placeholder, any tag is valid.
/// Validates that the version part contains only digits and dots.
///
/// # Arguments
/// * `tag` - The tag to validate (e.g., "v1.2.3")
/// * `pattern` - The pattern template (e.g., "v{version}" or "release-v{version}-final")
///
/// # Returns
/// * `Ok(())` - If the tag matches the pattern
/// * `Err(anyhow::Error)` - If the tag doesn't match or pattern is invalid
///
/// # Examples
///
/// ```ignore
/// validate_tag_format("v1.2.3", "v{version}")                   // Ok
/// validate_tag_format("1.2.3", "v{version}")                    // Err - missing prefix
/// validate_tag_format("v1.2.3-release", "v{version}-release")   // Ok
/// validate_tag_format("anything", "free-form")                  // Ok (no {version})
/// ```
pub fn validate_tag_format(tag: &str, pattern: &str) -> Result<()> {
    // If pattern doesn't contain {version}, no validation needed
    if !pattern.contains("{version}") {
        return Ok(());
    }

    // Extract prefix and suffix from pattern around {version}
    let parts: Vec<&str> = pattern.split("{version}").collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!(
            "Invalid pattern '{}': should have exactly one {{version}} placeholder",
            pattern
        ));
    }

    let prefix = parts[0];
    let suffix = parts[1];

    // Check if tag starts with prefix
    if !tag.starts_with(prefix) {
        return Err(anyhow::anyhow!(
            "Tag '{}' does not match pattern '{}': missing prefix '{}'",
            tag,
            pattern,
            prefix
        ));
    }

    // Check if tag ends with suffix
    if !tag.ends_with(suffix) {
        return Err(anyhow::anyhow!(
            "Tag '{}' does not match pattern '{}': missing suffix '{}'",
            tag,
            pattern,
            suffix
        ));
    }

    // Extract version part
    let version_part = &tag[prefix.len()..tag.len() - suffix.len()];

    // Validate it looks like a version (basic check: contains only digits and dots)
    if !version_part.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return Err(anyhow::anyhow!(
            "Tag '{}' has invalid version format '{}'",
            tag,
            version_part
        ));
    }

    Ok(())
}

/// Prompts user to select or customize a tag.
///
/// Provides three options:
/// 1. Press Enter to use the recommended tag
/// 2. Enter a custom tag
/// 3. Enter 'e' to edit the recommended tag
///
/// # Arguments
/// * `recommended_tag` - The default recommended tag
/// * `_pattern` - The tag pattern for validation (currently unused but kept for API compatibility)
///
/// # Returns
/// * `Ok(String)` - The selected or customized tag
/// * `Err` - If input error occurs
///
/// # Examples
/// ```ignore
/// let tag = select_or_customize_tag("v1.2.3", "v{version}")?;
/// // Returns "v1.2.3" if user presses Enter
/// // Returns custom tag if user enters one
/// // Returns edited tag if user enters 'e'
/// ```
pub fn select_or_customize_tag(recommended_tag: &str, _pattern: &str) -> Result<String> {
    print!(
        "\nTag options:\n  (press Enter to use recommended)\n  (enter custom tag)\n  (enter 'e' to edit)\n\nTag [{}]: ",
        recommended_tag
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    match input {
        "" => Ok(recommended_tag.to_string()),
        "e" => {
            print!("Edit tag [{}]: ", recommended_tag);
            io::stdout().flush()?;

            let mut edited = String::new();
            io::stdin().read_line(&mut edited)?;
            Ok(edited.trim().to_string())
        }
        custom => Ok(custom.to_string()),
    }
}

/// Confirms tag use with format validation.
///
/// Validates that the tag matches the configured pattern, then asks for confirmation.
/// Default is to confirm (user must enter 'n' or 'no' to decline).
///
/// # Arguments
/// * `tag` - The tag to validate and confirm
/// * `pattern` - The tag pattern to validate against
///
/// # Returns
/// * `Ok(true)` - If user confirms after successful validation (or presses Enter)
/// * `Ok(false)` - If user enters 'n' or 'no'
/// * `Err` - If validation fails or input error occurs
///
/// # Examples
/// ```ignore
/// if confirm_tag_use("v1.2.3", "v{version}")? {
///     // Proceed with tag creation
/// }
/// ```
pub fn confirm_tag_use(tag: &str, pattern: &str) -> Result<bool> {
    // First validate the tag format
    validate_tag_format(tag, pattern)?;

    // If validation passed, confirm with user
    // Default is Y (confirm) - user needs to enter 'n' or 'no' to decline
    print!("\nConfirm tag creation: {} (Y/n): ", tag);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let response = input.trim().to_lowercase();
    // Default to true (confirm) if empty input; only return false if user explicitly says 'n' or 'no'
    Ok(!(response == "n" || response == "no"))
}

/// Prompts user to confirm pushing a locally created tag to a remote.
///
/// Asks if the user wants to push the tag to the specified remote.
/// Default is not to push (user must enter "y" or "yes" to confirm).
///
/// # Arguments
/// * `tag` - The tag that was created locally
/// * `remote` - The remote name (e.g., "origin")
///
/// # Returns
/// * `Ok(true)` - If user enters "y" or "yes"
/// * `Ok(false)` - Otherwise (including Enter or "n"/"no")
/// * `Err` - If input error occurs
///
/// # Examples
/// ```ignore
/// if confirm_push_tag("v1.2.3", "origin")? {
///     // Push the tag to remote
/// }
/// ```
pub fn confirm_push_tag(tag: &str, remote: &str) -> Result<bool> {
    print!(
        "\nTag '{}' created locally. Push to remote '{}' (y/N): ",
        tag, remote
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let response = input.trim().to_lowercase();
    Ok(response == "y" || response == "yes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_tag_format_simple() {
        assert!(validate_tag_format("v1.2.3", "v{version}").is_ok());
    }

    #[test]
    fn test_validate_tag_format_no_constraint() {
        assert!(validate_tag_format("anything", "free-form").is_ok());
    }

    #[test]
    fn test_validate_tag_format_with_suffix() {
        assert!(validate_tag_format("v1.2.3-release", "v{version}-release").is_ok());
    }

    #[test]
    fn test_validate_tag_format_missing_prefix() {
        let result = validate_tag_format("1.2.3", "v{version}");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tag_format_missing_suffix() {
        let result = validate_tag_format("v1.2.3", "v{version}-release");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tag_format_invalid_version() {
        let result = validate_tag_format("v1.2.3abc", "v{version}");
        assert!(result.is_err());
    }
}
