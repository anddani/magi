use std::fs;
use std::path::Path;

use super::git_cmd;
use crate::{
    errors::{MagiError, MagiResult},
    git::{commit::get_commit_result, read_commit_message},
    i18n,
    model::{LineContent, SectionType},
};

pub use super::commit::CommitResult;

/// Action to perform on a commit in an interactive rebase todo list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebaseAction {
    Pick,
    Reword,
    Edit,
    Squash,
    Fixup,
    Drop,
}

impl RebaseAction {
    /// The action word used in the git-rebase-todo file
    pub fn as_str(self) -> &'static str {
        match self {
            RebaseAction::Pick => "pick",
            RebaseAction::Reword => "reword",
            RebaseAction::Edit => "edit",
            RebaseAction::Squash => "squash",
            RebaseAction::Fixup => "fixup",
            RebaseAction::Drop => "drop",
        }
    }

    /// Returns true if the action folds the commit into the previous one,
    /// meaning it cannot be the first entry in the todo list.
    pub fn is_fold(self) -> bool {
        matches!(self, RebaseAction::Squash | RebaseAction::Fixup)
    }
}

/// A single line of an interactive rebase todo list being edited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebaseTodoEntry {
    pub action: RebaseAction,
    /// Full commit hash
    pub hash: String,
    /// Commit subject line
    pub message: String,
}

/// Returns true if the given commit has a parent (i.e. is not a root commit).
pub fn commit_has_parent(workdir: &Path, commit: &str) -> bool {
    git_cmd(
        workdir,
        &["rev-parse", "--verify", "--quiet", &format!("{commit}^")],
    )
    .output()
    .is_ok_and(|output| output.status.success())
}

