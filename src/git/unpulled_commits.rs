use std::collections::HashMap;

use git2::Repository;

use crate::{
    errors::MagiResult,
    model::{Line, LineContent, SectionType},
};

use super::{CommitInfo, CommitRef, CommitRefType};

const MAX_COMMITS: usize = 10;

/// Returns the lines representing unpulled commits from the upstream branch
pub fn get_lines(repository: &Repository) -> MagiResult<Vec<Line>> {
    let mut lines = Vec::new();

    let head = match repository.head() {
        Ok(head) => head,
        Err(_) => return Ok(lines), // No commits yet
    };

    // Only proceed if we're on a branch
    if !head.is_branch() {
        return Ok(lines);
    }

    let head_commit = match head.peel_to_commit() {
        Ok(commit) => commit,
        Err(_) => return Ok(lines),
    };

    // Get the upstream branch name
    let branch_ref = head.resolve()?;
    let branch_name = branch_ref.name().unwrap_or("unknown");

    let upstream_name = match repository
        .branch_upstream_name(branch_name)
        .ok()
        .and_then(|n| n.as_str().map(|s| s.to_string()))
    {
        Some(name) => name,
        None => return Ok(lines), // No upstream configured
    };

    let upstream_ref = match repository.find_reference(&upstream_name) {
        Ok(r) => r,
        Err(_) => return Ok(lines),
    };

    let upstream_commit = match upstream_ref.peel_to_commit() {
        Ok(commit) => commit,
        Err(_) => return Ok(lines),
    };

    // Get the shorthand name for display (e.g., "origin/main")
    let upstream_shorthand = upstream_ref
        .shorthand()
        .unwrap_or(&upstream_name)
        .to_string();

    // Find commits that are on upstream but not in HEAD
    // These are commits we need to pull
    let mut revwalk = repository.revwalk()?;
    revwalk.push(upstream_commit.id())?;
    revwalk.hide(head_commit.id())?;

    let commit_oids: Vec<_> = revwalk
        .take(MAX_COMMITS)
        .filter_map(|oid| oid.ok())
        .collect();

    if commit_oids.is_empty() {
        return Ok(lines);
    }

    // Build maps for tags and branches
    let tag_map = build_tag_map(repository)?;
    let local_branch_map = build_local_branch_map(repository)?;
    let remote_branch_map = build_remote_branch_map(repository)?;

    // Add section header
    lines.push(Line {
        content: LineContent::UnpulledSectionHeader {
            remote_name: upstream_shorthand,
            count: commit_oids.len(),
        },
        section: Some(SectionType::Unpulled),
    });

    // Add commit lines
    for oid in &commit_oids {
        let commit = match repository.find_commit(*oid) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let hash = format!("{:.7}", oid);
        let message = commit.summary().unwrap_or("").to_string();

        let mut refs = Vec::new();

        // Add local branches
        if let Some(local_branches) = local_branch_map.get(oid) {
            for branch in local_branches {
                refs.push(CommitRef {
                    name: branch.clone(),
                    ref_type: CommitRefType::LocalBranch,
                });
            }
        }

        // Add remote branches
        if let Some(remote_branches) = remote_branch_map.get(oid) {
            for branch in remote_branches {
                refs.push(CommitRef {
                    name: branch.clone(),
                    ref_type: CommitRefType::RemoteBranch,
                });
            }
        }

        let tag = tag_map.get(oid).cloned();

        let commit_info = CommitInfo {
            hash,
            refs,
            tag,
            message,
        };

        lines.push(Line {
            content: LineContent::Commit(commit_info),
            section: Some(SectionType::Unpulled),
        });
    }

    Ok(lines)
}

/// Build a map of commit OID -> tag name
fn build_tag_map(repository: &Repository) -> MagiResult<HashMap<git2::Oid, String>> {
    let mut tag_map = HashMap::new();

    repository.tag_foreach(|oid, name| {
        let tag_name = String::from_utf8_lossy(name)
            .strip_prefix("refs/tags/")
            .unwrap_or(&String::from_utf8_lossy(name))
            .to_string();

        if let Ok(obj) = repository.find_object(oid, None) {
            let target_oid = if let Some(tag) = obj.as_tag() {
                tag.target_id()
            } else {
                oid
            };
            tag_map.insert(target_oid, tag_name);
        }
        true
    })?;

    Ok(tag_map)
}

/// Build a map of commit OID -> list of local branch names
fn build_local_branch_map(repository: &Repository) -> MagiResult<HashMap<git2::Oid, Vec<String>>> {
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

    for branches in branch_map.values_mut() {
        branches.sort();
    }

    Ok(branch_map)
}

