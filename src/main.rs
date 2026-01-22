use anyhow::Result;
use clap::Parser;

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
            // Fetch failures are not fatal - proceed with local data
            ui::display_status(&format!(
                "Warning: Could not fetch from remote: {}. Using local branch data.",
                e
            ));
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
        ui::display_status("No new commits found since the last tag.");
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
                // If we can't parse the current tag, start with 0.1.0
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

    // Confirm with the user
    if !args.force && !args.dry_run && !ui::confirm_action("Create and push this tag?")? {
        println!("Tag creation cancelled by user.");
        return Ok(());
    }

    if args.dry_run {
        ui::display_status("Dry run: Would create tag");
        ui::display_success(&format!("Would create tag: {}", new_tag));
        if !args.force {
            ui::display_status("Dry run: Would push tag to remote");
            ui::display_success(&format!("Would push tag: {} to remote", new_tag));
        }
        return Ok(());
    }

    // Create the tag
    ui::display_status(&format!("Creating tag: {}", new_tag));
    if let Err(e) = git_repo.create_tag(&new_tag) {
        ui::display_error(&format!("Failed to create tag '{}': {}", new_tag, e));
        std::process::exit(1);
    }
    ui::display_success(&format!("Created tag: {}", new_tag));

    // Push the tag to remote
    ui::display_status(&format!("Pushing tag: {} to remote", new_tag));
    if let Err(e) = git_repo.push_tag(&new_tag) {
        ui::display_error(&format!("Failed to push tag '{}': {}", new_tag, e));
        if !args.force {
            // If not in force mode, let the user know they can retry
            eprintln!("Tag was created locally but not pushed. You can try pushing manually with: git push origin {}", new_tag);
        }
        std::process::exit(1);
    }
    ui::display_success(&format!("Pushed tag: {} to remote", new_tag));

    println!(
        "\nSuccessfully published tag {} for branch {}",
        new_tag, branch_to_tag
    );

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
