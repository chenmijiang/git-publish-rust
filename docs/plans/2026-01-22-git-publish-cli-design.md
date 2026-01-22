# Git Publish CLI Tool - Design Document

## Overview

A Rust CLI tool that creates and pushes git tags based on configured branch-to-tag-pattern mappings and conventional commit analysis. The tool reads a `gitpublish.toml` configuration to determine which branches can be tagged and how to format the tags.

## Configuration System

### GitPublish TOML Schema

The tool will use a `gitpublish.toml` configuration file with the following structure:

```toml
[branches]
# Maps branch names to tag patterns
main = "v{version}"
develop = "d{version}"
gray = "g{version}"

[patterns]
# Defines version formats
version_format = { major = "{major}.{minor}.{patch}", minor = "{major}.{minor}.{patch}", patch = "{major}.{minor}.{patch}" }

[conventional_commits]
types = ["feat", "fix", "docs", "style", "refactor", "test", "chore", "build", "ci", "perf"]
breaking_change_indicators = ["BREAKING CHANGE:", "BREAKING-CHANGE:"]
major_keywords = ["breaking", "deprecate"]
minor_keywords = ["feature", "feat", "enhancement"]
```

### Configuration Resolution

1. Checks for `./gitpublish.toml` in the current git repository root
2. Falls back to `~/.gitpublish.toml` if no local config exists
3. Default configuration if no config files exist

## Core Functionality

### Branch Tagging Process

1. **Configuration Loading**: Load and parse `gitpublish.toml`
2. **Branch Selection**: Present configured branches to user for selection
3. **Tag Analysis**: Determine the last tag on the selected branch
4. **Commit Parsing**: Analyze commits between last tag and HEAD for conventional commit patterns
5. **Version Bumping**: Calculate new version based on commit types (major/minor/patch)
6. **Preview**: Show user the commits since last tag and proposed new tag
7. **Confirmation**: Ask for user confirmation before proceeding
8. **Tag Creation**: Create the new tag locally
9. **Remote Push**: Push the tag to the remote repository

### Version Calculation Logic

The tool will parse commits using conventional commit standards:

- **Major version bump**: Found breaking changes (`BREAKING CHANGE:` in footer) or major keywords
- **Minor version bump**: Found `feat` commits or minor keywords  
- **Patch version bump**: Found `fix`, `docs`, `style`, `refactor`, `test`, `build`, `ci`, `perf`, or `chore` commits

### Command Structure

```
git-publish [OPTIONS]

Options:
  -c, --config <FILE>     Custom configuration file path
  -b, --branch <BRANCH>   Explicitly specify branch to tag (bypasses selection)
  -f, --force            Skip confirmation prompts
  -n, --dry-run          Preview what would happen without making changes
  --list                 Show available configured branches and exit
  -h, --help             Print help information
  -V, --version          Print version information
```

## Technical Implementation

### Dependencies

- `clap` - CLI argument parsing
- `toml` - Configuration file parsing
- `serde` - Configuration deserialization
- `git2` - Git operations
- `semver` - Semantic version handling
- `console` - Colored output and user interaction

### Main Components

#### 1. Configuration Module (`config.rs`)
Handles loading, parsing, and validating the `gitpublish.toml` configuration.

#### 2. Git Operations Module (`git_ops.rs`)
Manages all git interactions including:
- Retrieving repository information
- Finding tags on specific branches
- Getting commits between tags
- Creating and pushing tags

#### 3. Conventional Commits Parser (`conventional.rs`)
Analyzes commit messages to determine version bump type based on:
- Commit types (feat, fix, etc.)
- Breaking change indicators
- Keywords in commit subjects

#### 4. Version Calculator (`version.rs`)
Calculates new version numbers based on the analysis from conventional commits.

#### 5. User Interface (`ui.rs`)
Handles user interaction including:
- Presenting branch selection menu
- Showing commit analysis and proposed tags
- Requesting confirmations
- Providing visual feedback

### Error Handling

The tool will gracefully handle common scenarios:
- Repository not found
- No configured branches
- No tags exist yet on a branch
- Git operations failing
- Invalid configuration
- Network issues during push

## User Experience Flow

1. **Startup**: Detect git repository and load configuration
2. **Branch Selection**: Display configured branches, allow user to select
3. **Analysis**: Analyze commits on selected branch since last tag
4. **Preview**: Show commits and calculated new tag, ask for confirmation
5. **Execution**: Create and push tag with progress feedback
6. **Completion**: Confirm successful tag creation and push

## Security Considerations

- Validate all configuration values to prevent injection attacks
- Sanitize tag names to follow git naming conventions
- Confirm before pushing tags to remote repository
- Support dry-run mode to preview actions