/// Resolves a (possibly abbreviated) commit-ish to its full commit hash.
fn resolve_commit_hash(workdir: &Path, commit: &str) -> MagiResult<String> {
    let output = git_cmd(
        workdir,
        &["rev-parse", "--verify", &format!("{commit}^{{commit}}")],
    )
    .output()?;

    if !output.status.success() {
        return Err(MagiError::Generic(format!(
            "Failed to resolve commit {}: {}",
            commit,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Starts an interactive rebase that stops at `commit` so it can be modified:
/// the todo marks `commit` as `edit` and picks every commit after it.
/// The rebase stops right after applying `commit`, leaving it as HEAD for
/// amending; `git rebase --continue` then replays the remaining commits.
pub fn run_modify_commit(workdir: &Path, commit: &str) -> MagiResult<CommitResult> {
    run_action_on_commit(workdir, commit, RebaseAction::Edit, "modified")
}

/// Starts an interactive rebase that rewords `commit`: the todo marks
/// `commit` as `reword` and picks every commit after it. Git opens the
/// user's editor for the new message, then the rebase finishes on its own.
pub fn run_reword_commit(workdir: &Path, commit: &str) -> MagiResult<CommitResult> {
    run_action_on_commit(workdir, commit, RebaseAction::Reword, "reworded")
}

/// Starts an interactive rebase where `commit` is marked with `action` and
/// every commit after it is picked. `action_desc` is the past-tense verb
/// used in the error message when the commit is not in the todo.
fn run_action_on_commit(
    workdir: &Path,
    commit: &str,
    action: RebaseAction,
    action_desc: &str,
) -> MagiResult<CommitResult> {
    let full_hash = resolve_commit_hash(workdir, commit)?;
    let has_parent = commit_has_parent(workdir, &full_hash);
    let mut entries = get_interactive_rebase_commits(workdir, &full_hash, has_parent)?;

    let mut found = false;
    for entry in &mut entries {
        if entry.hash == full_hash {
            entry.action = action;
            found = true;
        }
    }
    if !found {
        return Err(MagiError::Generic(format!(
            "Commit {} cannot be {} (not reachable from HEAD, or a merge commit)",
            commit, action_desc
        )));
    }

    start_interactive_rebase(workdir, &full_hash, has_parent, &entries)
}

/// Returns the initial todo entries for an interactive rebase that includes
/// `base` and every commit after it up to HEAD (oldest first, all `pick`).
/// Merge commits are excluded, matching what `git rebase -i` generates
/// without `--rebase-merges`.
pub fn get_interactive_rebase_commits(
    workdir: &Path,
    base: &str,
    base_has_parent: bool,
) -> MagiResult<Vec<RebaseTodoEntry>> {
    let range = if base_has_parent {
        format!("{base}^..HEAD")
    } else {
        "HEAD".to_string()
    };
    let output = git_cmd(
        workdir,
        &[
            "log",
            "--reverse",
            "--no-merges",
            "--format=%H%x09%s",
            &range,
        ],
    )
    .output()?;

    if !output.status.success() {
        return Err(MagiError::Generic(format!(
            "Failed to list commits: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let entries = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let (hash, message) = line.split_once('\t')?;
            Some(RebaseTodoEntry {
                action: RebaseAction::Pick,
                hash: hash.to_string(),
                message: message.to_string(),
            })
        })
        .collect();

    Ok(entries)
}

/// Starts an interactive rebase with a pre-built todo list.
///
/// The entries are written to a file inside `.git/`, and `sequence.editor`
/// is set to a command that copies that file over git's generated todo.
/// Git stays authoritative for executing the rebase; reword/squash may open
/// the user's `$GIT_EDITOR`, so the caller must run this while the TUI is
/// suspended (via `RunningState::LaunchExternalCommand`).
pub fn start_interactive_rebase(
    workdir: &Path,
    base: &str,
    base_has_parent: bool,
    entries: &[RebaseTodoEntry],
) -> MagiResult<CommitResult> {
    let todo_path = workdir.join(".git").join("magi-rebase-todo");
    let content: String = entries
        .iter()
        .map(|e| format!("{} {} {}\n", e.action.as_str(), e.hash, e.message))
        .collect();
    fs::write(&todo_path, content)?;

    // Git runs the sequence editor through the shell with the todo file path
    // appended, so this becomes `cp '<our file>' <git's todo file>`.
    let quoted_path = todo_path.display().to_string().replace('\'', "'\\''");
    let sequence_editor = format!("sequence.editor=cp '{quoted_path}'");

    let mut args = vec![
        "-c",
        sequence_editor.as_str(),
        "rebase",
        "--interactive",
        "--autostash",
    ];
    let parent = format!("{base}^");
    if base_has_parent {
        args.push(&parent);
    } else {
        args.push("--root");
    }

    let status = git_cmd(workdir, &args).status();
    let _ = fs::remove_file(&todo_path);

    get_commit_result(workdir, status?, "Rebase")
}

/// Returns true if a rebase sequence is currently in progress.
/// Checks for `rebase-merge/` directory (interactive) or `rebase-apply/onto` (patch-based).
pub fn rebase_in_progress(workdir: &Path) -> bool {
    let git_dir = workdir.join(".git");
    git_dir.join("rebase-merge").is_dir() || git_dir.join("rebase-apply").join("onto").exists()
}

/// A single entry shown in the "Rebasing" section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebasingEntry {
    pub hash: String,
    pub message: String,
    /// true = the commit currently stopped on (conflict); false = pending in todo
    pub is_current: bool,
}

/// Returns the list of rebasing entries when a rebase sequence is in progress.
/// The stopped commit (if any) comes first, followed by pending todo entries.
pub fn get_rebasing_entries(workdir: &Path) -> Vec<RebasingEntry> {
    let git_dir = workdir.join(".git");
    let merge_dir = git_dir.join("rebase-merge");
    let mut entries = Vec::new();

    if !merge_dir.is_dir() {
        return entries;
    }

    // Current stopped commit from stopped-sha
    let stopped_sha_path = merge_dir.join("stopped-sha");
    if let Ok(hash_raw) = fs::read_to_string(&stopped_sha_path) {
        let hash = hash_raw.trim().to_string();
        if !hash.is_empty() {
            let short_hash: String = hash.chars().take(7).collect();
            let message = read_commit_message(workdir, &hash).unwrap_or_default();
            entries.push(RebasingEntry {
                hash: short_hash,
                message,
                is_current: true,
            });
        }
    }

    // Pending commits from git-rebase-todo
    let todo_path = merge_dir.join("git-rebase-todo");
    if let Ok(content) = fs::read_to_string(&todo_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Format: "<cmd> <hash> <message>"
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() < 2 {
                continue;
            }
            // Only show commit-based commands; skip exec/label/reset/merge/break
            match parts[0] {
                "pick" | "p" | "edit" | "e" | "reword" | "r" | "squash" | "s" | "fixup" | "f"
                | "drop" | "d" => {}
                _ => continue,
            }
            let short_hash: String = parts[1].chars().take(7).collect();
            let message = parts
                .get(2)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            entries.push(RebasingEntry {
                hash: short_hash,
                message,
                is_current: false,
            });
        }
    }

    entries
}

/// Returns model lines for the "Rebasing" section.
/// Returns an empty vec if no rebase is in progress.
pub fn get_rebasing_lines(workdir: &Path) -> MagiResult<Vec<crate::model::Line>> {
    if !rebase_in_progress(workdir) {
        return Ok(vec![]);
    }

    let entries = get_rebasing_entries(workdir);
    if entries.is_empty() {
        return Ok(vec![]);
    }

    let mut lines = Vec::new();

    lines.push(crate::model::Line {
        content: LineContent::SectionHeader {
            title: i18n::t().section_rebasing.to_string(),
            count: None,
        },
        section: Some(SectionType::Rebasing),
    });

    for entry in entries {
        lines.push(crate::model::Line {
            content: LineContent::RebasingEntry {
                hash: entry.hash,
                message: entry.message,
                is_current: entry.is_current,
            },
            section: Some(SectionType::Rebasing),
        });
    }

    Ok(lines)
}

/// Runs `git rebase --continue` which opens the user's configured editor
/// to edit the commit message after resolving conflicts.
pub fn run_rebase_continue_with_editor<P: AsRef<Path>>(repo_path: P) -> MagiResult<CommitResult> {
    let status = git_cmd(&repo_path, &["rebase", "--continue"]).status()?;

    get_commit_result(repo_path, status, "Rebase continue")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    /// Points GIT_EDITOR at a command that replaces the commit message with
    /// "Reworded". The env var outranks core.editor, so any GIT_EDITOR from
    /// the developer's environment would otherwise leak into reword tests
    /// (or, if unset, git would open a real editor and hang).
    fn set_reword_editor() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| unsafe { std::env::set_var("GIT_EDITOR", "echo 'Reworded' >") });
    }

    /// Commit subjects, newest first.
    fn log_subjects(workdir: &Path) -> Vec<String> {
        let output = git_cmd(workdir, &["log", "--format=%s"]).output().unwrap();
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn test_commit_has_parent() {
        let test_repo = TestRepo::new();
        let root_hash = test_repo.head_hash();
        test_repo.commit_file("a.txt", "a", "Commit A");
        let second_hash = test_repo.head_hash();
        let workdir = test_repo.repo_path();

        assert!(!commit_has_parent(workdir, &root_hash));
        assert!(commit_has_parent(workdir, &second_hash));
    }

    #[test]
    fn test_get_interactive_rebase_commits_lists_base_to_head_oldest_first() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("a.txt", "a", "Commit A");
        let base_hash = test_repo.head_hash();
        test_repo.commit_file("b.txt", "b", "Commit B");

        let entries =
            get_interactive_rebase_commits(test_repo.repo_path(), &base_hash, true).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].message, "Commit A");
        assert_eq!(entries[0].hash, base_hash);
        assert_eq!(entries[0].action, RebaseAction::Pick);
        assert_eq!(entries[1].message, "Commit B");
        assert_eq!(entries[1].action, RebaseAction::Pick);
    }

    #[test]
    fn test_get_interactive_rebase_commits_from_root() {
        let test_repo = TestRepo::new();
        let root_hash = test_repo.head_hash();
        test_repo.commit_file("a.txt", "a", "Commit A");

        let entries =
            get_interactive_rebase_commits(test_repo.repo_path(), &root_hash, false).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].message, "Initial commit");
        assert_eq!(entries[1].message, "Commit A");
    }

    #[test]
    fn test_start_interactive_rebase_drop_removes_commit() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("a.txt", "a", "Commit A");
        let base_hash = test_repo.head_hash();
        test_repo.commit_file("b.txt", "b", "Commit B");
        let workdir = test_repo.repo_path();

        let mut entries = get_interactive_rebase_commits(workdir, &base_hash, true).unwrap();
        entries[0].action = RebaseAction::Drop;

        let result = start_interactive_rebase(workdir, &base_hash, true, &entries).unwrap();

        assert!(result.success);
        assert_eq!(log_subjects(workdir), vec!["Commit B", "Initial commit"]);
        assert!(!workdir.join("a.txt").exists());
        assert!(workdir.join("b.txt").exists());
    }

    #[test]
    fn test_start_interactive_rebase_fixup_folds_commit() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("a.txt", "a", "Commit A");
        let base_hash = test_repo.head_hash();
        test_repo.commit_file("b.txt", "b", "Commit B");
        let workdir = test_repo.repo_path();

        let mut entries = get_interactive_rebase_commits(workdir, &base_hash, true).unwrap();
        entries[1].action = RebaseAction::Fixup;

        let result = start_interactive_rebase(workdir, &base_hash, true, &entries).unwrap();

        assert!(result.success);
        // Commit B is folded into Commit A; its file changes survive
        assert_eq!(log_subjects(workdir), vec!["Commit A", "Initial commit"]);
        assert!(workdir.join("b.txt").exists());
    }

    #[test]
    fn test_start_interactive_rebase_reorders_commits() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("a.txt", "a", "Commit A");
        let base_hash = test_repo.head_hash();
        test_repo.commit_file("b.txt", "b", "Commit B");
        let workdir = test_repo.repo_path();

        let mut entries = get_interactive_rebase_commits(workdir, &base_hash, true).unwrap();
        entries.swap(0, 1);

        let result = start_interactive_rebase(workdir, &base_hash, true, &entries).unwrap();

        assert!(result.success);
        assert_eq!(
            log_subjects(workdir),
            vec!["Commit A", "Commit B", "Initial commit"]
        );
    }

    #[test]
    fn test_start_interactive_rebase_cleans_up_todo_file() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("a.txt", "a", "Commit A");
        let base_hash = test_repo.head_hash();
        let workdir = test_repo.repo_path();

        let entries = get_interactive_rebase_commits(workdir, &base_hash, true).unwrap();
        start_interactive_rebase(workdir, &base_hash, true, &entries).unwrap();

        assert!(!workdir.join(".git").join("magi-rebase-todo").exists());
    }

    #[test]
    fn test_run_modify_commit_stops_at_commit() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("a.txt", "a", "Commit A");
        let target = test_repo.head_hash();
        test_repo.commit_file("b.txt", "b", "Commit B");
        let workdir = test_repo.repo_path();

        let result = run_modify_commit(workdir, &target).unwrap();

        assert!(result.success);
        // The rebase stops at the target commit, leaving it as HEAD for amending
        assert!(rebase_in_progress(workdir));
        assert_eq!(log_subjects(workdir), vec!["Commit A", "Initial commit"]);
        // The following commit is still pending in the todo
        let entries = get_rebasing_entries(workdir);
        assert!(entries.iter().any(|e| e.message == "Commit B"));

        git_cmd(workdir, &["rebase", "--abort"]).status().unwrap();
    }

    #[test]
    fn test_run_modify_commit_accepts_short_hash() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("a.txt", "a", "Commit A");
        let target = test_repo.head_hash();
        test_repo.commit_file("b.txt", "b", "Commit B");
        let workdir = test_repo.repo_path();

        let result = run_modify_commit(workdir, &target[..7]).unwrap();

        assert!(result.success);
        assert!(rebase_in_progress(workdir));
        assert_eq!(log_subjects(workdir), vec!["Commit A", "Initial commit"]);

        git_cmd(workdir, &["rebase", "--abort"]).status().unwrap();
    }

    #[test]
    fn test_run_modify_commit_on_root_commit() {
        let test_repo = TestRepo::new();
        let root = test_repo.head_hash();
        test_repo.commit_file("a.txt", "a", "Commit A");
        let workdir = test_repo.repo_path();

        let result = run_modify_commit(workdir, &root).unwrap();

        assert!(result.success);
        assert!(rebase_in_progress(workdir));
        assert_eq!(log_subjects(workdir), vec!["Initial commit"]);

        git_cmd(workdir, &["rebase", "--abort"]).status().unwrap();
    }

    #[test]
    fn test_run_modify_commit_unknown_commit_fails() {
        let test_repo = TestRepo::new();
        let workdir = test_repo.repo_path();

        let result = run_modify_commit(workdir, "0000000000000000000000000000000000000000");

        assert!(result.is_err());
        assert!(!rebase_in_progress(workdir));
    }

    #[test]
    fn test_run_reword_commit_rewords_message() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("a.txt", "a", "Commit A");
        let target = test_repo.head_hash();
        test_repo.commit_file("b.txt", "b", "Commit B");
        let workdir = test_repo.repo_path();

        set_reword_editor();
        let result = run_reword_commit(workdir, &target).unwrap();

        assert!(result.success);
        // The rebase runs to completion; only the message changes
        assert!(!rebase_in_progress(workdir));
        assert_eq!(
            log_subjects(workdir),
            vec!["Commit B", "Reworded", "Initial commit"]
        );
        assert!(workdir.join("a.txt").exists());
        assert!(workdir.join("b.txt").exists());
    }

    #[test]
    fn test_run_reword_commit_on_root_commit() {
        let test_repo = TestRepo::new();
        let root = test_repo.head_hash();
        test_repo.commit_file("a.txt", "a", "Commit A");
        let workdir = test_repo.repo_path();

        set_reword_editor();
        let result = run_reword_commit(workdir, &root).unwrap();

        assert!(result.success);
        assert!(!rebase_in_progress(workdir));
        assert_eq!(log_subjects(workdir), vec!["Commit A", "Reworded"]);
    }

    #[test]
    fn test_run_reword_commit_unknown_commit_fails() {
        let test_repo = TestRepo::new();
        let workdir = test_repo.repo_path();

        let result = run_reword_commit(workdir, "0000000000000000000000000000000000000000");

        assert!(result.is_err());
        assert!(!rebase_in_progress(workdir));
    }

    #[test]
    fn test_rebase_in_progress_no_files() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!rebase_in_progress(dir.path()));
    }

    #[test]
    fn test_rebase_in_progress_with_rebase_merge_dir() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_merge = git_dir.join("rebase-merge");
        fs::create_dir_all(&rebase_merge).unwrap();
        assert!(rebase_in_progress(dir.path()));
    }

    #[test]
    fn test_rebase_in_progress_with_rebase_apply_onto() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_apply = git_dir.join("rebase-apply");
        fs::create_dir_all(&rebase_apply).unwrap();
        fs::write(rebase_apply.join("onto"), "abc1234\n").unwrap();
        assert!(rebase_in_progress(dir.path()));
    }

    #[test]
    fn test_rebase_not_in_progress_without_rebase_apply_onto() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        // Create rebase-apply dir but without "onto" file
        let rebase_apply = git_dir.join("rebase-apply");
        fs::create_dir_all(&rebase_apply).unwrap();
        assert!(!rebase_in_progress(dir.path()));
    }

    #[test]
    fn test_get_rebasing_entries_with_stopped_sha() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_merge = git_dir.join("rebase-merge");
        fs::create_dir_all(&rebase_merge).unwrap();
        fs::write(rebase_merge.join("stopped-sha"), "abc1234abcdef\n").unwrap();

        let entries = get_rebasing_entries(dir.path());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].hash, "abc1234");
        assert!(entries[0].is_current);
    }

    #[test]
    fn test_get_rebasing_entries_with_todo() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_merge = git_dir.join("rebase-merge");
        fs::create_dir_all(&rebase_merge).unwrap();
        fs::write(
            rebase_merge.join("git-rebase-todo"),
            "pick abc1234abcdef Fix bug\npick def5678abcdef Add feature\n# comment line\n",
        )
        .unwrap();

        let entries = get_rebasing_entries(dir.path());
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].hash, "abc1234");
        assert_eq!(entries[0].message, "Fix bug");
        assert!(!entries[0].is_current);
        assert_eq!(entries[1].hash, "def5678");
        assert_eq!(entries[1].message, "Add feature");
        assert!(!entries[1].is_current);
    }

    #[test]
    fn test_get_rebasing_entries_skips_non_commit_commands() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_merge = git_dir.join("rebase-merge");
        fs::create_dir_all(&rebase_merge).unwrap();
        fs::write(
            rebase_merge.join("git-rebase-todo"),
            "pick abc1234abcdef Fix bug\nexec echo hello\nlabel my-label\npick def5678abcdef Add feature\n",
        )
        .unwrap();

        let entries = get_rebasing_entries(dir.path());
        // Only commit-based entries
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].hash, "abc1234");
        assert_eq!(entries[1].hash, "def5678");
    }

    #[test]
    fn test_get_rebasing_entries_stopped_and_todo() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_merge = git_dir.join("rebase-merge");
        fs::create_dir_all(&rebase_merge).unwrap();
        fs::write(rebase_merge.join("stopped-sha"), "aaa1111abcdef\n").unwrap();
        fs::write(
            rebase_merge.join("git-rebase-todo"),
            "pick bbb2222abcdef Next commit\n",
        )
        .unwrap();

        let entries = get_rebasing_entries(dir.path());
        assert_eq!(entries.len(), 2);
        assert!(entries[0].is_current);
        assert_eq!(entries[0].hash, "aaa1111");
        assert!(!entries[1].is_current);
        assert_eq!(entries[1].hash, "bbb2222");
    }

    #[test]
    fn test_get_rebasing_lines_empty_when_not_in_progress() {
        let dir = tempfile::tempdir().unwrap();
        let lines = get_rebasing_lines(dir.path()).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn test_get_rebasing_lines_returns_section_and_entries() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_merge = git_dir.join("rebase-merge");
        fs::create_dir_all(&rebase_merge).unwrap();
        fs::write(rebase_merge.join("stopped-sha"), "abc1234abcdef\n").unwrap();

        let lines = get_rebasing_lines(dir.path()).unwrap();
        // Section header + 1 entry
        assert_eq!(lines.len(), 2);
        assert!(matches!(
            &lines[0].content,
            crate::model::LineContent::SectionHeader { title, .. } if title == "Rebasing"
        ));
        assert!(matches!(
            &lines[1].content,
            crate::model::LineContent::RebasingEntry {
                is_current: true,
                ..
            }
        ));
    }
}
