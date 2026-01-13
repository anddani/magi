use crate::{
    errors::MagiResult,
    model::{Line, LineContent, SectionType},
};
use git2::{DiffOptions, Repository};

use super::diff_utils::{build_change_lines, collect_file_changes};

/// Returns the lines representing unstaged changes in the Git repository
pub fn get_lines(repository: &Repository) -> MagiResult<Vec<Line>> {
    // Get the diff between index and workdir (unstaged changes)
    let mut diff_options = DiffOptions::new();
    diff_options.include_untracked(false);

    let diff = repository.diff_index_to_workdir(None, Some(&mut diff_options))?;

    let file_changes = collect_file_changes(&diff)?;

    Ok(build_change_lines(
        file_changes,
        "Unstaged changes",
        SectionType::UnstagedChanges,
        LineContent::UnstagedFile,
        |path| SectionType::UnstagedFile { path },
        |path, hunk_index| SectionType::UnstagedHunk { path, hunk_index },
    ))
}
