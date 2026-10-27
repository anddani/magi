use git2::{Delta, DiffOptions, Repository};

use crate::{
    errors::MagiResult,
    i18n,
    model::{Line, LineContent, SectionType},
};

use super::{
    diff_utils::{build_change_lines, collect_file_changes},
    unmerged_changes::collect_unmerged_from_paths,
    untracked_files::build_untracked_lines,
};

/// The untracked and unstaged sections, both derived from a single working-tree
/// walk.
pub struct WorkingTreeSections {
    pub untracked: Vec<Line>,
    pub unstaged: Vec<Line>,
}

/// Builds both the untracked-files and unstaged-changes sections from a single
/// `diff_index_to_workdir` walk.
///
/// Previously the status view scanned the whole working tree three times per
/// refresh: once via `statuses` for untracked files, once via the unstaged diff,
/// and once more via `statuses` for conflicted files. On a repository the size
/// of nixpkgs (≈55k files) each scan is ≈1.2–1.4s, so the status view took
/// ≈3.8s. libgit2 reports untracked and conflicted entries in the same
/// index-to-workdir diff it uses for modified files, so one walk yields all
/// three groups and the refresh drops to ≈1.3s.
pub fn get_sections(repository: &Repository) -> MagiResult<WorkingTreeSections> {
    // Match the flags the standalone untracked scan used: include untracked
    // files but not ignored ones, and don't recurse into untracked directories
    // (they're listed as a single entry, like before).
    let mut diff_options = DiffOptions::new();
    diff_options.include_untracked(true).include_ignored(false);

    let diff = repository.diff_index_to_workdir(None, Some(&mut diff_options))?;

    // Pull untracked and conflicted paths out of the diff's deltas so neither
    // needs its own scan of the working tree.
    let mut untracked_paths: Vec<String> = Vec::new();
    let mut conflicted_paths: Vec<String> = Vec::new();
    for delta in diff.deltas() {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_string_lossy().to_string());
        match delta.status() {
            Delta::Untracked => {
                if let Some(p) = path {
                    untracked_paths.push(p);
                }
            }
            Delta::Conflicted => {
                if let Some(p) = path {
                    conflicted_paths.push(p);
                }
            }
            _ => {}
        }
    }

    // Modified files with their hunks (untracked and conflicted deltas are
    // skipped by collect_file_changes), plus the combined diffs for conflicts.
    let mut file_changes = collect_file_changes(&diff)?;
    file_changes.extend(collect_unmerged_from_paths(repository, conflicted_paths)?);
    file_changes.sort_by(|(a, _), (b, _)| a.path.cmp(&b.path));

    let unstaged = build_change_lines(
        file_changes,
        i18n::t().section_unstaged_changes,
        SectionType::UnstagedChanges,
        LineContent::UnstagedFile,
        |path| SectionType::UnstagedFile { path },
        |path, hunk_index| SectionType::UnstagedHunk { path, hunk_index },
    );

    let untracked = build_untracked_lines(untracked_paths);

    Ok(WorkingTreeSections {
        untracked,
        unstaged,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    fn open(test_repo: &TestRepo) -> Repository {
        Repository::open(test_repo.repo_path()).unwrap()
    }

    #[test]
    fn test_clean_repo_has_no_sections() {
        let test_repo = TestRepo::new();
        let repo = open(&test_repo);

        let sections = get_sections(&repo).unwrap();
        assert!(sections.untracked.is_empty());
        assert!(sections.unstaged.is_empty());
    }

    #[test]
    fn test_untracked_and_unstaged_from_single_walk() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("tracked.txt", "one\ntwo\n", "Add tracked");
        // Modify a tracked file (unstaged) and add an untracked file.
        test_repo.write_file_content("tracked.txt", "one\nCHANGED\n");
        test_repo.write_file_content("untracked.txt", "hello\n");

        let repo = open(&test_repo);
        let sections = get_sections(&repo).unwrap();

        // Untracked section lists the new file.
        assert!(sections.untracked.iter().any(|l| matches!(
            &l.content,
            LineContent::UntrackedFile(p) if p == "untracked.txt"
        )));
        // The untracked file must NOT leak into the unstaged section.
        assert!(!sections.untracked.is_empty());

        // Unstaged section shows the modified tracked file with a hunk.
        assert!(sections.unstaged.iter().any(|l| matches!(
            &l.content,
            LineContent::UnstagedFile(fc) if fc.path == "tracked.txt"
        )));
        assert!(
            sections
                .unstaged
                .iter()
                .any(|l| matches!(&l.content, LineContent::DiffHunk(_)))
        );
        // And the untracked file is not present among the modified files.
        assert!(!sections.unstaged.iter().any(|l| matches!(
            &l.content,
            LineContent::UnstagedFile(fc) if fc.path == "untracked.txt"
        )));
    }

    #[test]
    fn test_conflicted_file_appears_in_unstaged() {
        let test_repo = TestRepo::new();
        test_repo.create_merge_conflict("conflict.txt");

        let repo = open(&test_repo);
        let sections = get_sections(&repo).unwrap();

        // The conflicted file shows up as an unstaged (unmerged) change with a
        // combined diff hunk, without a separate statuses scan.
        assert!(sections.unstaged.iter().any(|l| matches!(
            &l.content,
            LineContent::UnstagedFile(fc) if fc.path == "conflict.txt"
        )));
        assert!(sections.unstaged.iter().any(
            |l| matches!(&l.content, LineContent::DiffHunk(h) if h.header.starts_with("@@@"))
        ));
    }
}
