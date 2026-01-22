use anyhow::Result;
use std::io::{self, Write};

use crate::boundary::BoundaryWarning;

pub fn display_error(message: &str) {
    eprintln!("\x1b[31mERROR:\x1b[0m {}", message); // Red color
}

pub fn display_success(message: &str) {
    println!("\x1b[32m✓\x1b[0m {}", message); // Green color
}

pub fn display_status(message: &str) {
    println!("\x1b[33m→\x1b[0m {}", message); // Yellow color
}

pub fn display_commit_analysis(commit_messages: &[String], branch_name: &str) {
    println!(
        "\n\x1b[1mAnalyzing commits on branch '{}'\x1b[0m",
        branch_name
    );
    println!("\x1b[4mLast {} commits:\x1b[0m", commit_messages.len());

    for (i, message) in commit_messages.iter().take(10).enumerate() {
        let short_msg = if message.len() > 60 {
            &message[..60]
        } else {
            message
        };
        println!("  {}. {}", i + 1, short_msg);
    }

    if commit_messages.len() > 10 {
        println!("  ... and {} more commits", commit_messages.len() - 10);
    }
}

pub fn display_proposed_tag(old_tag: Option<&str>, new_tag: &str) {
    match old_tag {
        Some(old) => {
            println!("\n\x1b[1mProposed Tag Change:\x1b[0m");
            println!("  From: \x1b[31m{}\x1b[0m", old);
            println!("  To:   \x1b[32m{}\x1b[0m", new_tag);
        }
        None => {
            println!("\n\x1b[1mInitial Tag:\x1b[0m");
            println!("  New tag: \x1b[32m{}\x1b[0m", new_tag);
        }
    }
}

pub fn select_branch(available_branches: &[String]) -> Result<String> {
    if available_branches.len() == 1 {
        return Ok(available_branches[0].clone());
    }

    println!("\n\x1b[1mAvailable branches for tagging:\x1b[0m");
    for (i, branch) in available_branches.iter().enumerate() {
        println!("  {}. {}", i + 1, branch);
    }

    print!("\nSelect a branch (1-{}): ", available_branches.len());
    std::io::stdout().flush().unwrap(); // Need to import std::io::Write

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let selection = input.trim().parse::<usize>().unwrap_or(0);

    if selection > 0 && selection <= available_branches.len() {
        Ok(available_branches[selection - 1].clone())
    } else {
        Err(anyhow::anyhow!("Invalid selection"))
    }
}

pub fn confirm_action(prompt: &str) -> Result<bool> {
    print!("\n{} (y/N): ", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let response = input.trim().to_lowercase();
    Ok(response == "y" || response == "yes")
}

pub fn display_available_branches(branches: &[String]) {
    println!("\x1b[1mConfigured branches:\x1b[0m");
    for branch in branches {
        println!("  - {}", branch);
    }
}

/// Validates that a tag matches the configured pattern.
///
/// # Arguments
/// * `tag` - The tag to validate (e.g., "v1.2.3")
/// * `pattern` - The pattern template (e.g., "v{version}" or "release-v{version}-final")
///
/// # Returns
/// * `Ok(())` if the tag matches the pattern or pattern has no {version} constraint
/// * `Err(anyhow::Error)` if the tag doesn't match the pattern
///
/// # Examples
///
/// ```ignore
/// // Simple pattern with version
/// validate_tag_format("v1.2.3", "v{version}") // Ok
/// validate_tag_format("1.2.3", "v{version}")   // Err - missing prefix
///
/// // Pattern with suffix
/// validate_tag_format("v1.2.3-release", "v{version}-release") // Ok
/// validate_tag_format("v1.2.3", "v{version}-release")         // Err - missing suffix
///
/// // Pattern without {version} constraint
/// validate_tag_format("anything", "free-form") // Ok
/// ```
#[allow(dead_code)]
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

/// Display a boundary warning to the user.
///
/// # Arguments
/// * `warning` - The boundary warning to display
pub fn display_boundary_warning(warning: &BoundaryWarning) {
    eprintln!("\x1b[33m⚠ WARNING:\x1b[0m {}", warning);
}
