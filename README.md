# Git Publish

A Rust CLI tool for creating and pushing git tags based on configurable branch-to-tag-pattern mappings and conventional commit analysis.

## Features

- Configurable branch-to-tag-pattern mappings via `gitpublish.toml`
- Automatic semantic version bump detection based on conventional commits
- Interactive branch selection
- Commit analysis and preview before tagging
- Dry-run mode for testing
- Confirmation prompts before creating and pushing tags

## Installation

First, make sure you have Rust and Cargo installed. Then:

```bash
# Clone the repository
git clone https://github.com/your-repo/git-publish.git
cd git-publish

# Build the project
cargo build --release

# The executable will be available at target/release/git-publish
./target/release/git-publish --help
```

Alternatively, if you have this project as a local directory, you can run:

```bash
# Run directly with cargo
cargo run -- --help

# Or build and run
cargo build --release
./target/release/git-publish --help
```

## Configuration

Create a `gitpublish.toml` file in your repository root or in your home directory (`~/.gitpublish.toml`). An example configuration is provided in `gitpublish.toml.example`.

Example configuration:

```toml
[branches]
main = "v{version}"
develop = "d{version}"
gray = "g{version}"

[conventional_commits]
types = ["feat", "fix", "docs", "style", "refactor", "test", "chore", "build", "ci", "perf"]
breaking_change_indicators = ["BREAKING CHANGE:", "BREAKING-CHANGE:"]
major_keywords = ["breaking", "deprecate"]
minor_keywords = ["feature", "feat", "enhancement"]
```

## Usage

```bash
# Interactive mode - select branch to tag
git-publish

# Specify branch directly
git-publish --branch main

# Dry run - preview what would happen
git-publish --dry-run

# Skip confirmation prompts
git-publish --force

# Use custom configuration file
git-publish --config /path/to/config.toml

# List configured branches
git-publish --list

# Show help
git-publish --help

# Show version
git-publish --version
```

## Conventional Commit Detection

The tool analyzes commits using conventional commit format to determine version bumps:

- **Major version bump**: Commits with `BREAKING CHANGE:` in the footer or commit type with `!` (e.g., `feat!: ...`)
- **Minor version bump**: `feat` type commits
- **Patch version bump**: `fix` and other types of commits

## Options

- `-c, --config <FILE>`: Custom configuration file path
- `-b, --branch <BRANCH>`: Explicitly specify branch to tag (bypasses selection)
- `-f, --force`: Skip confirmation prompts
- `-n, --dry-run`: Preview what would happen without making changes
- `--list`: Show available configured branches and exit
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for your changes
5. Run the test suite (`cargo test`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.