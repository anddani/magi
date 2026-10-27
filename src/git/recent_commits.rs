use git2::Repository;

use crate::{
    errors::MagiResult,
    i18n,
    model::{Line, LineContent, SectionType},
};

use super::commit_utils::{
    build_local_branch_map, build_push_remote_map, build_refs_for_commit, build_remote_branch_map,
    build_tag_map, create_commit_line, enrich_refs_with_push_remote, sort_refs,
};
use super::{CommitRef, CommitRefType};

const MAX_COMMITS: usize = 10;

/// Returns the lines for the recent-commits slot of the status buffer.
///
/// Mirrors magit's `magit-insert-unpushed-to-upstream-or-recent`: when the current
/// branch has an upstream (the "Merge" ref) that is behind HEAD, this shows the commits
/// that have not yet been merged into it under an "Unmerged into <upstream>" header.
/// Otherwise (no upstream, or the upstream is not behind HEAD) it falls back to showing
/// the most recent commits under a "Recent commits" header.
pub fn get_lines(repository: &Repository) -> MagiResult<Vec<Line>> {
    let mut lines = Vec::new();

    let head = match repository.head() {
        Ok(head) => head,
        Err(_) => return Ok(lines), // No commits yet
    };

    let head_commit = match head.peel_to_commit() {
        Ok(commit) => commit,
        Err(_) => return Ok(lines),
    };

    let is_detached = repository.head_detached().unwrap_or(false);

    let current_branch = if is_detached {
        None
    } else {
        head.shorthand().ok().map(|s| s.to_string())
    };

    // Build a map of commit OID -> tag names
    let tag_map = build_tag_map(repository)?;

    // Build maps of commit OID -> branch names
    let local_branch_map = build_local_branch_map(repository)?;
    let remote_branch_map = build_remote_branch_map(repository)?;

    // Build push remote map for split-colored labels
    let push_remote_map = build_push_remote_map(repository);

    // Decide between the "Unmerged into <upstream>" and "Recent commits" variants.
    let (commits, header) = match unmerged_into_upstream(repository, &head, &head_commit)? {
        Some((upstream_shorthand, oids)) => {
            let header = Line {
                content: LineContent::UnmergedIntoSectionHeader {
                    branch_name: upstream_shorthand,
                    count: oids.len(),
                },
                section: Some(SectionType::RecentCommits),
            };
            (oids, header)
        }
        None => {
            // Walk through recent commits starting from HEAD
            let mut revwalk = repository.revwalk()?;
            revwalk.push(head_commit.id())?;
            let oids: Vec<_> = revwalk
                .take(MAX_COMMITS)
                .filter_map(|oid| oid.ok())
                .collect();

            let header = Line {
                content: LineContent::SectionHeader {
                    title: i18n::t().section_recent_commits.to_string(),
                    count: None,
                },
                section: Some(SectionType::RecentCommits),
            };
            (oids, header)
        }
    };

    if commits.is_empty() {
        return Ok(lines);
    }

    lines.push(header);

    for (index, oid) in commits.iter().enumerate() {
        let commit = match repository.find_commit(*oid) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Build refs for this commit
        let mut refs = build_refs_for_commit(oid, &local_branch_map, &remote_branch_map, &tag_map);

        // For HEAD commit, add HEAD indicator if detached
        if index == 0 && is_detached {
            refs.push(CommitRef {
                name: "@".to_string(),
                ref_type: CommitRefType::Head,
                push_remote: None,
            });
        }

        // Sort refs with current branch first (if applicable)
        let refs = sort_refs(refs, current_branch.as_deref());

        // Enrich with push remote info (split-colored labels)
        let refs = enrich_refs_with_push_remote(refs, &push_remote_map);

        lines.push(create_commit_line(
            &commit,
            refs,
            SectionType::RecentCommits,
        ));
    }

    Ok(lines)
}

