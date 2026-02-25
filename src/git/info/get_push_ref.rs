use crate::git::GitRef;
use crate::git::config::get_push_remote;
use git2::Repository;

/// Get the push remote reference information from a Git repository.
/// Returns None if no push remote is explicitly configured via
/// `branch.<name>.pushRemote` or `remote.pushDefault`.
pub fn get_push_ref(repo: &Repository) -> Result<Option<GitRef>, git2::Error> {
    let head = repo.head()?;

    if !head.is_branch() {
        return Ok(None);
    }

    let branch_ref = head.resolve()?;
    let branch_name = match branch_ref.shorthand() {
        Some(name) => name.to_string(),
        None => return Ok(None),
    };

    // Only show push ref when pushRemote or pushDefault is explicitly configured
    let Some(push_remote) = get_push_remote(repo, &branch_name) else {
        return Ok(None);
    };

    // Construct the remote tracking ref path: refs/remotes/<remote>/<branch>
    let push_ref_name = format!("refs/remotes/{}/{}", push_remote, branch_name);

    let push_ref = match repo.find_reference(&push_ref_name) {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    let commit = push_ref.peel_to_commit()?;
    let commit_hash = commit.id().to_string();
    let short_hash = &commit_hash[..7];
    let commit_message = commit.message().unwrap_or("").to_string();
    let push_shorthand = push_ref.shorthand().unwrap_or(&push_ref_name).to_string();

    Ok(Some(GitRef::new_remote_branch(
        push_shorthand,
        short_hash.to_string(),
        commit_message,
    )))
}
