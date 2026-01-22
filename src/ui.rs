use anyhow::Result;
use std::io::{self, Write};

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
