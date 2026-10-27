use git2::Repository;

use crate::{
    errors::MagiResult,
    model::{Line, LineContent, SectionType},
};

use super::commit_utils::{
    build_local_branch_map, build_refs_for_commit, build_remote_branch_map, build_tag_map,
    create_commit_line,
};
use super::config::get_push_remote;

const MAX_COMMITS: usize = 10;

/// Resolves the push branch for HEAD as `(shorthand, commit_oid)` — e.g.
/// `("origin/main", <oid>)` — but only when a push remote is configured AND the push
/// branch differs from the upstream branch.
///
/// Mirrors magit's `magit--insert-pushremote-log-p`: the pushremote sections are
/// suppressed when the push branch is identical to the upstream, to avoid showing the
/// same commits twice (they are already covered by the upstream-based sections).
fn push_branch_target(repository: &Repository) -> Option<(String, git2::Oid)> {
    let head = repository.head().ok()?;
    if !head.is_branch() {
        return None;
    }
    let branch_ref = head.resolve().ok()?;

    // Short branch name (e.g. "main") used to look up the configured push remote.
    let branch_name = branch_ref.shorthand().ok()?.to_string();

    let push_remote = get_push_remote(repository, &branch_name)?;
    let push_ref_name = format!("refs/remotes/{}/{}", push_remote, branch_name);
    let push_ref = repository.find_reference(&push_ref_name).ok()?;

    // Suppress when the push branch equals the upstream branch.
    if let Ok(full_branch_name) = branch_ref.name()
        && let Ok(upstream) = repository.branch_upstream_name(full_branch_name)
        && upstream.as_str().ok() == Some(push_ref_name.as_str())
    {
        return None;
    }

    let push_commit = push_ref.peel_to_commit().ok()?;
    let shorthand = push_ref.shorthand().unwrap_or(&push_ref_name).to_string();
    Some((shorthand, push_commit.id()))
}

/// Walks `range` commits (configured by `push`/`hide`) and renders them as a section with
/// the given header, or returns an empty vec when there is nothing to show.
fn section_lines(
    repository: &Repository,
    header: LineContent,
    section: SectionType,
    push_oid: git2::Oid,
    hide_oid: git2::Oid,
) -> MagiResult<Vec<Line>> {
    let mut lines = Vec::new();

    let mut revwalk = repository.revwalk()?;
    revwalk.push(push_oid)?;
    revwalk.hide(hide_oid)?;
    let commit_oids: Vec<_> = revwalk
        .take(MAX_COMMITS)
        .filter_map(|oid| oid.ok())
        .collect();

    if commit_oids.is_empty() {
        return Ok(lines);
    }

    let tag_map = build_tag_map(repository)?;
    let local_branch_map = build_local_branch_map(repository)?;
    let remote_branch_map = build_remote_branch_map(repository)?;

    // The caller passes a header with a placeholder count; fill in the real one.
    let header = match header {
        LineContent::UnpushedSectionHeader { remote_name, .. } => {
            LineContent::UnpushedSectionHeader {
                remote_name,
                count: commit_oids.len(),
            }
        }
        LineContent::UnpulledSectionHeader { remote_name, .. } => {
            LineContent::UnpulledSectionHeader {
                remote_name,
                count: commit_oids.len(),
            }
        }
        other => other,
    };

    lines.push(Line {
        content: header,
        section: Some(section.clone()),
    });

    for oid in &commit_oids {
        let commit = match repository.find_commit(*oid) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let refs = build_refs_for_commit(oid, &local_branch_map, &remote_branch_map, &tag_map);
        lines.push(create_commit_line(&commit, refs, section.clone()));
    }

    Ok(lines)
}

/// Returns the "Unpushed to <push-remote>" section: commits on HEAD that have not yet
/// been pushed to the push-remote branch (`@{push}..HEAD`).
pub fn get_unpushed_lines(repository: &Repository) -> MagiResult<Vec<Line>> {
    let Some((shorthand, push_oid)) = push_branch_target(repository) else {
        return Ok(Vec::new());
    };

    let head_commit = match repository.head().and_then(|h| h.peel_to_commit()) {
        Ok(c) => c,
        Err(_) => return Ok(Vec::new()),
    };

    section_lines(
        repository,
        LineContent::UnpushedSectionHeader {
            remote_name: shorthand,
            count: 0,
        },
        SectionType::UnpushedToPushremote,
        head_commit.id(), // commits reachable from HEAD...
        push_oid,         // ...but not from the push branch
    )
}

