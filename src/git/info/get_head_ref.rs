use crate::git::{GitRef, ReferenceType};
use git2::Repository;

/// Get the HEAD reference information from a Git repository
pub fn get_head_ref(repo: &Repository) -> Result<GitRef, git2::Error> {
    let head = repo.head()?;

    // If HEAD is a branch, use the branch name and commit info
    if head.is_branch() {
        let name = head.shorthand().unwrap_or("unknown").to_string();
        let commit = head.peel_to_commit()?;
        let commit_hash = commit.id().to_string();
        let short_hash = commit_hash.chars().take(7).collect::<String>();
        let commit_message = commit.message().unwrap_or("").to_string();

        // Determine if it's a local or remote branch
        let reference_type = if name.starts_with("origin/") {
            ReferenceType::RemoteBranch
        } else {
            ReferenceType::LocalBranch
        };

        Ok(GitRef::new(
            name,
            short_hash,
            commit_message,
            reference_type,
        ))
    } else {
        // Detached HEAD
        let commit = head.peel_to_commit()?;
        let commit_hash = commit.id().to_string();
        let short_hash = commit_hash.chars().take(7).collect::<String>();
        let commit_message = commit.message().unwrap_or("").to_string();

        Ok(GitRef::new_detached_head(short_hash, commit_message))
    }
}
