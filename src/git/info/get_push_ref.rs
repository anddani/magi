use crate::git::GitRef;
use git2::Repository;

/// Get the push reference information from a Git repository
pub fn get_push_ref(repo: &Repository) -> Result<Option<GitRef>, git2::Error> {
    let head = repo.head()?;

    if !head.is_branch() {
        return Ok(None);
    }

    // Get the full branch name (e.g., "refs/heads/main")
    let branch_ref = head.resolve()?;
    let branch_name = branch_ref.name().unwrap_or("unknown");

    // Gets the full remote branch name (e.g. "refs/remotes/origin/main")
    let Some(upstream_name) = repo
        .branch_upstream_name(branch_name)
        .ok()
        .and_then(|n| n.as_str().map(|s| s.to_string()))
    else {
        return Ok(None);
    };

    let upstream_ref = match repo.find_reference(&upstream_name) {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    let commit = upstream_ref.peel_to_commit()?;
    let commit_hash = commit.id().to_string();
    let short_hash = &commit_hash[..7];
    let commit_message = commit.message().unwrap_or("").to_string();
    let upstream_shorthand = upstream_ref
        .shorthand()
        .unwrap_or(&upstream_name)
        .to_string();

    Ok(Some(GitRef::new_remote_branch(
        upstream_shorthand,
        short_hash.to_string(),
        commit_message,
    )))
}
