use crate::{
    errors::MagiResult,
    model::{Line, LineContent, SectionType},
};
use git2::Repository;

/// Returns the lines representing untracked files in the Git repository
pub fn get_lines(repository: &Repository) -> MagiResult<Vec<Line>> {
    let mut lines = Vec::new();

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
                entry.path().map(|path| path.to_string())
            } else {
                None
            }
        })
        .collect();

    let untracked_count = untracked_files.len();

    // If there are no untracked files, return an empty vector
    if untracked_count == 0 {
        return Ok(vec![]);
    }

    // Add section header
    let header_line = Line {
        content: LineContent::SectionHeader {
            title: "Untracked files".to_string(),
            count: Some(untracked_count),
        },
        section: Some(SectionType::UntrackedFiles),
    };
    lines.push(header_line);

    // Add each untracked file
    for file_path in untracked_files {
        let file_line = Line {
            content: LineContent::UntrackedFile(file_path),
            section: Some(SectionType::UntrackedFiles),
        };
        lines.push(file_line);
    }

    Ok(lines)
}