/// Build a map of commit OID -> list of remote branch names
fn build_remote_branch_map(repository: &Repository) -> MagiResult<HashMap<git2::Oid, Vec<String>>> {
    let mut branch_map: HashMap<git2::Oid, Vec<String>> = HashMap::new();

    let branches = repository.branches(Some(git2::BranchType::Remote))?;
    for (branch, _) in branches.flatten() {
        if let Ok(Some(name)) = branch.name() {
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

    for branches in branch_map.values_mut() {
        branches.sort();
    }

    Ok(branch_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;
    use git2::{BranchType, Signature};
    use std::fs;

    /// Creates a commit on the given reference (e.g., "refs/remotes/origin/main")
    fn create_commit_on_ref(repo: &Repository, ref_name: &str, message: &str, parent_oid: git2::Oid) {
        let sig = Signature::now("Test", "test@test.com").unwrap();
        let parent = repo.find_commit(parent_oid).unwrap();

        let path = repo.workdir().unwrap().join("remote_dummy.txt");
        fs::write(&path, message).unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("remote_dummy.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let new_tree = repo.find_tree(tree_id).unwrap();

        repo.commit(Some(ref_name), &sig, &sig, message, &new_tree, &[&parent])
            .unwrap();
    }

    #[test]
    fn test_get_lines_no_upstream() {
        let test_repo = TestRepo::new();
        let lines = get_lines(&test_repo.repo).unwrap();

        // No upstream configured, so no lines
        assert!(lines.is_empty());
    }

    /// Helper to set up upstream tracking: creates remote, remote-tracking branch, and configures upstream
    fn setup_upstream(repo: &Repository, head_oid: git2::Oid) {
        // Create the remote (required for set_upstream to work)
        repo.remote("origin", "https://example.com/repo.git").unwrap();

        // Create a remote-tracking branch
        repo.reference("refs/remotes/origin/main", head_oid, false, "create remote branch")
            .unwrap();

        // Configure upstream tracking for main -> origin/main
        let mut branch = repo.find_branch("main", BranchType::Local).unwrap();
        branch.set_upstream(Some("origin/main")).unwrap();
    }

    #[test]
    fn test_get_lines_with_upstream_no_unpulled() {
        let test_repo = TestRepo::new();
        let head_oid = test_repo.repo.head().unwrap().peel_to_commit().unwrap().id();

        setup_upstream(&test_repo.repo, head_oid);

        let lines = get_lines(&test_repo.repo).unwrap();

        // Same commit, so no unpulled commits
        assert!(lines.is_empty());
    }

    #[test]
    fn test_get_lines_with_unpulled_commits() {
        let test_repo = TestRepo::new();
        let head_oid = test_repo.repo.head().unwrap().peel_to_commit().unwrap().id();

        setup_upstream(&test_repo.repo, head_oid);

        // Add commits to the remote branch (simulating unpulled commits)
        create_commit_on_ref(&test_repo.repo, "refs/remotes/origin/main", "Remote commit 1", head_oid);
        let remote_oid = test_repo
            .repo
            .find_reference("refs/remotes/origin/main")
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id();
        create_commit_on_ref(&test_repo.repo, "refs/remotes/origin/main", "Remote commit 2", remote_oid);

        let lines = get_lines(&test_repo.repo).unwrap();

        // Should have section header + 2 unpulled commits
        assert_eq!(lines.len(), 3);

        // First line should be section header
        match &lines[0].content {
            LineContent::UnpulledSectionHeader { remote_name, count } => {
                assert_eq!(remote_name, "origin/main");
                assert_eq!(*count, 2);
            }
            _ => panic!("Expected UnpulledSectionHeader"),
        }

        // Second line should be most recent unpulled commit
        if let LineContent::Commit(ref info) = lines[1].content {
            assert_eq!(info.message, "Remote commit 2");
        } else {
            panic!("Expected Commit content");
        }

        // Third line should be older unpulled commit
        if let LineContent::Commit(ref info) = lines[2].content {
            assert_eq!(info.message, "Remote commit 1");
        } else {
            panic!("Expected Commit content");
        }
    }

    #[test]
    fn test_unpulled_commits_have_correct_section() {
        let test_repo = TestRepo::new();
        let head_oid = test_repo.repo.head().unwrap().peel_to_commit().unwrap().id();

        setup_upstream(&test_repo.repo, head_oid);

        // Add one unpulled commit
        create_commit_on_ref(&test_repo.repo, "refs/remotes/origin/main", "Unpulled commit", head_oid);

        let lines = get_lines(&test_repo.repo).unwrap();

        // All lines should have Unpulled section
        for line in &lines {
            assert_eq!(line.section, Some(SectionType::Unpulled));
        }
    }

    #[test]
    fn test_unpulled_section_header_structure() {
        // Unit test that verifies the line structure without git operations
        let lines = vec![Line {
            content: LineContent::UnpulledSectionHeader {
                remote_name: "origin/main".to_string(),
                count: 3,
            },
            section: Some(SectionType::Unpulled),
        }];

        assert_eq!(lines.len(), 1);
        match &lines[0].content {
            LineContent::UnpulledSectionHeader { remote_name, count } => {
                assert_eq!(remote_name, "origin/main");
                assert_eq!(*count, 3);
            }
            _ => panic!("Expected UnpulledSectionHeader"),
        }
        assert_eq!(lines[0].section, Some(SectionType::Unpulled));
    }
}
