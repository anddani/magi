use std::path::Path;
use std::process::Command;

use git2::{DiffOptions, Error as Git2Error, Repository};

use crate::{errors::MagiResult, model::LineContent};

/// Creates a `Command` for git with `-C <repo_path>` and the given args pre-configured.
pub fn git_cmd<P: AsRef<Path>>(repo_path: P, args: &[&str]) -> Command {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo_path.as_ref().as_os_str()).args(args);
    cmd
}

/// Reads the subject line of a commit from git log.
pub fn read_commit_message(workdir: &Path, hash: &str) -> Option<String> {
    let output = git_cmd(workdir, &["log", "--format=%s", "-1", hash])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub mod checkout;
pub mod cherry_pick;
pub mod commit;
mod commit_utils;
pub mod config;
pub mod credential;
mod diff_utils;
pub mod discard;
pub mod file_checkout;
pub mod info;
pub mod log;
pub mod merge;
pub mod open_pr;
pub mod preview;
pub mod pty_command;
pub mod push;
pub mod rebase;
pub mod recent_commits;
pub mod reset;
pub mod revert;
pub mod stage;
pub mod staged_changes;
pub mod stashes;
pub mod test_repo;
pub mod unpulled_commits;
pub mod unstaged_changes;
pub mod untracked_files;
pub mod worktree;

pub struct GitInfo {
    pub repository: Repository,
}

impl GitInfo {
    pub fn new() -> Result<Self, Git2Error> {
        let repository = Repository::open(".")?;
        Ok(Self { repository })
    }

    pub fn new_from_path<P: AsRef<Path>>(path: P) -> Result<Self, Git2Error> {
        let repository = Repository::open(path)?;
        Ok(Self { repository })
    }

    pub fn has_staged_changes(&self) -> MagiResult<bool> {
        let head = self.repository.head()?.peel_to_tree()?;
        let mut diff_options = DiffOptions::new();
        let diff =
            self.repository
                .diff_tree_to_index(Some(&head), None, Some(&mut diff_options))?;
        Ok(diff.deltas().count() > 0)
    }

    /// Gets lines from each section of GitInfo and joins them into a single Vec<Line>,
    /// inserting an empty line between each section.
    pub fn get_lines(&self) -> MagiResult<Vec<crate::model::Line>> {
        let empty_line = crate::model::Line {
            content: LineContent::EmptyLine,
            section: None,
        };

        let workdir = self
            .repository
            .workdir()
            .unwrap_or_else(|| std::path::Path::new("."));

        let info_lines = info::get_lines(&self.repository)?;
        let rebasing_lines = rebase::get_rebasing_lines(workdir)?;
        let reverting_lines = revert::get_reverting_lines(workdir)?;
        let cherry_picking_lines = cherry_pick::get_cherry_picking_lines(workdir)?;
        let untracked_files = untracked_files::get_lines(&self.repository)?;
        let unstaged_changes = unstaged_changes::get_lines(&self.repository)?;
        let staged_changes = staged_changes::get_lines(&self.repository)?;
        let stashes = stashes::get_lines(&self.repository)?;
        let unpulled_commits = unpulled_commits::get_lines(&self.repository)?;
        let recent_commits = recent_commits::get_lines(&self.repository)?;

        let all_sections = [
            info_lines,
            rebasing_lines,
            reverting_lines,
            cherry_picking_lines,
            untracked_files,
            unstaged_changes,
            staged_changes,
            stashes,
            unpulled_commits,
            recent_commits,
        ];
        let result = all_sections
            .into_iter()
            .filter(|section| !section.is_empty())
            .collect::<Vec<_>>()
            .join(&[empty_line.clone()][..]);

        Ok(result)
    }

    pub fn current_branch(&self) -> Option<String> {
        self.repository
            .head()
            .ok()
            .and_then(|head| head.shorthand().map(String::from))
    }
}

/// Represents a Git reference with its name, commit hash, and message
#[derive(Debug, Clone)]
pub struct GitRef {
    pub name: String,
    pub commit_hash: String,
    pub commit_summary: String,
    pub reference_type: ReferenceType,
}

/// Represents a git tag with name and number of commits ahead
#[derive(Debug, Clone)]
pub struct TagInfo {
    pub name: String,
    pub commits_ahead: usize,
}

/// Represents a commit in the recent commits list
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Short commit hash (7 characters)
    pub hash: String,
    /// References pointing to this commit (branches, tags, HEAD), in display order
    pub refs: Vec<CommitRef>,
    /// Commit message (first line)
    pub message: String,
}

/// A reference (branch, tag, or HEAD) pointing to a commit
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitRef {
    pub name: String,
    pub ref_type: CommitRefType,
    /// For LocalBranch: the push remote name (e.g., "origin") when the push branch
    /// (`remote/local-name`) is at the same commit. Used to render a split-colored label.
    pub push_remote: Option<String>,
}

/// Type of reference pointing to a commit
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitRefType {
    /// "@" indicator for detached HEAD
    Head,
    /// Local branch (e.g., "main", "feature-branch")
    LocalBranch,
    /// Remote branch (e.g., "origin/main")
    RemoteBranch,
    /// Tag (e.g., "v1.0.0")
    Tag,
}

/// Enum representing different types of Git references
#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceType {
    LocalBranch,
    RemoteBranch,
    DetachedHead,
}

/// Represents a single stash entry in the stash stack
#[derive(Debug, Clone)]
pub struct StashEntry {
    /// Stash index (0-based), used to format "stash@{N}"
    pub index: usize,
    /// Stash message (e.g., "WIP on main: abc1234 commit message")
    pub message: String,
}

impl GitRef {
    /// Creates a new GitRef with the specified reference type
    pub fn new(
        name: String,
        commit_hash: String,
        commit_summary: String,
        reference_type: ReferenceType,
    ) -> Self {
        Self {
            name,
            commit_hash,
            commit_summary,
            reference_type,
        }
    }

    /// Creates a new GitRef for a remote branch
    pub fn new_remote_branch(name: String, commit_hash: String, commit_summary: String) -> Self {
        Self::new(
            name,
            commit_hash,
            commit_summary,
            ReferenceType::RemoteBranch,
        )
    }

    /// Creates a new GitRef for a detached HEAD
    pub fn new_detached_head(commit_hash: String, commit_summary: String) -> Self {
        Self::new(
            "HEAD (detached)".to_string(),
            commit_hash,
            commit_summary,
            ReferenceType::DetachedHead,
        )
    }
}
