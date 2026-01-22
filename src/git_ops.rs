use anyhow::Result;
use git2::{BranchType, Commit, Oid, Repository};

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    pub fn new() -> Result<Self> {
        // Check if we're in a git repository
        let repo = match Repository::discover(".") {
            Ok(repo) => repo,
            Err(e) => return Err(anyhow::anyhow!("Not in a git repository: {}", e)),
        };
        Ok(GitRepo { repo })
    }

    /// Fetch latest data from remote to ensure branch and tags are up-to-date
    pub fn fetch_from_remote(&self, remote_name: &str) -> Result<()> {
        let mut remote = self
            .repo
            .find_remote(remote_name)
            .map_err(|_| anyhow::anyhow!("Remote '{}' not found", remote_name))?;

        let refspecs: &[&str] = &[];
        remote
            .fetch(refspecs, None, None)
            .map_err(|e| anyhow::anyhow!("Failed to fetch from remote '{}': {}", remote_name, e))?;

        Ok(())
    }

    pub fn get_branch_head_oid(&self, branch_name: &str) -> Result<Oid> {
        let branch = self.repo.find_branch(branch_name, BranchType::Local)?;
        let commit = branch.into_reference().peel_to_commit()?;
        Ok(commit.id())
    }

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

    pub fn create_tag(&self, tag_name: &str) -> Result<()> {
        let head = self.repo.head()?.peel_to_commit()?;
        self.repo
            .tag_lightweight(tag_name, head.as_object(), false)?;
        Ok(())
    }

    pub fn push_tag(&self, tag_name: &str) -> Result<()> {
        let mut remote = match self.repo.find_remote("origin") {
            Ok(remote) => remote,
            Err(_) => return Err(anyhow::anyhow!("No remote named 'origin' found")),
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
