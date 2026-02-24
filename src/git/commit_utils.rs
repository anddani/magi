use std::collections::{HashMap, HashSet};

use git2::Repository;

use crate::{
    errors::MagiResult,
    model::{Line, LineContent, SectionType},
};

use super::{CommitInfo, CommitRef, CommitRefType};

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

/// Build a map of local branch name -> push remote name.
/// Checks `branch.<name>.pushRemote` per branch, falling back to `remote.pushDefault`.
pub fn build_push_remote_map(repository: &Repository) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let config = match repository.config() {
        Ok(c) => c,
        Err(_) => return map,
    };
    let push_default = config
        .get_string("remote.pushDefault")
        .ok()
        .filter(|s| !s.is_empty());

    if let Ok(branches) = repository.branches(Some(git2::BranchType::Local)) {
        for (branch, _) in branches.flatten() {
            if let Ok(Some(name)) = branch.name() {
                let remote = config
                    .get_string(&format!("branch.{}.pushRemote", name))
                    .ok()
                    .filter(|s| !s.is_empty())
                    .or_else(|| push_default.clone());
                if let Some(r) = remote {
                    map.insert(name.to_string(), r);
                }
            }
        }
    }
    map
}

/// Enrich a commit's refs with push remote info.
///
/// For each local branch that has a push remote configured AND whose push branch
/// (`remote/local-name`) is also in the refs for this commit:
/// - Sets `push_remote` on the local branch ref
/// - Removes the matching remote branch ref (it's already shown via the split-colored label)
///
/// Remote branch refs with a different name are kept as-is.
pub fn enrich_refs_with_push_remote(
    refs: Vec<CommitRef>,
    push_remote_map: &HashMap<String, String>,
) -> Vec<CommitRef> {
    // Collect all remote branch names present at this commit
    let remote_names: HashSet<String> = refs
        .iter()
        .filter(|r| r.ref_type == CommitRefType::RemoteBranch)
        .map(|r| r.name.clone())
        .collect();

    // Determine which remote refs to suppress (merged into a local branch label)
    let mut suppressed: HashSet<String> = HashSet::new();

    let mut enriched: Vec<CommitRef> = refs
        .into_iter()
        .map(|mut r| {
            if r.ref_type == CommitRefType::LocalBranch
                && let Some(remote) = push_remote_map.get(&r.name) {
                    let push_branch = format!("{}/{}", remote, r.name);
                    if remote_names.contains(&push_branch) {
                        r.push_remote = Some(remote.clone());
                        suppressed.insert(push_branch);
                    }
                }
            r
        })
        .collect();

    // Remove suppressed remote refs
    enriched
        .retain(|r| !(r.ref_type == CommitRefType::RemoteBranch && suppressed.contains(&r.name)));

    enriched
}

/// Builds refs (branches, tags) for a commit from the prebuilt maps
pub fn build_refs_for_commit(
    oid: &git2::Oid,
    local_branch_map: &HashMap<git2::Oid, Vec<String>>,
    remote_branch_map: &HashMap<git2::Oid, Vec<String>>,
    tag_map: &HashMap<git2::Oid, String>,
) -> Vec<CommitRef> {
    let mut refs = Vec::new();

    if let Some(branches) = local_branch_map.get(oid) {
        refs.extend(branches.iter().map(|b| CommitRef {
            name: b.clone(),
            ref_type: CommitRefType::LocalBranch,
            push_remote: None,
        }));
    }

    if let Some(branches) = remote_branch_map.get(oid) {
        refs.extend(branches.iter().map(|b| CommitRef {
            name: b.clone(),
            ref_type: CommitRefType::RemoteBranch,
            push_remote: None,
        }));
    }

    if let Some(tag) = tag_map.get(oid) {
        refs.push(CommitRef {
            name: tag.clone(),
            ref_type: CommitRefType::Tag,
            push_remote: None,
        });
    }

    refs
}

/// Sorts refs in a consistent order:
/// 1. Current branch (if provided and exists in refs)
/// 2. HEAD indicator (@) if present (for detached HEAD)
/// 3. Local branches (sorted alphabetically)
/// 4. Remote branches (sorted alphabetically)
/// 5. Tags (sorted alphabetically)
pub fn sort_refs(refs: Vec<CommitRef>, current_branch: Option<&str>) -> Vec<CommitRef> {
    let mut head_ref: Option<CommitRef> = None;
    let mut current_branch_ref: Option<CommitRef> = None;
    let mut local_branches = Vec::new();
    let mut remote_branches = Vec::new();
    let mut tags = Vec::new();

    for r in refs {
        match r.ref_type {
            CommitRefType::Head => head_ref = Some(r),
            CommitRefType::LocalBranch => {
                if current_branch == Some(r.name.as_str()) {
                    current_branch_ref = Some(r);
                } else {
                    local_branches.push(r);
                }
            }
            CommitRefType::RemoteBranch => remote_branches.push(r),
            CommitRefType::Tag => tags.push(r),
        }
    }

    // Sort each category alphabetically
    local_branches.sort_by(|a, b| a.name.cmp(&b.name));
    remote_branches.sort_by(|a, b| a.name.cmp(&b.name));
    tags.sort_by(|a, b| a.name.cmp(&b.name));

    // Combine in order: current_branch, HEAD (@), local branches, remote branches, tags
    let mut result = Vec::new();
    if let Some(branch) = current_branch_ref {
        result.push(branch);
    }
    if let Some(head) = head_ref {
        result.push(head);
    }
    result.append(&mut local_branches);
    result.append(&mut remote_branches);
    result.append(&mut tags);

    result
}

