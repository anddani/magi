use std::collections::HashMap;

use git2::Repository;

use crate::{
    errors::MagiResult,
    model::{Line, LineContent, SectionType},
};

use super::CommitInfo;

const MAX_COMMITS: usize = 10;

/// Returns the lines representing recent commits in the Git repository
pub fn get_lines(repository: &Repository) -> MagiResult<Vec<Line>> {
    let mut lines = Vec::new();

    // Get HEAD commit to start the walk
    let head = match repository.head() {
        Ok(head) => head,
        Err(_) => return Ok(lines), // No commits yet
    };

    let head_commit = match head.peel_to_commit() {
        Ok(commit) => commit,
        Err(_) => return Ok(lines),
    };

    // Check if HEAD is detached
    let is_detached = repository.head_detached().unwrap_or(false);

    // Get the current branch name if not detached
    let current_branch = if is_detached {
        None
    } else {
        head.shorthand().map(|s| s.to_string())
    };

    // Get upstream branch name if available
    let upstream_name = get_upstream_name(repository, current_branch.as_deref());

    // Build a map of commit OID -> tag names
    let tag_map = build_tag_map(repository)?;

    // Build a map of commit OID -> branch names (for commits other than HEAD)
    let branch_map = build_branch_map(repository)?;

    // Walk through commits
    let mut revwalk = repository.revwalk()?;
    revwalk.push(head_commit.id())?;

    let commits: Vec<_> = revwalk.take(MAX_COMMITS).filter_map(|oid| oid.ok()).collect();

    if commits.is_empty() {
        return Ok(lines);
    }

    // Add section header
    lines.push(Line {
        content: LineContent::SectionHeader {
            title: "Recent commits".to_string(),
            count: None,
        },
        section: Some(SectionType::RecentCommits),
    });

    // Add commit lines
    for (index, oid) in commits.iter().enumerate() {
        let commit = match repository.find_commit(*oid) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let hash = format!("{:.7}", oid);
        let message = commit
            .summary()
            .unwrap_or("")
            .to_string();

        // Determine branch/upstream for this commit
        let (branch, upstream) = if index == 0 {
            // First commit is HEAD
            if is_detached {
                (Some("@".to_string()), None)
            } else {
                (current_branch.clone(), upstream_name.clone())
            }
        } else {
            // For other commits, check if any branch points to them
            (branch_map.get(oid).cloned(), None)
        };

        // Check for tag
        let tag = tag_map.get(oid).cloned();

        let commit_info = CommitInfo {
            hash,
            branch,
            upstream,
            tag,
            message,
        };

        lines.push(Line {
            content: LineContent::Commit(commit_info),
            section: Some(SectionType::RecentCommits),
        });
    }

    Ok(lines)
}

/// Get the upstream branch name for the current branch
fn get_upstream_name(repository: &Repository, branch_name: Option<&str>) -> Option<String> {
    let branch_name = branch_name?;
    let branch = repository.find_branch(branch_name, git2::BranchType::Local).ok()?;
    let upstream = branch.upstream().ok()?;
    upstream.name().ok()?.map(|s| s.to_string())
}

/// Build a map of commit OID -> tag name
fn build_tag_map(repository: &Repository) -> MagiResult<HashMap<git2::Oid, String>> {
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

/// Build a map of commit OID -> branch name for all local branches
fn build_branch_map(repository: &Repository) -> MagiResult<HashMap<git2::Oid, String>> {
    let mut branch_map = HashMap::new();

    let branches = repository.branches(Some(git2::BranchType::Local))?;
    for (branch, _) in branches.flatten() {
        if let Ok(Some(name)) = branch.name() {
            if let Ok(commit) = branch.get().peel_to_commit() {
                branch_map.insert(commit.id(), name.to_string());
            }
        }
    }

    Ok(branch_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;
    use git2::Signature;
    use std::fs;

    fn create_commit(repo: &Repository, message: &str) {
        let sig = Signature::now("Test", "test@test.com").unwrap();
        let head = repo.head().unwrap();
        let parent = head.peel_to_commit().unwrap();

        // Create a dummy change
        let path = repo.workdir().unwrap().join("dummy.txt");
        fs::write(&path, message).unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("dummy.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let new_tree = repo.find_tree(tree_id).unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, message, &new_tree, &[&parent])
            .unwrap();
    }

    #[test]
    fn test_get_lines_with_initial_commit() {
        let test_repo = TestRepo::new();
        let lines = get_lines(&test_repo.repo).unwrap();

        // TestRepo::new() creates an initial commit
        // Should have section header + 1 commit
        assert_eq!(lines.len(), 2);

        // First line should be section header
        assert!(matches!(
            lines[0].content,
            LineContent::SectionHeader { ref title, .. } if title == "Recent commits"
        ));

        // Second line should be the initial commit
        if let LineContent::Commit(ref info) = lines[1].content {
            assert_eq!(info.message, "Initial commit");
            assert_eq!(info.hash.len(), 7);
        } else {
            panic!("Expected Commit content");
        }
    }

    #[test]
    fn test_get_lines_with_multiple_commits() {
        let test_repo = TestRepo::new();
        create_commit(&test_repo.repo, "Second commit");

        let lines = get_lines(&test_repo.repo).unwrap();

        // Should have section header + 2 commits
        assert_eq!(lines.len(), 3);

        // Second line should be most recent commit
        if let LineContent::Commit(ref info) = lines[1].content {
            assert_eq!(info.message, "Second commit");
        } else {
            panic!("Expected Commit content");
        }

        // Third line should be initial commit
        if let LineContent::Commit(ref info) = lines[2].content {
            assert_eq!(info.message, "Initial commit");
        } else {
            panic!("Expected Commit content");
        }
    }

    #[test]
    fn test_get_lines_max_commits() {
        let test_repo = TestRepo::new();

        // Create more commits (TestRepo already has 1)
        for i in 0..12 {
            create_commit(&test_repo.repo, &format!("Commit {}", i));
        }

        let lines = get_lines(&test_repo.repo).unwrap();

        // Should have section header + MAX_COMMITS (10)
        assert_eq!(lines.len(), MAX_COMMITS + 1);
    }

    #[test]
    fn test_commit_has_branch_info() {
        let test_repo = TestRepo::new();

        let lines = get_lines(&test_repo.repo).unwrap();

        // The first commit should have branch info (main branch)
        if let LineContent::Commit(ref info) = lines[1].content {
            assert!(info.branch.is_some());
            assert_eq!(info.branch.as_deref(), Some("main"));
        } else {
            panic!("Expected Commit content");
        }
    }

    #[test]
    fn test_detached_head_shows_at_symbol() {
        let test_repo = TestRepo::new();
        test_repo.detach_head();

        let lines = get_lines(&test_repo.repo).unwrap();

        // The first commit should show "@" for detached head
        if let LineContent::Commit(ref info) = lines[1].content {
            assert_eq!(info.branch.as_deref(), Some("@"));
        } else {
            panic!("Expected Commit content");
        }
    }
}
