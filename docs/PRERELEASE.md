# Pre-Release Versions Guide

This guide explains how to use the pre-release version feature in git-publish.

## Table of Contents

1. [What are Pre-Release Versions?](#what-are-pre-release-versions)
2. [Configuration](#configuration)
3. [Usage Examples](#usage-examples)
4. [Version Bump Behavior](#version-bump-behavior)
5. [Iteration Numbers](#iteration-numbers)
6. [Common Workflows](#common-workflows)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)

## What are Pre-Release Versions?

Pre-release versions follow [Semantic Versioning (semver.org)](https://semver.org/) specification:

```
MAJOR.MINOR.PATCH[-PRERELEASE]
  ↓      ↓      ↓        ↓
  1      2      3    - beta.1
```

The pre-release part comes after a hyphen and indicates the software is not ready for production.

### Built-in Pre-Release Types

- **alpha** (or **a**): Early testing phase, features incomplete
- **beta** (or **b**): Feature complete, testing for bugs
- **rc** (release candidate): Final testing before stable release
- **custom**: Any alphanumeric identifier you define (e.g., "dev", "staging", "internal")

### Examples

```
v1.0.0-alpha       # First alpha of v1.0.0
v1.0.0-beta.1      # First beta iteration
v1.0.0-rc.2        # Second release candidate
v1.0.0             # Final stable release
v2.0.0-staging.5   # Custom identifier with iteration
```

## Configuration

### Enable Pre-Release Support

Create or edit `gitpublish.toml`:

```toml
[prerelease]
enabled = true
default_identifier = "beta"
auto_increment = true
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `false` | Enable/disable pre-release support |
| `default_identifier` | string | `"alpha"` | Default pre-release type if not specified |
| `auto_increment` | bool | `true` | Auto-increment iteration number on subsequent releases |

### Examples

**Development Team (frequent releases)**:
```toml
[prerelease]
enabled = true
default_identifier = "dev"
auto_increment = true
```

**Beta Program (controlled rollout)**:
```toml
[prerelease]
enabled = true
default_identifier = "beta"
auto_increment = true
```

**Release Candidates Only**:
```toml
[prerelease]
enabled = true
default_identifier = "rc"
auto_increment = true
```

## Usage Examples

### Example 1: Alpha Development Cycle

```bash
# Start v1.0.0 development
$ git-publish
# Creates tag: v1.0.0-alpha

# Add features, fix bugs
$ git commit -m "feat(core): initial implementation"
$ git-publish
# Creates tag: v1.0.0-alpha.1 (auto-incremented)

$ git commit -m "fix(core): bug fix"
$ git-publish
# Creates tag: v1.0.0-alpha.2
```

### Example 2: Beta Phase

```bash
# Move to beta after feature freeze
$ git commit -m "feat(ui): complete ui"

# Create beta (switch from alpha)
# Edit gitpublish.toml or use `--prerelease beta`
$ git-publish
# Creates tag: v1.0.0-beta

# Continue bug fixes
$ git commit -m "fix(ui): styling issue"
$ git-publish
# Creates tag: v1.0.0-beta.1

$ git commit -m "fix(api): edge case"
$ git-publish
# Creates tag: v1.0.0-beta.2
```

### Example 3: Release Candidate Phase

```bash
# No new features, only critical bugs
$ git commit -m "fix: critical security issue"

# Move to RC
$ git-publish
# Creates tag: v1.0.0-rc

# Final polish
$ git commit -m "fix: minor issue"
$ git-publish
# Creates tag: v1.0.0-rc.1

# Ready for production
$ git commit -m "docs: release notes for v1.0.0"
$ git-publish
# Creates tag: v1.0.0-rc.2
```

### Example 4: Stable Release

```bash
# After RC approval, release stable
$ git commit -m "chore: release v1.0.0"

# Use major/minor/patch bump with stable version
$ git-publish
# Creates tag: v1.0.0 (no pre-release suffix)

# Now start next version
$ git commit -m "feat: new feature for v1.1.0"
$ git-publish
# Creates tag: v1.1.0-alpha (or your default)
```

## Version Bump Behavior

### How Bumping Works

Pre-release versions use the same bump type as conventional commits:

```
┌─────────────────────────────────────────┐
│ Current Version: v1.0.0-beta.1          │
└────────────────┬────────────────────────┘
                 │
        ┌────────▼────────┐
        │ New Commits     │
        └────────┬────────┘
                 │
        ┌────────▼─────────────────┐
        │ Analyze Commit Types     │
        └────────┬─────────────────┘
                 │
        ┌────────▼─────────────────────────┐
        │ Determine Bump Type               │
        ├─────────────────────────────────┤
        │ Breaking? → Major                │
        │ Features? → Minor                │
        │ Otherwise → Patch (increment)    │
        └────────┬─────────────────────────┘
                 │
        ┌────────▼──────────────────────────┐
        │ Apply Bump with auto_increment    │
        └────────┬──────────────────────────┘
                 │
        ┌────────▼──────────────────────────┐
        │ New Version: v1.0.0-beta.2        │
        │ (iteration incremented)           │
        └────────────────────────────────────┘
```

### Examples by Commit Type

From `v1.0.0-beta.1`:

| Commits | Result | Explanation |
|---------|--------|-------------|
| `fix: bug` | `v1.0.0-beta.2` | Iteration incremented, same pre-release |
| `feat: new` | `v1.0.0-beta.2` | Iteration incremented (feature doesn't bump with pre-release) |
| `fix!: breaking` | `v1.1.0-beta` | Minor bump, iteration reset |

From `v2.0.0-rc.3`:

| Commits | Result | Explanation |
|---------|--------|-------------|
| `fix: bug` | `v2.0.0-rc.4` | Keep pre-release type, increment iteration |
| `feat: new` | `v2.0.0-rc.4` | Keep pre-release type, increment iteration |
| `feat!: breaking` | `v3.0.0-alpha` | Major bump resets to default identifier |

## Iteration Numbers

### Auto-Increment Behavior

When `auto_increment = true`:

```
v1.0.0-beta        (no iteration)
    ↓ (any commit)
v1.0.0-beta.1      (iteration becomes 1)
    ↓ (any commit)
v1.0.0-beta.2      (iteration increments)
    ↓ (any commit)
v1.0.0-beta.3      (iteration increments)
```

### Manual Iteration Control

If `auto_increment = false`, you manage iteration numbers manually:

```
v1.0.0-beta.1
v1.0.0-beta.1      (stays same, user increments manually)
v1.0.0-beta.2      (user sets to .2)
```

## Common Workflows

### Workflow 1: Daily Development Releases

**Setup**:
```toml
[prerelease]
enabled = true
default_identifier = "dev"
auto_increment = true
```

**Process**:
```bash
# Day 1
$ git commit -m "feat: search ui"
$ git-publish          # → v1.0.0-dev

# Day 2
$ git commit -m "fix: search filtering"
$ git-publish          # → v1.0.0-dev.1

# Day 3
$ git commit -m "feat: advanced filters"
$ git-publish          # → v1.0.0-dev.2

# Day 4 (release to beta)
$ git commit -m "chore: feature complete"
$ git-publish --prerelease=beta  # → v1.0.0-beta
```

### Workflow 2: Staged Release (Alpha → Beta → RC → Stable)

**Configuration**:
```toml
[prerelease]
enabled = true
default_identifier = "alpha"
auto_increment = true
```

**Process**:

```bash
# ALPHA PHASE (1-2 weeks)
$ git-publish          # v1.0.0-alpha (feature development)
$ git-publish          # v1.0.0-alpha.1
$ git-publish          # v1.0.0-alpha.2

# BETA PHASE (1-2 weeks)
$ git-publish --prerelease=beta  # v1.0.0-beta (feature freeze)
$ git-publish          # v1.0.0-beta.1 (bug fixes only)
$ git-publish          # v1.0.0-beta.2

# RC PHASE (3-5 days)
$ git-publish --prerelease=rc   # v1.0.0-rc (final testing)
$ git-publish          # v1.0.0-rc.1 (critical fixes only)

# STABLE RELEASE
$ git-publish --stable  # v1.0.0 (remove pre-release)

# Start next version
$ git-publish          # v1.1.0-alpha (or v2.0.0-alpha depending on commits)
```

### Workflow 3: Parallel Branches

```bash
# main branch: stable releases only
# develop branch: pre-release for next version
# release/v1.0 branch: bug fixes for stable releases

# On develop:
$ git commit -m "feat: new feature"
$ git-publish          # v1.1.0-alpha

# On release/v1.0 (bug fix for current stable):
$ git commit -m "fix: critical bug"
$ git-publish          # v1.0.1 (stable patch)

# Merge release/v1.0 back to develop:
$ git merge release/v1.0
$ git-publish          # v1.1.0-alpha.1 (bump after merge)
```

## Best Practices

### 1. Use Standard Identifiers When Possible
✅ **Recommended**: alpha, beta, rc  
❌ **Avoid**: snapshot, nightly, unstable

**Why**: Standard identifiers are widely recognized and tool-compatible.

### 2. Automate the Progression
✅ **Recommended**: Enable `auto_increment = true`  
❌ **Avoid**: Manual iteration number management

**Why**: Reduces human error and ensures consistent versioning.

### 3. Freeze Features at Beta
✅ **Recommended**: 
- Alpha: All features
- Beta: Bug fixes only
- RC: Critical fixes only

❌ **Avoid**: Adding features during beta/RC phases

**Why**: Pre-release phases should narrow in scope.

### 4. Use Semantic Versioning for Core Version
✅ **Recommended**:
```
v1.0.0-alpha    (initial)
v1.0.0-beta     (features frozen)
v1.0.0-rc       (testing complete)
v1.0.0          (stable)
v1.1.0-alpha    (next version)
```

❌ **Avoid**:
```
v1.0-alpha      (incomplete semver)
v1-beta         (too short)
v1.0.0.0-alpha  (too many parts)
```

### 5. Document Phase Transitions
Create tags with meaningful commit messages:

```bash
# Mark phase transitions
$ git commit --allow-empty -m "chore: enter beta phase"
$ git-publish --prerelease=beta
```

### 6. Clean Up Pre-Release Tags Before Stable
Before releasing v1.0.0, clean up old pre-release tags:

```bash
# List pre-release tags
$ git tag -l 'v1.0.0-*'

# Delete old pre-release tags (after validation)
$ git tag -d v1.0.0-alpha v1.0.0-alpha.1 v1.0.0-beta v1.0.0-beta.1 v1.0.0-rc
$ git push origin --delete v1.0.0-alpha v1.0.0-alpha.1 ...

# Now release stable
$ git-publish
# → v1.0.0
```

## Troubleshooting

### Issue: Pre-release tags not created

**Cause**: Pre-release support disabled or not configured  
**Solution**: 
```toml
[prerelease]
enabled = true
```

### Issue: Version jumps to minor instead of iteration

**Cause**: Commits contain new features (feat)  
**Solution**: Only include fixes during pre-release phases
```bash
# During beta, only:
$ git commit -m "fix: bug"        # OK
# Not:
$ git commit -m "feat: new thing" # Will bump version
```

### Issue: Iteration number doesn't increment

**Cause**: `auto_increment = false`  
**Solution**: Either enable auto_increment or manually specify:
```toml
[prerelease]
auto_increment = true
```

### Issue: Can't switch pre-release types

**Cause**: Version bump indicates major version change  
**Solution**: Switch types only if it aligns with version bump
```bash
# OK: Switch with major bump
$ git commit -m "feat!: breaking change"
$ git-publish --prerelease=beta  # v2.0.0-beta ✓

# Problematic: Try to switch without bump
$ git commit -m "fix: minor fix"
$ git-publish --prerelease=rc   # Still v1.0.0-beta.1 ✗
```

### Issue: Tag name not recognized by pattern

**Cause**: Tag format doesn't match branch pattern  
**Solution**: Verify `gitpublish.toml` and tag pattern
```toml
[branches]
main = "v{version}"       # Expects: v1.0.0-beta

# Check actual tags:
$ git tag -l | grep beta
```

### Issue: Pre-release version appears in stable builds

**Cause**: Deploying pre-release tag  
**Solution**: Always deploy stable (non-pre-release) tags
```bash
# ✓ Correct
$ git checkout v1.0.0         # Stable

# ✗ Wrong
$ git checkout v1.0.0-rc      # Pre-release
```

---

**Guide Version**: 1.0  
**Last Updated**: 2025-01-23
