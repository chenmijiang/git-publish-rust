use anyhow::Result;
use clap::Parser;

use boundary::BoundaryWarning;

mod boundary;
mod config;
mod conventional;
mod git_ops;
mod ui;
mod version;

#[derive(clap::Parser)]
#[command(
    name = "git-publish",
    about = "Create and push git tags based on conventional commits"
)]
struct Args {
    #[arg(short, long, help = "Custom configuration file path")]
    config: Option<String>,

    #[arg(short, long, help = "Explicitly specify branch to tag")]
    branch: Option<String>,

    #[arg(short, long, help = "Skip confirmation prompts")]
    force: bool,

    #[arg(long, help = "Preview what would happen without making changes")]
    dry_run: bool,

    #[arg(long, help = "Show available configured branches and exit")]
    list: bool,

    #[arg(short, long, help = "Print version information")]
    version: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.version {
        println!("git-publish {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.list {
        list_configured_branches(args.config.as_deref())?;
        return Ok(());
    }

    // Load configuration
    let config = match config::load_config(args.config.as_deref()) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    // Select branch to tag
    let branch_to_tag = if let Some(branch) = args.branch {
        branch
    } else {
        // Get configured branches as a vector
        let configured_branches: Vec<String> = config.branches.keys().cloned().collect();
        if configured_branches.is_empty() {
            ui::display_error("No branches configured for tagging in gitpublish.toml");
            std::process::exit(1);
        }

        ui::select_branch(&configured_branches)?
    };

    // Verify the selected branch exists in config
    if !config.branches.contains_key(&branch_to_tag) {
        eprintln!(
            "Error: Branch '{}' is not configured for tagging",
            branch_to_tag
        );
        std::process::exit(1);
    }

    // Initialize git operations
    let git_repo = match git_ops::GitRepo::new() {
        Ok(repo) => repo,
        Err(e) => {
            ui::display_error(&format!("Git repository error: {}", e));
            std::process::exit(1);
        }
    };

    // Fetch latest from remote to ensure we have the latest tags and commits
    ui::display_status("Fetching latest data from remote...");
    match git_repo.fetch_from_remote("origin") {
        Ok(_) => {
            ui::display_success("Successfully fetched latest data from remote");
        }
        Err(e) => {
            // Check if it's an authentication error
            let error_msg = e.to_string();
            if error_msg.contains("auth")
                || error_msg.contains("Auth")
                || error_msg.contains("permission")
                || error_msg.contains("Permission")
            {
                let warning = BoundaryWarning::FetchAuthenticationFailed {
                    remote: "origin".to_string(),
                };
                ui::display_boundary_warning(&warning);

                if !args.force
                    && !args.dry_run
                    && !ui::confirm_action("Continue using local data?")?
                {
                    println!("Operation cancelled by user.");
                    return Ok(());
                }
            } else {
                // Non-auth errors are still warnings
                ui::display_status(&format!(
                    "Warning: Could not fetch from remote: {}. Using local branch data.",
                    e
                ));
            }
        }
    }

    // Get the latest tag on the selected branch
    let latest_tag = match git_repo.get_latest_tag_on_branch(&branch_to_tag) {
        Ok(tag) => tag,
        Err(e) => {
            ui::display_error(&format!(
                "Failed to get latest tag on branch '{}': {}",
                branch_to_tag, e
            ));
            std::process::exit(1);
        }
    };

    // Get commits since the latest tag
    let commits = match git_repo.get_commits_since_tag(&branch_to_tag, latest_tag.as_deref()) {
        Ok(commits) => commits,
        Err(e) => {
            ui::display_error(&format!(
                "Failed to get commits since tag on branch '{}': {}",
                branch_to_tag, e
            ));
            std::process::exit(1);
        }
    };

    // Extract commit messages for analysis
    let commit_messages: Vec<String> = commits
        .iter()
        .filter_map(|commit| commit.message().map(|msg| msg.to_string()))
        .collect();

    if commits.is_empty() {
        let head_hash = git_repo.get_current_head_hash()?;
        let warning = BoundaryWarning::NoNewCommits {
            latest_tag: latest_tag.clone().unwrap_or_else(|| "unknown".to_string()),
            current_commit_hash: head_hash,
        };

        ui::display_boundary_warning(&warning);

        if !args.force && !args.dry_run && !ui::confirm_action("Continue with no new commits?")? {
            println!("Operation cancelled by user.");
            return Ok(());
        }
    }

    // Display commit analysis
    ui::display_commit_analysis(&commit_messages, &branch_to_tag);

    // Determine the version bump based on commits
    let version_bump =
        conventional::determine_version_bump(&commit_messages, &config.conventional_commits);

    // Calculate the new version
    let new_version = match latest_tag.as_ref() {
        Some(tag) => {
            if let Some(current_version) = version::parse_version_from_tag(tag) {
                version::bump_version(current_version, &version_bump)
            } else {
                // Unable to parse tag - display warning
                let warning = BoundaryWarning::UnparsableTag {
                    tag: tag.clone(),
                    reason: "Version number format not recognized".to_string(),
                };
                ui::display_boundary_warning(&warning);

                if !args.force
                    && !args.dry_run
                    && !ui::confirm_action("Use initial version v0.1.0 and continue?")?
                {
                    println!("Operation cancelled by user.");
                    return Ok(());
                }

                version::Version::new(0, 1, 0)
            }
        }
        None => {
            // If no tag exists, start with 0.1.0
            version::Version::new(0, 1, 0)
        }
    };

    // Format the new tag using the configured pattern
    let new_tag_pattern = config
        .branches
        .get(&branch_to_tag)
        .cloned()
        .unwrap_or_else(|| "v{version}".to_string());
    let new_tag = new_tag_pattern.replace("{version}", &new_version.to_string());

    // Display the proposed tag
    ui::display_proposed_tag(latest_tag.as_deref(), &new_tag);

    // Get user's tag selection (use default, customize, or edit)
    let final_tag = if !args.force && !args.dry_run {
        ui::select_or_customize_tag(&new_tag, &new_tag_pattern)?
    } else {
        new_tag.clone()
    };

    // Confirm tag use (checks format and gets user confirmation)
    if !args.force && !args.dry_run && !ui::confirm_tag_use(&final_tag, &new_tag_pattern)? {
        println!("Tag creation cancelled by user.");
        return Ok(());
    }

    if args.dry_run {
        ui::display_status("Dry run模式:");
        ui::display_success(&format!("  Step 1: 将创建本地tag: {}", final_tag));
        ui::display_success("  Step 2: 将询问是否推送tag到远程");
        ui::display_success(&format!("  Step 3: (可选) 推送 {} 到 origin", final_tag));
        return Ok(());
    }

    // Create the tag
    ui::display_status(&format!("Creating tag: {}", final_tag));
    if let Err(e) = git_repo.create_tag(&final_tag) {
        ui::display_error(&format!("Failed to create tag '{}': {}", final_tag, e));
        std::process::exit(1);
    }
    ui::display_success(&format!("Created tag: {}", final_tag));

    // Step 2: Ask user whether to push the tag
    let should_push = if !args.force {
        ui::confirm_push_tag(&final_tag, "origin")?
    } else {
        true // In force mode, push automatically
    };

    // Step 3: Push if user confirmed (or in force mode)
    if should_push {
        ui::display_status(&format!("Pushing tag: {} to remote", final_tag));
        if let Err(e) = git_repo.push_tag(&final_tag) {
            ui::display_error(&format!("Failed to push tag '{}': {}", final_tag, e));
            std::process::exit(1);
        }
        ui::display_success(&format!("Pushed tag: {} to remote", final_tag));

        println!(
            "\n\x1b[32m✓\x1b[0m Successfully published tag {} for branch {}\n",
            final_tag, branch_to_tag
        );
    } else {
        // Tag created locally, but not pushed
        ui::display_manual_push_instruction(&final_tag, "origin");

        println!(
            "\n\x1b[32m✓\x1b[0m Tag {} created locally for branch {}\n",
            final_tag, branch_to_tag
        );
    }

    Ok(())
}

fn list_configured_branches(config_path: Option<&str>) -> Result<()> {
    let config = match config::load_config(config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };
    let branches: Vec<String> = config.branches.keys().cloned().collect();

    if branches.is_empty() {
        ui::display_error("No branches configured for tagging in gitpublish.toml");
        std::process::exit(1);
    }

    ui::display_available_branches(&branches);
    Ok(())
}
