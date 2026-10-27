use crate::{
    errors::MagiResult,
    i18n,
    model::{Line, LineContent, SectionType},
};
use git2::Repository;

/// Returns the lines representing untracked files in the Git repository
pub fn get_lines(repository: &Repository) -> MagiResult<Vec<Line>> {
    // Get untracked files
    let mut status_options = git2::StatusOptions::new();
    status_options.include_untracked(true);
    status_options.include_ignored(false);

    let statuses = repository.statuses(Some(&mut status_options))?;

    // Filter for untracked files only
    let untracked_files: Vec<String> = statuses
        .iter()
        .filter_map(|entry| {
            if entry.status().is_wt_new() {
                entry.path().ok().map(|path| path.to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(build_untracked_lines(untracked_files))
}

/// Builds the untracked-files section lines from a list of untracked paths.
///
/// Shared between the standalone [`get_lines`] (which gets its paths from a
/// `statuses` call) and the combined working-tree walk in [`crate::git`], which
/// derives untracked paths from the same diff it uses for unstaged changes,
/// avoiding a second full scan of the working tree.
pub fn build_untracked_lines(untracked_files: Vec<String>) -> Vec<Line> {
    let mut lines = Vec::new();
    let untracked_count = untracked_files.len();

    // If there are no untracked files, return an empty vector
    if untracked_count == 0 {
        return lines;
    }

    // Add section header
    lines.push(Line {
        content: LineContent::SectionHeader {
            title: i18n::t().section_untracked_files.to_string(),
            count: Some(untracked_count),
        },
        section: Some(SectionType::UntrackedFiles),
    });

    // Add each untracked file
    for file_path in untracked_files {
        lines.push(Line {
            content: LineContent::UntrackedFile(file_path),
            section: Some(SectionType::UntrackedFiles),
        });
    }

    lines
}