/// Returns the "Unpulled from <push-remote>" section: commits on the push-remote branch
/// that are not yet in HEAD (`..@{push}`).
pub fn get_unpulled_lines(repository: &Repository) -> MagiResult<Vec<Line>> {
    let Some((shorthand, push_oid)) = push_branch_target(repository) else {
        return Ok(Vec::new());
    };

    let head_commit = match repository.head().and_then(|h| h.peel_to_commit()) {
        Ok(c) => c,
        Err(_) => return Ok(Vec::new()),
    };

    section_lines(
        repository,
        LineContent::UnpulledSectionHeader {
            remote_name: shorthand,
            count: 0,
        },
        SectionType::UnpulledFromPushremote,
        push_oid,         // commits reachable from the push branch...
        head_commit.id(), // ...but not from HEAD
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;
    use git2::{BranchType, Signature};
    use std::fs;

    /// Configure `branch.main.pushRemote = origin` and create the push-tracking ref
    /// `refs/remotes/origin/main` pointing at `push_oid`.
    fn setup_push_remote(repo: &Repository, push_oid: git2::Oid) {
        repo.remote("origin", "https://example.com/repo.git")
            .unwrap();
        repo.reference("refs/remotes/origin/main", push_oid, false, "push branch")
            .unwrap();
        repo.config()
            .unwrap()
            .set_str("branch.main.pushRemote", "origin")
            .unwrap();
    }

    /// Create a commit on the given ref (e.g. "refs/remotes/origin/main").
    fn create_commit_on_ref(
        repo: &Repository,
        ref_name: &str,
        message: &str,
        parent_oid: git2::Oid,
    ) {
        let sig = Signature::now("Test", "test@test.com").unwrap();
        let parent = repo.find_commit(parent_oid).unwrap();
        let path = repo.workdir().unwrap().join("remote_dummy.txt");
        fs::write(&path, message).unwrap();
        let mut index = repo.index().unwrap();
        index
            .add_path(std::path::Path::new("remote_dummy.txt"))
            .unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let new_tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some(ref_name), &sig, &sig, message, &new_tree, &[&parent])
            .unwrap();
    }

    #[test]
    fn test_no_push_remote_configured() {
        let test_repo = TestRepo::new();
        assert!(get_unpushed_lines(&test_repo.repo).unwrap().is_empty());
        assert!(get_unpulled_lines(&test_repo.repo).unwrap().is_empty());
    }

    #[test]
    fn test_unpushed_to_pushremote() {
        let file = "f.txt";
        let test_repo = TestRepo::new();

        // Pin the push branch at the initial commit, then advance HEAD past it.
        let push_oid = test_repo
            .repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id();
        setup_push_remote(&test_repo.repo, push_oid);

        test_repo
            .create_file(file)
            .stage_files(&[file])
            .commit("Commit A");
        test_repo.write_file_content(file, "b").commit("Commit B");

        let lines = get_unpushed_lines(&test_repo.repo).unwrap();

        assert_eq!(lines.len(), 3);
        match &lines[0].content {
            LineContent::UnpushedSectionHeader { remote_name, count } => {
                assert_eq!(remote_name, "origin/main");
                assert_eq!(*count, 2);
            }
            _ => panic!("Expected UnpushedSectionHeader"),
        }
        if let LineContent::Commit(ref info) = lines[1].content {
            assert_eq!(info.message, "Commit B");
        } else {
            panic!("Expected Commit content");
        }
        for line in &lines {
            assert_eq!(line.section, Some(SectionType::UnpushedToPushremote));
        }

        // Nothing to pull in this direction.
        assert!(get_unpulled_lines(&test_repo.repo).unwrap().is_empty());
    }

    #[test]
    fn test_unpulled_from_pushremote() {
        let test_repo = TestRepo::new();
        let head_oid = test_repo
            .repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id();

        test_repo
            .repo
            .remote("origin", "https://example.com/repo.git")
            .unwrap();
        // Push branch is ahead of HEAD by one commit.
        create_commit_on_ref(
            &test_repo.repo,
            "refs/remotes/origin/main",
            "Pushed commit",
            head_oid,
        );
        test_repo
            .repo
            .config()
            .unwrap()
            .set_str("branch.main.pushRemote", "origin")
            .unwrap();

        let lines = get_unpulled_lines(&test_repo.repo).unwrap();

        assert_eq!(lines.len(), 2);
        match &lines[0].content {
            LineContent::UnpulledSectionHeader { remote_name, count } => {
                assert_eq!(remote_name, "origin/main");
                assert_eq!(*count, 1);
            }
            _ => panic!("Expected UnpulledSectionHeader"),
        }
        for line in &lines {
            assert_eq!(line.section, Some(SectionType::UnpulledFromPushremote));
        }

        // Nothing to push in this direction.
        assert!(get_unpushed_lines(&test_repo.repo).unwrap().is_empty());
    }

    #[test]
    fn test_suppressed_when_push_branch_equals_upstream() {
        let file = "f.txt";
        let test_repo = TestRepo::new();
        let head_oid = test_repo
            .repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id();

        // origin/main is BOTH the upstream and the push remote → identical ref.
        setup_push_remote(&test_repo.repo, head_oid);
        test_repo
            .repo
            .find_branch("main", BranchType::Local)
            .unwrap()
            .set_upstream(Some("origin/main"))
            .unwrap();

        // Advance HEAD so there *would* be unpushed commits if not suppressed.
        test_repo
            .create_file(file)
            .stage_files(&[file])
            .commit("Commit A");

        assert!(get_unpushed_lines(&test_repo.repo).unwrap().is_empty());
        assert!(get_unpulled_lines(&test_repo.repo).unwrap().is_empty());
    }
}
