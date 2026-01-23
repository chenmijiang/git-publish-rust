# Git-Publish Hooks Guide

This guide explains how to use the hooks system for extensibility in git-publish.

## Table of Contents

1. [Overview](#overview)
2. [Hook Types](#hook-types)
3. [Hook Configuration](#hook-configuration)
4. [Environment Variables](#environment-variables)
5. [Writing Hooks](#writing-hooks)
6. [Examples](#examples)
7. [Error Handling](#error-handling)
8. [Best Practices](#best-practices)

## Overview

Hooks allow you to run custom scripts at key points in the git-publish workflow. This enables:

- **Pre-tag automation**: Validate, build, test before creating tags
- **Post-tag notifications**: Slack messages, webhooks, release notes
- **Post-push notifications**: Deploy, archive, or record version information

### Hook Lifecycle

```
User runs: git-publish
    ↓
Analyze commits
    ↓
Calculate new version
    ↓
PRE-TAG-CREATE hook ──→ Script can PREVENT tag creation ✓/✗
    ↓
Create local tag
    ↓
POST-TAG-CREATE hook ──→ Script runs after tag created (informational)
    ↓
Push tags to remote
    ↓
POST-PUSH hook ──→ Script runs after push complete (informational)
    ↓
✅ Done
```

## Hook Types

### 1. pre-tag-create

**When**: Before creating the tag locally  
**Can Block**: Yes - exit code non-zero prevents tag creation  
**Environment**: Full context available  

**Use Cases**:
- Validate version number format
- Run tests/builds to ensure code quality
- Check for required commit messages
- Verify deployment prerequisites

**Example**:
```bash
#!/bin/bash
# Pre-tag validation hook

# Reject versions that don't pass tests
npm run test || exit 1

# Prevent if uncommitted changes
git diff --quiet || exit 1

# Success - allow tag creation
exit 0
```

### 2. post-tag-create

**When**: After creating tag locally, before pushing  
**Can Block**: No - failures are logged but don't block push  
**Environment**: Full context available  

**Use Cases**:
- Generate release notes
- Build artifacts
- Create GitHub releases
- Update documentation

**Example**:
```bash
#!/bin/bash
# Post-tag release notes

echo "Generated release notes for $GITPUBLISH_TAG_NAME"
echo "Version bump: $GITPUBLISH_VERSION_BUMP"
echo "Commits: $GITPUBLISH_COMMIT_COUNT"
```

### 3. post-push

**When**: After tags pushed to remote  
**Can Block**: No - failures are logged but not fatal  
**Environment**: Limited (success assumed at this point)  

**Use Cases**:
- Deploy to staging/production
- Trigger CI/CD pipelines
- Send notifications
- Update version trackers

**Example**:
```bash
#!/bin/bash
# Post-push notification

curl -X POST https://api.example.com/deploy \
  -d "tag=$GITPUBLISH_TAG_NAME" \
  -d "version_bump=$GITPUBLISH_VERSION_BUMP"
```

## Hook Configuration

### Basic Configuration

In `gitpublish.toml`:

```toml
[hooks]
pre_tag_create = "./scripts/pre-tag.sh"
post_tag_create = "./scripts/post-tag.sh"
post_push = "./scripts/post-push.sh"
```

### Optional Hooks

Any hook can be omitted - only configured hooks are executed:

```toml
[hooks]
pre_tag_create = "./scripts/pre-tag.sh"
# post_tag_create is omitted - will not be executed
# post_push is omitted - will not be executed
```

### Paths

- **Relative paths**: Resolved from repository root
- **Absolute paths**: Used as-is
- **Files must exist**: Error if script file not found
- **Must be executable**: `chmod +x script.sh`

**Examples**:
```toml
# Current directory
pre_tag_create = "./hooks/pre-tag.sh"

# Subdirectory
pre_tag_create = "./git-hooks/pre-tag.sh"

# Absolute path
pre_tag_create = "/opt/hooks/pre-tag.sh"
```

## Environment Variables

All hooks receive environment variables with context:

### Always Available

| Variable | Example | Description |
|----------|---------|-------------|
| `GITPUBLISH_BRANCH` | `main` | Branch being tagged |
| `GITPUBLISH_TAG_NAME` | `v1.2.3` | Tag being created |
| `GITPUBLISH_REMOTE` | `origin` | Remote being pushed to |

### Conditionally Available

| Variable | Availability | Example |
|----------|--------------|---------|
| `GITPUBLISH_VERSION_BUMP` | pre/post-tag | `Major`, `Minor`, `Patch` |
| `GITPUBLISH_COMMIT_COUNT` | pre/post-tag | `5` |

### Accessing in Scripts

**Bash**:
```bash
#!/bin/bash
echo "Creating tag: $GITPUBLISH_TAG_NAME"
echo "On branch: $GITPUBLISH_BRANCH"
echo "Version bump: $GITPUBLISH_VERSION_BUMP"
```

**Python**:
```python
#!/usr/bin/env python3
import os

tag_name = os.environ.get('GITPUBLISH_TAG_NAME')
branch = os.environ.get('GITPUBLISH_BRANCH')
version_bump = os.environ.get('GITPUBLISH_VERSION_BUMP')
```

**Node.js**:
```javascript
#!/usr/bin/env node

const tagName = process.env.GITPUBLISH_TAG_NAME;
const branch = process.env.GITPUBLISH_BRANCH;
const versionBump = process.env.GITPUBLISH_VERSION_BUMP;
```

## Writing Hooks

### Shell Script Template

```bash
#!/bin/bash
# Bash hook template

set -e  # Exit on error
set -u  # Exit on undefined variable

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Log function
log() {
  echo -e "${GREEN}[git-publish-hook]${NC} $1"
}

error() {
  echo -e "${RED}[git-publish-hook] ERROR:${NC} $1" >&2
  exit 1
}

warn() {
  echo -e "${YELLOW}[git-publish-hook] WARN:${NC} $1"
}

# Log available context
log "Hook executing with context:"
log "  Branch: $GITPUBLISH_BRANCH"
log "  Tag: $GITPUBLISH_TAG_NAME"
log "  Remote: $GITPUBLISH_REMOTE"

if [[ -n "${GITPUBLISH_VERSION_BUMP:-}" ]]; then
  log "  Version bump: $GITPUBLISH_VERSION_BUMP"
fi

if [[ -n "${GITPUBLISH_COMMIT_COUNT:-}" ]]; then
  log "  Commits: $GITPUBLISH_COMMIT_COUNT"
fi

# Your hook logic here
if [[ "$GITPUBLISH_TAG_NAME" == "v"* ]]; then
  log "Valid version tag"
else
  error "Invalid tag format"
fi

log "Hook completed successfully"
exit 0
```

### Python Template

```python
#!/usr/bin/env python3
"""Python hook template"""

import os
import sys
import subprocess

def log(msg):
    print(f"[git-publish-hook] {msg}")

def error(msg):
    print(f"[git-publish-hook] ERROR: {msg}", file=sys.stderr)
    sys.exit(1)

def main():
    # Get environment variables
    branch = os.environ.get('GITPUBLISH_BRANCH')
    tag_name = os.environ.get('GITPUBLISH_TAG_NAME')
    remote = os.environ.get('GITPUBLISH_REMOTE')
    version_bump = os.environ.get('GITPUBLISH_VERSION_BUMP')
    commit_count = os.environ.get('GITPUBLISH_COMMIT_COUNT')
    
    log(f"Hook executing with context:")
    log(f"  Branch: {branch}")
    log(f"  Tag: {tag_name}")
    log(f"  Remote: {remote}")
    
    if version_bump:
        log(f"  Version bump: {version_bump}")
    
    if commit_count:
        log(f"  Commits: {commit_count}")
    
    # Your hook logic here
    if not tag_name.startswith('v'):
        error("Invalid tag format")
    
    log("Hook completed successfully")
    return 0

if __name__ == '__main__':
    sys.exit(main())
```

### Node.js Template

```javascript
#!/usr/bin/env node
/**Node.js hook template*/

function log(msg) {
  console.log(`[git-publish-hook] ${msg}`);
}

function error(msg) {
  console.error(`[git-publish-hook] ERROR: ${msg}`);
  process.exit(1);
}

async function main() {
  // Get environment variables
  const branch = process.env.GITPUBLISH_BRANCH;
  const tagName = process.env.GITPUBLISH_TAG_NAME;
  const remote = process.env.GITPUBLISH_REMOTE;
  const versionBump = process.env.GITPUBLISH_VERSION_BUMP;
  const commitCount = process.env.GITPUBLISH_COMMIT_COUNT;
  
  log('Hook executing with context:');
  log(`  Branch: ${branch}`);
  log(`  Tag: ${tagName}`);
  log(`  Remote: ${remote}`);
  
  if (versionBump) {
    log(`  Version bump: ${versionBump}`);
  }
  
  if (commitCount) {
    log(`  Commits: ${commitCount}`);
  }
  
  // Your hook logic here
  if (!tagName.startsWith('v')) {
    error('Invalid tag format');
  }
  
  log('Hook completed successfully');
  return 0;
}

main().catch(error);
```

## Examples

### Example 1: Pre-Tag Validation

```bash
#!/bin/bash
# scripts/pre-tag-create.sh
# Validates that code is ready for tagging

set -e

echo "[pre-tag] Validating code quality..."

# Check for uncommitted changes
if ! git diff --quiet; then
  echo "Error: Uncommitted changes present"
  exit 1
fi

# Run tests
echo "[pre-tag] Running tests..."
npm test || {
  echo "Error: Tests failed"
  exit 1
}

# Run linter
echo "[pre-tag] Linting code..."
npm run lint || {
  echo "Error: Linting failed"
  exit 1
}

echo "[pre-tag] ✓ Code quality checks passed"
exit 0
```

**Configure**:
```toml
[hooks]
pre_tag_create = "./scripts/pre-tag-create.sh"
```

### Example 2: Post-Tag Notification (Slack)

```bash
#!/bin/bash
# scripts/post-tag-create.sh
# Sends Slack notification on tag creation

SLACK_WEBHOOK="${SLACK_WEBHOOK_URL}"

if [[ -z "$SLACK_WEBHOOK" ]]; then
  echo "Warning: SLACK_WEBHOOK_URL not set, skipping notification"
  exit 0
fi

# Build message
MESSAGE="New version released: $GITPUBLISH_TAG_NAME on $GITPUBLISH_BRANCH"
if [[ -n "${GITPUBLISH_VERSION_BUMP:-}" ]]; then
  MESSAGE="$MESSAGE (bump: $GITPUBLISH_VERSION_BUMP)"
fi

# Send to Slack
curl -X POST "$SLACK_WEBHOOK" \
  -H 'Content-Type: application/json' \
  -d "{
    \"text\": \"$MESSAGE\",
    \"attachments\": [
      {
        \"color\": \"good\",
        \"fields\": [
          {\"title\": \"Tag\", \"value\": \"$GITPUBLISH_TAG_NAME\", \"short\": true},
          {\"title\": \"Branch\", \"value\": \"$GITPUBLISH_BRANCH\", \"short\": true},
          {\"title\": \"Remote\", \"value\": \"$GITPUBLISH_REMOTE\", \"short\": true},
          {\"title\": \"Commits\", \"value\": \"$GITPUBLISH_COMMIT_COUNT\", \"short\": true}
        ]
      }
    ]
  }"

exit 0
```

**Configure**:
```toml
[hooks]
post_tag_create = "./scripts/post-tag-create.sh"
```

**Run with**:
```bash
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/T00/B00/XXX"
git-publish
```

### Example 3: Post-Push Deployment

```bash
#!/bin/bash
# scripts/post-push.sh
# Triggers deployment after tags are pushed

set -e

API_ENDPOINT="${DEPLOY_API_ENDPOINT}"

if [[ -z "$API_ENDPOINT" ]]; then
  echo "Warning: DEPLOY_API_ENDPOINT not set, skipping deployment"
  exit 0
fi

echo "[post-push] Triggering deployment for $GITPUBLISH_TAG_NAME"

# Call deployment API
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST \
  "$API_ENDPOINT/deploy" \
  -H "Authorization: Bearer $DEPLOY_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"tag\": \"$GITPUBLISH_TAG_NAME\",
    \"branch\": \"$GITPUBLISH_BRANCH\",
    \"remote\": \"$GITPUBLISH_REMOTE\"
  }")

if [[ $HTTP_CODE -eq 200 ]]; then
  echo "[post-push] ✓ Deployment triggered successfully"
  exit 0
else
  echo "[post-push] ✗ Deployment trigger failed (HTTP $HTTP_CODE)"
  exit 1
fi
```

### Example 4: Release Notes Generation

```python
#!/usr/bin/env python3
# scripts/post-tag-create.py
# Generates release notes

import os
import json
from datetime import datetime

def generate_notes():
    tag_name = os.environ.get('GITPUBLISH_TAG_NAME', 'unknown')
    branch = os.environ.get('GITPUBLISH_BRANCH', 'unknown')
    version_bump = os.environ.get('GITPUBLISH_VERSION_BUMP', 'unknown')
    commit_count = os.environ.get('GITPUBLISH_COMMIT_COUNT', '0')
    
    notes = {
        'version': tag_name,
        'branch': branch,
        'bump_type': version_bump,
        'commits': int(commit_count),
        'created_at': datetime.now().isoformat(),
    }
    
    # Write to file
    with open('RELEASE_NOTES.json', 'a') as f:
        json.dump(notes, f)
        f.write('\n')
    
    print(f"✓ Release notes updated for {tag_name}")

if __name__ == '__main__':
    generate_notes()
```

## Error Handling

### Pre-Tag-Create Errors (Blocking)

Hook exits with non-zero code → Tag creation prevented:

```bash
#!/bin/bash
if [[ some_condition ]]; then
  echo "Error: Cannot create tag"
  exit 1  # ← Non-zero exit prevents tag
fi
exit 0    # ← Zero exit allows tag
```

### Post-Tag-Create Errors (Non-Blocking)

Hook exits with non-zero code → Error logged but push continues:

```bash
#!/bin/bash
# Even if this fails, push will continue
curl https://api.example.com/notify || exit 1
```

**Configuration**: Set `RUST_LOG=debug` to see hook failures:
```bash
RUST_LOG=debug git-publish
```

## Best Practices

### 1. Keep Hooks Focused
✅ **Good**: One responsibility per hook  
❌ **Bad**: Multiple unrelated operations in one hook

```bash
# ✗ Bad: Too many responsibilities
pre_tag_create:
  - run tests
  - build artifacts
  - deploy
  - notify slack

# ✓ Good: Single responsibility
pre_tag_create: validate (tests only)
post_tag_create: build and notify
post_push: deploy
```

### 2. Handle Missing Environment Variables

```bash
#!/bin/bash
# Check for required env vars
TAG_NAME="${GITPUBLISH_TAG_NAME:-}"
if [[ -z "$TAG_NAME" ]]; then
  echo "Error: GITPUBLISH_TAG_NAME not set"
  exit 1
fi
```

### 3. Use Proper Exit Codes
- `0`: Success (allow tag creation if pre-tag)
- `1-255`: Failure (block tag creation if pre-tag)

```bash
if condition_fails; then
  exit 1  # Failure
fi
exit 0   # Success
```

### 4. Log Clearly

```bash
#!/bin/bash
echo "[hook-name] Starting operation"
echo "[hook-name] Processing $GITPUBLISH_TAG_NAME"
echo "[hook-name] ✓ Operation complete"
```

### 5. Make Hooks Version Controllable

```bash
#!/bin/bash
# Allow disabling hooks via environment variable
if [[ "${SKIP_HOOKS:-false}" == "true" ]]; then
  echo "[hook] Skipped (SKIP_HOOKS=true)"
  exit 0
fi
```

### 6. Test Hooks Independently

```bash
# Test pre-tag hook standalone
export GITPUBLISH_BRANCH="main"
export GITPUBLISH_TAG_NAME="v1.0.0"
export GITPUBLISH_REMOTE="origin"
export GITPUBLISH_VERSION_BUMP="Minor"
export GITPUBLISH_COMMIT_COUNT="5"

./scripts/pre-tag-create.sh
```

### 7. Document Required Environment Variables

```bash
#!/bin/bash
# pre-tag-create.sh
# Requires: 
#   - DEPLOY_TOKEN: API token for deployment service
#   - BUILD_COMMAND: Command to build artifacts

if [[ -z "$DEPLOY_TOKEN" ]]; then
  echo "Error: DEPLOY_TOKEN environment variable required"
  exit 1
fi
```

### 8. Set Executable Bit

```bash
chmod +x scripts/pre-tag-create.sh
chmod +x scripts/post-tag-create.sh
chmod +x scripts/post-push.sh
```

---

**Guide Version**: 1.0  
**Last Updated**: 2025-01-23
