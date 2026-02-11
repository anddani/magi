use std::collections::HashMap;

use git2::Repository;

use crate::errors::MagiResult;

/// Build a map of commit OID -> tag name
pub fn build_tag_map(repository: &Repository) -> MagiResult<HashMap<git2::Oid, String>> {
    let mut tag_map = HashMap::new();

    repository.tag_foreach(|oid, name| {
        // Tag names come as "refs/tags/tagname"
        let tag_name = String::from_utf8_lossy(name)
            .strip_prefix("refs/tags/")
            .unwrap_or(&String::from_utf8_lossy(name))
            .to_string();

        // Try to get the target commit for annotated tags
        if let Ok(obj) = repository.find_object(oid, None) {
            let target_oid = if let Some(tag) = obj.as_tag() {
                // Annotated tag - get the target commit
                tag.target_id()
            } else {
                // Lightweight tag - the OID is the commit
                oid
            };
            tag_map.insert(target_oid, tag_name);
        }
        true
    })?;

    Ok(tag_map)
}

/// Build a map of commit OID -> list of local branch names
pub fn build_local_branch_map(
    repository: &Repository,
) -> MagiResult<HashMap<git2::Oid, Vec<String>>> {
    let mut branch_map: HashMap<git2::Oid, Vec<String>> = HashMap::new();

    let branches = repository.branches(Some(git2::BranchType::Local))?;
    for (branch, _) in branches.flatten() {
        if let Ok(Some(name)) = branch.name()
            && let Ok(commit) = branch.get().peel_to_commit()
        {
            branch_map
                .entry(commit.id())
                .or_default()
                .push(name.to_string());
        }
    }

    // Sort branch names for consistent ordering
    for branches in branch_map.values_mut() {
        branches.sort();
    }

    Ok(branch_map)
}

/// Build a map of commit OID -> list of remote branch names
pub fn build_remote_branch_map(
    repository: &Repository,
) -> MagiResult<HashMap<git2::Oid, Vec<String>>> {
    let mut branch_map: HashMap<git2::Oid, Vec<String>> = HashMap::new();

    let branches = repository.branches(Some(git2::BranchType::Remote))?;
    for (branch, _) in branches.flatten() {
        if let Ok(Some(name)) = branch.name() {
            // Skip HEAD references like "origin/HEAD"
            if name.ends_with("/HEAD") {
                continue;
            }
            if let Ok(commit) = branch.get().peel_to_commit() {
                branch_map
                    .entry(commit.id())
                    .or_default()
                    .push(name.to_string());
            }
        }
    }

    // Sort branch names for consistent ordering
    for branches in branch_map.values_mut() {
        branches.sort();
    }

    Ok(branch_map)
}
