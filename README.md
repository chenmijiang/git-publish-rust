# Git Publish

A Rust CLI tool for creating and pushing git tags based on configurable branch-to-tag-pattern mappings and conventional commit analysis.

## Features

- Configurable branch-to-tag-pattern mappings via `gitpublish.toml`
- Automatic semantic version bump detection based on conventional commits
- Interactive branch and remote selection
- Multi-remote support with CLI override
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

Create a `gitpublish.toml` file in your repository root or home directory (`~/.gitpublish.toml`). See `gitpublish.toml.example` for a complete example.

```toml
[branches]
main = "v{version}"
develop = "d{version}"

[conventional_commits]
types = ["feat", "fix", "docs", "style", "refactor", "test", "chore", "build", "ci", "perf"]
breaking_change_indicators = ["BREAKING CHANGE:", "BREAKING-CHANGE:"]
major_keywords = ["breaking", "deprecate"]
minor_keywords = ["feature", "feat"]

[behavior]
skip_remote_selection = false  # Auto-select single remote without prompting
```

### Configuration Options

**`[behavior] skip_remote_selection`** (boolean, default: `false`)  
When `true` and the repository has only one remote, git-publish automatically selects it without prompting.

## Usage

```bash
# Interactive mode
git-publish

# Specify branch directly
git-publish --branch main

# Specify remote directly
git-publish --remote origin
git-publish -r upstream

# Dry run - preview without making changes
git-publish --dry-run

# Skip confirmation prompts
git-publish --force

# Show help / version
git-publish --help
git-publish --version
```

## Conventional Commit Detection

The tool analyzes commits using conventional commit format to determine version bumps:

- **Major version bump**: Commits with `BREAKING CHANGE:` in the footer or commit type with `!` (e.g., `feat!: ...`)
- **Minor version bump**: `feat` type commits
- **Patch version bump**: `fix` and other types of commits

## Options

| Flag | Description |
|------|-------------|
| `-b, --branch <BRANCH>` | Explicitly specify branch to tag |
| `-r, --remote <REMOTE>` | Specify which git remote to use |
| `-f, --force` | Skip confirmation prompts |
| `-n, --dry-run` | Preview without making changes |
| `-c, --config <FILE>` | Custom configuration file path |
| `-h, --help` | Show help information |
| `-V, --version` | Show version information |

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