/// If the current branch has an upstream (the "Merge" ref) that is behind HEAD, returns
/// the upstream shorthand (e.g. "origin/main") together with the commits on HEAD that are
/// not yet merged into it (newest first, capped at `MAX_COMMITS`).
///
/// Returns `None` when there is no upstream configured, or when HEAD is an ancestor of the
/// upstream (nothing unmerged) — in which case the caller shows recent commits instead.
fn unmerged_into_upstream(
    repository: &Repository,
    head: &git2::Reference,
    head_commit: &git2::Commit,
) -> MagiResult<Option<(String, Vec<git2::Oid>)>> {
    if !head.is_branch() {
        return Ok(None);
    }

    let branch_ref = head.resolve()?;
    let branch_name = branch_ref.name().unwrap_or("unknown");

    let Some(upstream_name) = repository
        .branch_upstream_name(branch_name)
        .ok()
        .and_then(|n| n.as_str().ok().map(|s| s.to_string()))
    else {
        return Ok(None); // No upstream configured
    };

    let upstream_ref = match repository.find_reference(&upstream_name) {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    let upstream_commit = match upstream_ref.peel_to_commit() {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };

    // When HEAD is an ancestor of (or equal to) the upstream there is nothing unmerged,
    // so fall back to recent commits (matching magit's `magit-rev-ancestor-p "HEAD" ...`).
    if head_commit.id() == upstream_commit.id()
        || repository.graph_descendant_of(upstream_commit.id(), head_commit.id())?
    {
        return Ok(None);
    }

    let upstream_shorthand = upstream_ref
        .shorthand()
        .unwrap_or(&upstream_name)
        .to_string();

    // Commits reachable from HEAD but not from the upstream — i.e. not yet merged into it.
    let mut revwalk = repository.revwalk()?;
    revwalk.push(head_commit.id())?;
    revwalk.hide(upstream_commit.id())?;
    let oids: Vec<_> = revwalk
        .take(MAX_COMMITS)
        .filter_map(|oid| oid.ok())
        .collect();

    if oids.is_empty() {
        return Ok(None);
    }

    Ok(Some((upstream_shorthand, oids)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

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
        let file_name = "test.txt";
        let test_repo = TestRepo::new();
        test_repo
            .create_file(file_name)
            .stage_files(&[file_name])
            .commit("Second commit");

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
        let file_name = "test.txt";
        let test_repo = TestRepo::new();
        test_repo
            .create_file(file_name)
            .stage_files(&[file_name])
            .commit("Second commit");

        // Create more commits
        for i in 0..12 {
            let content = &format!("Commit #{}", i);
            test_repo
                .write_file_content(file_name, content)
                .commit(content);
        }

        let lines = get_lines(&test_repo.repo).unwrap();

        // Should have section header + MAX_COMMITS (10)
        assert_eq!(lines.len(), MAX_COMMITS + 1);
    }

    #[test]
    fn test_commit_has_branch_info() {
        let file_name = "test.txt";
        let test_repo = TestRepo::new();

        test_repo
            .create_file(file_name)
            .stage_files(&[file_name])
            .commit("Test commit");

        let lines = get_lines(&test_repo.repo).unwrap();

        // The first commit should have branch info (main branch)
        if let LineContent::Commit(ref info) = lines[1].content {
            assert!(!info.refs.is_empty());
            assert_eq!(info.refs[0].name, "main");
            assert_eq!(info.refs[0].ref_type, CommitRefType::LocalBranch);
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
            assert!(!info.refs.is_empty());
            assert_eq!(info.refs[0].name, "@");
            assert_eq!(info.refs[0].ref_type, CommitRefType::Head);
        } else {
            panic!("Expected Commit content");
        }
    }

    /// Sets up upstream tracking main -> origin/main, with origin/main pointing at `upstream_oid`.
    fn setup_upstream(repo: &Repository, upstream_oid: git2::Oid) {
        repo.remote("origin", "https://example.com/repo.git")
            .unwrap();
        repo.reference(
            "refs/remotes/origin/main",
            upstream_oid,
            false,
            "create remote branch",
        )
        .unwrap();
        let mut branch = repo.find_branch("main", git2::BranchType::Local).unwrap();
        branch.set_upstream(Some("origin/main")).unwrap();
    }

    #[test]
    fn test_unmerged_into_upstream_when_ahead() {
        let file_name = "test.txt";
        let test_repo = TestRepo::new();

        // Pin the upstream at the initial commit, then advance the local branch past it.
        let upstream_oid = test_repo
            .repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id();
        setup_upstream(&test_repo.repo, upstream_oid);

        test_repo
            .create_file(file_name)
            .stage_files(&[file_name])
            .commit("Second commit");
        test_repo
            .write_file_content(file_name, "more")
            .commit("Third commit");

        let lines = get_lines(&test_repo.repo).unwrap();

        // Header should be "Unmerged into origin/main (2)" + 2 commits
        assert_eq!(lines.len(), 3);
        match &lines[0].content {
            LineContent::UnmergedIntoSectionHeader { branch_name, count } => {
                assert_eq!(branch_name, "origin/main");
                assert_eq!(*count, 2);
            }
            _ => panic!("Expected UnmergedIntoSectionHeader"),
        }

        // Newest-first ordering
        if let LineContent::Commit(ref info) = lines[1].content {
            assert_eq!(info.message, "Third commit");
        } else {
            panic!("Expected Commit content");
        }
        if let LineContent::Commit(ref info) = lines[2].content {
            assert_eq!(info.message, "Second commit");
        } else {
            panic!("Expected Commit content");
        }

        // All lines belong to the RecentCommits section (shared collapsible slot)
        for line in &lines {
            assert_eq!(line.section, Some(SectionType::RecentCommits));
        }
    }

    #[test]
    fn test_recent_commits_when_upstream_up_to_date() {
        let test_repo = TestRepo::new();

        // Upstream points at HEAD — nothing is unmerged, so we fall back to recent commits.
        let head_oid = test_repo
            .repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id();
        setup_upstream(&test_repo.repo, head_oid);

        let lines = get_lines(&test_repo.repo).unwrap();

        assert_eq!(lines.len(), 2);
        assert!(matches!(
            lines[0].content,
            LineContent::SectionHeader { ref title, .. } if title == "Recent commits"
        ));
    }

    #[test]
    fn test_recent_commits_when_upstream_ahead() {
        let test_repo = TestRepo::new();

        // Create a separate commit the upstream points at, which is ahead of HEAD.
        let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
        let sig = git2::Signature::now("Test", "test@test.com").unwrap();
        let path = test_repo.repo.workdir().unwrap().join("ahead.txt");
        std::fs::write(&path, "ahead").unwrap();
        let mut index = test_repo.repo.index().unwrap();
        index.add_path(std::path::Path::new("ahead.txt")).unwrap();
        index.write().unwrap();
        let tree = test_repo
            .repo
            .find_tree(index.write_tree().unwrap())
            .unwrap();
        let ahead_oid = test_repo
            .repo
            .commit(None, &sig, &sig, "Upstream ahead", &tree, &[&head])
            .unwrap();
        // Reset the working index back so it doesn't interfere.
        let head_tree = head.tree().unwrap();
        test_repo
            .repo
            .checkout_tree(head_tree.as_object(), None)
            .unwrap();

        setup_upstream(&test_repo.repo, ahead_oid);

        let lines = get_lines(&test_repo.repo).unwrap();

        // HEAD is an ancestor of the upstream → nothing unmerged → "Recent commits".
        assert!(matches!(
            lines[0].content,
            LineContent::SectionHeader { ref title, .. } if title == "Recent commits"
        ));
    }
}
