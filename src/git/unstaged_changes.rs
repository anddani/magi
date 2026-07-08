use crate::{
    errors::MagiResult,
    i18n,
    model::{Line, LineContent, SectionType},
};
use git2::{DiffOptions, Repository};

use super::{
    diff_utils::{build_change_lines, collect_file_changes},
    unmerged_changes::collect_unmerged_changes,
};

/// Returns the lines representing unstaged changes in the Git repository
pub fn get_lines(repository: &Repository) -> MagiResult<Vec<Line>> {
    // Get the diff between index and workdir (unstaged changes)
    let mut diff_options = DiffOptions::new();
    diff_options.include_untracked(false);

    let diff = repository.diff_index_to_workdir(None, Some(&mut diff_options))?;

    let mut file_changes = collect_file_changes(&diff)?;

    // Unmerged (conflicted) files are shown alongside unstaged changes,
    // like in Magit, with their combined diff
    file_changes.extend(collect_unmerged_changes(repository)?);
    file_changes.sort_by(|(a, _), (b, _)| a.path.cmp(&b.path));

    Ok(build_change_lines(
        file_changes,
        i18n::t().section_unstaged_changes,
        SectionType::UnstagedChanges,
        LineContent::UnstagedFile,
        |path| SectionType::UnstagedFile { path },
        |path, hunk_index| SectionType::UnstagedHunk { path, hunk_index },
    ))
}
