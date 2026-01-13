use crate::{
    errors::MagiResult,
    model::{Line, LineContent, SectionType},
};
use git2::{DiffOptions, Repository};

use super::diff_utils::{build_change_lines, collect_file_changes};

/// Returns the lines representing staged changes in the Git repository
pub fn get_lines(repository: &Repository) -> MagiResult<Vec<Line>> {
    // Get the diff between HEAD and index (staged changes)
    let head = repository.head()?.peel_to_tree()?;
    let mut diff_options = DiffOptions::new();
    let diff = repository.diff_tree_to_index(Some(&head), None, Some(&mut diff_options))?;

    let file_changes = collect_file_changes(&diff)?;

    Ok(build_change_lines(
        file_changes,
        "Staged changes",
        SectionType::StagedChanges,
        LineContent::StagedFile,
        |path| SectionType::StagedFile { path },
        |path, hunk_index| SectionType::StagedHunk { path, hunk_index },
    ))
}
