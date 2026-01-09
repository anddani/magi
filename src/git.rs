use git2::{DiffOptions, Error as Git2Error, Repository};
use std::path::Path;

use crate::{errors::MagiResult, model::LineContent};

pub mod commit;
mod diff_utils;
mod info;
pub mod stage;
mod staged_changes;
#[cfg(test)]
mod test_repo;
mod unstaged_changes;
mod untracked_files;

pub struct GitInfo {
    pub repository: Repository,
}

impl GitInfo {
    pub fn new() -> Result<Self, Git2Error> {
        let repository = Repository::open(".")?;
        Ok(Self { repository })
    }

    pub fn _new_from_path<P: AsRef<Path>>(path: P) -> Result<Self, Git2Error> {
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

        let lines = info::get_lines(&self.repository)?;
        let untracked_files = untracked_files::get_lines(&self.repository)?;
        let unstaged_changes = unstaged_changes::get_lines(&self.repository)?;
        let staged_changes = staged_changes::get_lines(&self.repository)?;

        let all_sections = [lines, untracked_files, unstaged_changes, staged_changes];
        let result = all_sections
            .into_iter()
            .filter(|section| !section.is_empty())
            .collect::<Vec<_>>()
            .join(&[empty_line.clone()][..]);

        Ok(result)
    }
}

/// Represents a Git reference with its name, commit hash, and message
#[derive(Debug, Clone)]
pub struct GitRef {
    pub name: String,
    pub commit_hash: String,
    pub commit_message: String,
    pub reference_type: ReferenceType,
}

/// Represents a git tag with name and number of commits ahead
#[derive(Debug, Clone)]
pub struct TagInfo {
    pub name: String,
    pub commits_ahead: usize,
}

/// Enum representing different types of Git references
#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceType {
    LocalBranch,
    RemoteBranch,
    DetachedHead,
}

impl GitRef {
    /// Creates a new GitRef with the specified reference type
    pub fn new(
        name: String,
        commit_hash: String,
        commit_message: String,
        reference_type: ReferenceType,
    ) -> Self {
        Self {
            name,
            commit_hash,
            commit_message,
            reference_type,
        }
    }

    /// Creates a new GitRef for a remote branch
    pub fn new_remote_branch(name: String, commit_hash: String, commit_message: String) -> Self {
        Self::new(
            name,
            commit_hash,
            commit_message,
            ReferenceType::RemoteBranch,
        )
    }

    /// Creates a new GitRef for a detached HEAD
    pub fn new_detached_head(commit_hash: String, commit_message: String) -> Self {
        Self::new(
            "HEAD (detached)".to_string(),
            commit_hash,
            commit_message,
            ReferenceType::DetachedHead,
        )
    }
}
