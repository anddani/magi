use git2::Repository;

use crate::{
    errors::MagiResult,
    model::{Line, LineContent, SectionType},
};

use super::commit_utils::{
    build_local_branch_map, build_refs_for_commit, build_remote_branch_map, build_tag_map,
    create_commit_line, sort_refs,
};
use super::{CommitRef, CommitRefType};

const MAX_COMMITS: usize = 10;

/// Returns the lines representing recent commits in the Git repository
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
        head.shorthand().map(|s| s.to_string())
    };

    // Build a map of commit OID -> tag names
    let tag_map = build_tag_map(repository)?;

    // Build maps of commit OID -> branch names
    let local_branch_map = build_local_branch_map(repository)?;
    let remote_branch_map = build_remote_branch_map(repository)?;

    // Walk through commits
    let mut revwalk = repository.revwalk()?;
    revwalk.push(head_commit.id())?;

    let commits: Vec<_> = revwalk
        .take(MAX_COMMITS)
        .filter_map(|oid| oid.ok())
        .collect();

    if commits.is_empty() {
        return Ok(lines);
    }

    lines.push(Line {
        content: LineContent::SectionHeader {
            title: "Recent commits".to_string(),
            count: None,
        },
        section: Some(SectionType::RecentCommits),
    });

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
            });
        }

        // Sort refs with current branch first (if applicable)
        let refs = sort_refs(refs, current_branch.as_deref());

        lines.push(create_commit_line(
            &commit,
            refs,
            SectionType::RecentCommits,
        ));
    }

    Ok(lines)
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
}
