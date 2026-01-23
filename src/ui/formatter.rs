//! Pure formatting functions for UI output.
//!
//! This module contains all display/formatting logic separated from user interaction.
//! Functions here are pure (no I/O side effects beyond printing) and testable.

use crate::boundary::BoundaryWarning;

/// Format and print an error message in red.
pub fn display_error(message: &str) {
    eprintln!("\x1b[31mERROR:\x1b[0m {}", message);
}

/// Format and print a success message with green checkmark.
pub fn display_success(message: &str) {
    println!("\x1b[32m✓\x1b[0m {}", message);
}

/// Format and print a status message with yellow arrow.
pub fn display_status(message: &str) {
    println!("\x1b[33m→\x1b[0m {}", message);
}

/// Display commit analysis for a branch.
///
/// Shows the branch name and up to 10 commits from the provided list.
/// If more than 10 commits exist, displays count of remaining commits.
///
/// # Arguments
/// * `commit_messages` - List of commit messages to display
/// * `branch_name` - The name of the branch being analyzed
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

/// Display the proposed tag change (or initial tag).
///
/// Shows either:
/// - If updating: "From: old_tag -> To: new_tag"
/// - If initial: "Initial Tag: new_tag"
///
/// # Arguments
/// * `old_tag` - Previous tag (None if this is the initial tag)
/// * `new_tag` - The new tag being proposed
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

/// Display a boundary warning to the user.
///
/// Shows a yellow warning icon followed by the warning message.
///
/// # Arguments
/// * `warning` - The boundary warning to display
pub fn display_boundary_warning(warning: &BoundaryWarning) {
    eprintln!("\x1b[33m⚠ WARNING:\x1b[0m {}", warning);
}

/// Display available branches configured for tagging.
///
/// # Arguments
/// * `branches` - List of branch names to display
pub fn display_available_branches(branches: &[String]) {
    println!("\x1b[1mConfigured branches:\x1b[0m");
    for branch in branches {
        println!("  - {}", branch);
    }
}

/// Display manual push instruction for a tag.
///
/// Shows the git command needed to push the tag to a remote.
///
/// # Arguments
/// * `tag` - The tag that was created locally
/// * `remote` - The remote name (e.g., "origin")
pub fn display_manual_push_instruction(tag: &str, remote: &str) {
    println!(
        "\n\x1b[33m→\x1b[0m To push this tag later, run:\n  \x1b[36mgit push {} {}\x1b[0m",
        remote, tag
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_error() {
        // Visual verification test - output is printed to stderr
        display_error("test error");
    }

    #[test]
    fn test_display_success() {
        // Visual verification test - output is printed to stdout
        display_success("test success");
    }

    #[test]
    fn test_display_status() {
        // Visual verification test - output is printed to stdout
        display_status("test status");
    }
}
