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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;
    use std::fs;

    #[test]
    fn test_get_lines_with_untracked_files() {
        let test_repo = TestRepo::new();

        // Create an untracked file
        fs::write(
            test_repo.repo.workdir().unwrap().join("new_untracked.txt"),
            "test content",
        )
        .unwrap();

        let lines = get_lines(&test_repo.repo).unwrap();

        // Should have 2 lines: header + 1 file
        assert_eq!(lines.len(), 2);

        // Check section header
        match &lines[0].content {
            crate::model::LineContent::SectionHeader { title, count } => {
                assert_eq!(title, "Untracked files");
                assert_eq!(*count, Some(1));
            }
            _ => panic!("Expected SectionHeader"),
        }

        // Check untracked file
        match &lines[1].content {
            crate::model::LineContent::UntrackedFile(path) => {
                assert_eq!(path, "new_untracked.txt");
            }
            _ => panic!("Expected UntrackedFile"),
        }
    }

    #[test]
    fn test_get_lines_no_untracked_files() {
        let test_repo = TestRepo::new();

        // The test repo commits test.txt, so there should be no untracked files
        let lines = get_lines(&test_repo.repo).unwrap();

        // Should have no lines when no untracked files (test.txt is committed)
        assert_eq!(lines.len(), 0);
    }
}
