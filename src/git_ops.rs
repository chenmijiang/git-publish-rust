use anyhow::Result;
use git2::{BranchType, Commit, Oid, Repository};

/// Wrapper around git2 Repository for tag and commit operations.
///
/// Provides high-level abstractions for common git operations used by git-publish,
/// including fetching, tagging, pushing, and commit history traversal.
pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    /// Creates a new GitRepo instance for the current working directory.
    ///
    /// Discovers the git repository in the current directory or parent directories.
    ///
    /// # Returns
    /// * `Ok(GitRepo)` - Successfully initialized repository wrapper
    /// * `Err` - If not in a git repository
    pub fn new() -> Result<Self> {
        // Check if we're in a git repository
        let repo = match Repository::discover(".") {
            Ok(repo) => repo,
            Err(e) => return Err(anyhow::anyhow!("Not in a git repository: {}", e)),
        };
        Ok(GitRepo { repo })
    }

    /// Gets all configured remote names from the repository.
    ///
    /// Remotes are sorted with "origin" first (if it exists), followed by others alphabetically.
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - Vector of remote names (e.g., ["origin", "upstream"])
    /// * `Err` - If unable to list remotes
    pub fn list_remotes(&self) -> Result<Vec<String>> {
        let remote_names = self.repo.remotes()?;
        let mut remotes = Vec::new();

        for name in remote_names.iter().flatten() {
            remotes.push(name.to_string());
        }

        // Sort remotes for consistent display, with "origin" first if it exists
        remotes.sort_by(|a, b| {
            if a == "origin" {
                std::cmp::Ordering::Less
            } else if b == "origin" {
                std::cmp::Ordering::Greater
            } else {
                a.cmp(b)
            }
        });

        Ok(remotes)
    }

    /// Fetches latest data from a remote repository and updates the specified branch.
    ///
    /// Fetches from the remote and updates both remote-tracking branches and the specified
    /// local branch if it can be fast-forwarded. This ensures the selected branch is in sync
    /// with its remote counterpart before processing.
    ///
    /// Supports SSH authentication via SSH agent, SSH keys from ~/.ssh/, or other credential helpers.
    ///
    /// # Arguments
    /// * `remote_name` - Name of the remote (e.g., "origin")
    /// * `branch_name` - Name of the local branch to update (e.g., "master")
    ///
    /// # Returns
    /// * `Ok(())` - Successfully fetched and updated
    /// * `Err` - If remote not found or fetch fails
    pub fn fetch_from_remote(&self, remote_name: &str, branch_name: &str) -> Result<()> {
        let mut remote = self
            .repo
            .find_remote(remote_name)
            .map_err(|_| anyhow::anyhow!("Remote '{}' not found", remote_name))?;

        let mut fetch_options = git2::FetchOptions::new();

        // Set credentials callback for authentication
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, allowed_types| {
            // SSH key authentication
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                // Try different key types in order of preference
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let key_paths = vec![
                    format!("{}/.ssh/id_ed25519", home),
                    format!("{}/.ssh/id_rsa", home),
                    format!("{}/.ssh/id_ecdsa", home),
                ];

                for key_path in key_paths {
                    let path = std::path::Path::new(&key_path);
                    if path.exists() {
                        if let Ok(cred) = git2::Cred::ssh_key(
                            username_from_url.unwrap_or("git"),
                            None,
                            path,
                            None,
                        ) {
                            return Ok(cred);
                        }
                    }
                }

                // Try SSH agent as fallback
                if let Ok(cred) = git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
                {
                    return Ok(cred);
                }
            }

            // Fall back to default credentials
            git2::Cred::default()
        });

        fetch_options.remote_callbacks(callbacks);

        // Use explicit refspecs to fetch all branches and tags from the remote.
        // The refspecs mean:
        // - "+refs/heads/*:refs/remotes/{remote_name}/*" - Fetch all remote branches
        // - "+refs/tags/*:refs/tags/*" - Fetch all tags
        let refspec_heads = format!("+refs/heads/*:refs/remotes/{}/*", remote_name);
        let refspecs = &[refspec_heads.as_str(), "+refs/tags/*:refs/tags/*"];
        remote
            .fetch(refspecs, Some(&mut fetch_options), None)
            .map_err(|e| anyhow::anyhow!("Failed to fetch from remote '{}': {}", remote_name, e))?;

        // After fetching, try to fast-forward the specified branch with its remote counterpart
        self.update_branch_from_remote(branch_name, remote_name)?;

        Ok(())
    }

    /// Updates a local branch to match its remote counterpart via fast-forward merge.
    ///
    /// If the remote has new commits that can be fast-forwarded into the local branch,
    /// this method will perform the merge. This is similar to `git pull --ff-only`.
    ///
    /// # Arguments
    /// * `branch_name` - Name of the local branch to update
    /// * `remote_name` - Name of the remote (e.g., "origin")
    ///
    /// # Returns
    /// * `Ok(())` - Successfully updated or no update needed
    /// * `Err` - If the operation cannot be completed
    pub fn update_branch_from_remote(&self, branch_name: &str, remote_name: &str) -> Result<()> {
        // Get the remote-tracking branch OID
        let remote_tracking_branch_name = format!("{}/{}", remote_name, branch_name);
        let remote_ref = match self
            .repo
            .find_reference(&format!("refs/remotes/{}", remote_tracking_branch_name))
        {
            Ok(r) => r,
            Err(_) => {
                // Remote branch doesn't exist, nothing to update
                return Ok(());
            }
        };

        let remote_oid = remote_ref.target().ok_or_else(|| {
            anyhow::anyhow!(
                "Remote {} reference is invalid",
                remote_tracking_branch_name
            )
        })?;

        // Get the local branch OID
        let local_branch = match self.repo.find_branch(branch_name, BranchType::Local) {
            Ok(b) => b,
            Err(_) => {
                // Local branch doesn't exist, create it from remote
                let remote_commit = self.repo.find_commit(remote_oid)?;
                self.repo.branch(branch_name, &remote_commit, false)?;
                return Ok(());
            }
        };

        let local_ref = local_branch.into_reference();
        let local_oid = match local_ref.target() {
            Some(oid) => oid,
            None => {
                // Local branch reference is invalid
                return Ok(());
            }
        };

        // If they're the same, nothing to do
        if local_oid == remote_oid {
            return Ok(());
        }

        // Check if we can fast-forward: remote must be reachable from local's perspective
        let can_fast_forward = self.repo.graph_descendant_of(remote_oid, local_oid)?;

        if !can_fast_forward {
            // Cannot fast-forward, branches have diverged
            // This is OK - the local branch is ahead or has diverged
            return Ok(());
        }

        // Perform the fast-forward: update the local branch reference to point to remote's commit
        let branch_ref_name = format!("refs/heads/{}", branch_name);
        match self.repo.find_reference(&branch_ref_name) {
            Ok(mut reference) => {
                reference.set_target(
                    remote_oid,
                    &format!("fast-forward from {}", remote_tracking_branch_name),
                )?;
            }
            Err(_) => {
                // Reference doesn't exist, which shouldn't happen since we found the branch earlier
                return Err(anyhow::anyhow!(
                    "Cannot find reference for branch {}",
                    branch_name
                ));
            }
        }

        Ok(())
    }

    /// Gets the commit object ID (OID) of a branch head.
    ///
    /// # Arguments
    /// * `branch_name` - Name of the branch
    ///
    /// # Returns
    /// * `Ok(Oid)` - The commit OID at the branch head
    /// * `Err` - If branch not found
    pub fn get_branch_head_oid(&self, branch_name: &str) -> Result<Oid> {
        let branch = self.repo.find_branch(branch_name, BranchType::Local)?;
        let commit = branch.into_reference().peel_to_commit()?;
        Ok(commit.id())
    }

    /// Finds the latest tag on a specific branch.
    ///
    /// Walks the commit history from the branch head backwards to find the most recent tag.
    /// Handles both lightweight and annotated tags.
    ///
    /// # Arguments
    /// * `branch_name` - Name of the branch to search
    ///
    /// # Returns
    /// * `Ok(Some(tag))` - The latest tag name found
    /// * `Ok(None)` - If no tags exist on this branch
    /// * `Err` - If branch lookup fails
    pub fn get_latest_tag_on_branch(&self, branch_name: &str) -> Result<Option<String>> {
        let branch_oid = self.get_branch_head_oid(branch_name)?;

        // Walk the commit history to find the latest tag
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push(branch_oid)?;

        // Get all tags and their OIDs (handles both lightweight and annotated tags)
        let mut tag_oids = std::collections::HashMap::new();
        let tags = self.repo.tag_names(None)?;

        for tag_name in tags.iter().flatten() {
            if let Ok(tag_ref) = self.repo.find_reference(&format!("refs/tags/{}", tag_name)) {
                // Peel to any object (commit, tag, etc.)
                if let Ok(tag_obj) = tag_ref.peel(git2::ObjectType::Any) {
                    let tag_oid = tag_obj.id();
                    tag_oids.insert(tag_oid, tag_name.to_string());
                }
            }
        }

        // Find the latest tag on this branch
        for oid in revwalk {
            match oid {
                Ok(oid) => {
                    if let Some(tag_name) = tag_oids.get(&oid) {
                        return Ok(Some(tag_name.clone()));
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(None)
    }

    /// Gets all commits on a branch since a specific tag.
    ///
    /// Walks the commit history from the branch head backwards, collecting all commits
    /// until the tag commit is reached. Returns commits in chronological order (oldest first).
    ///
    /// # Arguments
    /// * `branch_name` - Name of the branch
    /// * `tag_name` - Optional tag to stop at; if None, returns all commits on branch
    ///
    /// # Returns
    /// * `Ok(commits)` - Vector of commits since tag (chronological order)
    /// * `Err` - If branch lookup fails
    pub fn get_commits_since_tag(
        &self,
        branch_name: &str,
        tag_name: Option<&str>,
    ) -> Result<Vec<Commit<'_>>> {
        let branch_oid = self.get_branch_head_oid(branch_name)?;

        // Walk commits from branch head backwards until the tag commit
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push(branch_oid)?;

        if let Some(tag_name) = tag_name {
            // Find the tag OID to stop at
            let tag_oid = self
                .repo
                .find_reference(&format!("refs/tags/{}", tag_name))
                .ok()
                .and_then(|r| r.peel(git2::ObjectType::Any).ok())
                .map(|obj| obj.id());

            let mut commits = Vec::new();

            for oid in revwalk {
                let oid = oid?;

                // Stop if we reached the tag commit
                if let Some(target_oid) = tag_oid {
                    if oid == target_oid {
                        break;
                    }
                }

                if let Ok(commit) = self.repo.find_commit(oid) {
                    commits.push(commit);
                }
            }

            // Reverse to get chronological order (oldest first)
            commits.reverse();
            Ok(commits)
        } else {
            // If no tag, return all commits reachable from branch
            let mut commits = Vec::new();
            for oid in revwalk {
                let oid = oid?;
                if let Ok(commit) = self.repo.find_commit(oid) {
                    commits.push(commit);
                }
            }
            // Reverse to get chronological order
            commits.reverse();
            Ok(commits)
        }
    }

    /// Get the current HEAD git hash (full 40-character SHA-1)
    #[allow(dead_code)]
    pub fn get_current_head_hash(&self) -> Result<String> {
        let head = self.repo.head()?;
        let oid = head
            .target()
            .ok_or_else(|| anyhow::anyhow!("HEAD is detached or invalid"))?;
        Ok(oid.to_string())
    }

    /// Creates a lightweight tag on the current HEAD commit.
    ///
    /// # Arguments
    /// * `tag_name` - Name of the tag to create
    ///
    /// # Returns
    /// * `Ok(())` - Tag created successfully
    /// * `Err` - If tag creation fails
    pub fn create_tag(&self, tag_name: &str) -> Result<()> {
        let head = self.repo.head()?.peel_to_commit()?;
        self.repo
            .tag_lightweight(tag_name, head.as_object(), false)?;
        Ok(())
    }

    /// Pushes a tag to a specified remote.
    ///
    /// Attempts to authenticate using SSH credentials from ~/.ssh/id_rsa.
    ///
    /// # Arguments
    /// * `tag_name` - Name of the tag to push
    /// * `remote_name` - Name of the remote to push to (e.g., "origin", "upstream")
    ///
    /// # Returns
    /// * `Ok(())` - Tag pushed successfully
    /// * `Err` - If push fails (network, auth, or reference error)
    pub fn push_tag(&self, tag_name: &str, remote_name: &str) -> Result<()> {
        let mut remote = match self.repo.find_remote(remote_name) {
            Ok(remote) => remote,
            Err(_) => return Err(anyhow::anyhow!("No remote named '{}' found", remote_name)),
        };

        let mut push_options = git2::PushOptions::new();

        // Set credentials callback if needed
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            // Try to get credentials from various sources
            if let Some(username) = username_from_url {
                git2::Cred::ssh_key(
                    username,
                    None,
                    std::path::Path::new(&format!(
                        "{}/.ssh/id_rsa",
                        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
                    )),
                    None,
                )
            } else {
                git2::Cred::default()
            }
        });

        // Add a push update reference callback to catch errors during push
        callbacks.push_update_reference(|refname, status| {
            if let Some(status) = status {
                eprintln!(
                    "Warning: Could not update reference {}: {}",
                    refname, status
                );
                Err(git2::Error::from_str(&format!(
                    "Push failed for {}",
                    refname
                )))
            } else {
                Ok(())
            }
        });

        push_options.remote_callbacks(callbacks);

        match remote.push(
            &[&format!("refs/tags/{}", tag_name)],
            Some(&mut push_options),
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                // Provide more informative error message
                if e.class() == git2::ErrorClass::Net {
                    Err(anyhow::anyhow!("Network error during push: {}", e))
                } else if e.class() == git2::ErrorClass::Reference {
                    Err(anyhow::anyhow!("Reference error during push: {}", e))
                } else {
                    Err(anyhow::anyhow!("Failed to push tag '{}': {}", tag_name, e))
                }
            }
        }
    }
}
