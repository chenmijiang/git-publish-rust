# Configuration

## Behavior Configuration

The `[behavior]` section controls how git-publish handles interactive prompts.

### skip_remote_selection

**Type:** boolean  
**Default:** `false`  
**Description:** When set to `true`, if the repository has only a single remote, 
git-publish will automatically select it without prompting the user.

When multiple remotes exist, the prompt is shown regardless of this setting.

**Example:**
```toml
[behavior]
skip_remote_selection = true
```

## CLI Arguments

### --remote / -r

**Usage:** `git-publish --remote <REMOTE_NAME>`  
**Description:** Specify which git remote to fetch from and push to. 
Bypasses both the interactive prompt and the `skip_remote_selection` config option.

The specified remote must exist in the repository.

**Examples:**
```bash
git-publish --remote origin           # Use origin
git-publish -r upstream              # Use upstream (short form)
```

**Precedence:** When both `--remote` flag and config are present, the CLI flag takes precedence.

### Error Handling

If you specify a remote that doesn't exist:
```
Error: Remote 'invalid' not found. Available remotes: origin, upstream
```