/// Creates a commit Line from a git2::Commit
pub fn create_commit_line(
    commit: &git2::Commit,
    refs: Vec<CommitRef>,
    section: SectionType,
) -> Line {
    Line {
        content: LineContent::Commit(CommitInfo {
            hash: format!("{:.7}", commit.id()),
            refs,
            message: commit.summary().unwrap_or("").to_string(),
        }),
        section: Some(section),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local_branch(name: &str) -> CommitRef {
        CommitRef {
            name: name.to_string(),
            ref_type: CommitRefType::LocalBranch,
            push_remote: None,
        }
    }

    fn remote_branch(name: &str) -> CommitRef {
        CommitRef {
            name: name.to_string(),
            ref_type: CommitRefType::RemoteBranch,
            push_remote: None,
        }
    }

    fn tag(name: &str) -> CommitRef {
        CommitRef {
            name: name.to_string(),
            ref_type: CommitRefType::Tag,
            push_remote: None,
        }
    }

    fn head() -> CommitRef {
        CommitRef {
            name: "@".to_string(),
            ref_type: CommitRefType::Head,
            push_remote: None,
        }
    }

    #[test]
    fn test_sort_refs_current_branch_first() {
        let refs = vec![
            local_branch("beta"),
            local_branch("alpha"),
            local_branch("main"),
        ];
        let sorted = sort_refs(refs, Some("main"));

        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].name, "main");
        assert_eq!(sorted[1].name, "alpha");
        assert_eq!(sorted[2].name, "beta");
    }

    #[test]
    fn test_sort_refs_head_after_current_branch() {
        let refs = vec![head(), local_branch("beta"), local_branch("main")];
        let sorted = sort_refs(refs, Some("main"));

        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].name, "main");
        assert_eq!(sorted[0].ref_type, CommitRefType::LocalBranch);
        assert_eq!(sorted[1].name, "@");
        assert_eq!(sorted[1].ref_type, CommitRefType::Head);
        assert_eq!(sorted[2].name, "beta");
    }

    #[test]
    fn test_sort_refs_detached_head_first() {
        let refs = vec![local_branch("beta"), head(), local_branch("alpha")];
        let sorted = sort_refs(refs, None);

        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].name, "@");
        assert_eq!(sorted[0].ref_type, CommitRefType::Head);
        assert_eq!(sorted[1].name, "alpha");
        assert_eq!(sorted[2].name, "beta");
    }

    #[test]
    fn test_sort_refs_full_ordering() {
        // Input in random order
        let refs = vec![
            tag("v2.0"),
            remote_branch("origin/zebra"),
            local_branch("beta"),
            head(),
            tag("v1.0"),
            remote_branch("origin/apple"),
            local_branch("main"),
            local_branch("alpha"),
        ];
        let sorted = sort_refs(refs, Some("main"));

        assert_eq!(sorted.len(), 8);
        // 1. Current branch
        assert_eq!(sorted[0].name, "main");
        assert_eq!(sorted[0].ref_type, CommitRefType::LocalBranch);
        // 2. HEAD
        assert_eq!(sorted[1].name, "@");
        assert_eq!(sorted[1].ref_type, CommitRefType::Head);
        // 3-4. Local branches (sorted)
        assert_eq!(sorted[2].name, "alpha");
        assert_eq!(sorted[3].name, "beta");
        // 5-6. Remote branches (sorted)
        assert_eq!(sorted[4].name, "origin/apple");
        assert_eq!(sorted[5].name, "origin/zebra");
        // 7-8. Tags (sorted)
        assert_eq!(sorted[6].name, "v1.0");
        assert_eq!(sorted[7].name, "v2.0");
    }

    #[test]
    fn test_sort_refs_empty() {
        let refs = vec![];
        let sorted = sort_refs(refs, Some("main"));
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_sort_refs_no_current_branch_match() {
        let refs = vec![local_branch("beta"), local_branch("alpha")];
        let sorted = sort_refs(refs, Some("nonexistent"));

        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].name, "alpha");
        assert_eq!(sorted[1].name, "beta");
    }

    #[test]
    fn test_enrich_merges_push_branch_at_same_commit() {
        // Local branch "main" has pushRemote "origin" and "origin/main" is at the same commit
        let refs = vec![local_branch("main"), remote_branch("origin/main")];
        let push_remote_map: HashMap<String, String> =
            [("main".to_string(), "origin".to_string())].into();

        let enriched = enrich_refs_with_push_remote(refs, &push_remote_map);

        // "origin/main" should be suppressed; "main" gets push_remote = Some("origin")
        assert_eq!(enriched.len(), 1);
        assert_eq!(enriched[0].name, "main");
        assert_eq!(enriched[0].push_remote, Some("origin".to_string()));
    }

    #[test]
    fn test_enrich_keeps_different_remote_branch_separately() {
        // Local branch "main" has pushRemote "origin" but "origin/different" is at the same commit
        let refs = vec![local_branch("main"), remote_branch("origin/different")];
        let push_remote_map: HashMap<String, String> =
            [("main".to_string(), "origin".to_string())].into();

        let enriched = enrich_refs_with_push_remote(refs, &push_remote_map);

        // Both refs remain; "main" has no push_remote (push branch not at same commit)
        assert_eq!(enriched.len(), 2);
        assert_eq!(enriched[0].push_remote, None);
        assert_eq!(enriched[1].name, "origin/different");
    }

    #[test]
    fn test_enrich_no_push_remote_configured() {
        let refs = vec![local_branch("main"), remote_branch("origin/main")];
        let push_remote_map: HashMap<String, String> = HashMap::new();

        let enriched = enrich_refs_with_push_remote(refs, &push_remote_map);

        // Nothing changes when no push remote is configured
        assert_eq!(enriched.len(), 2);
        assert_eq!(enriched[0].push_remote, None);
    }
}
