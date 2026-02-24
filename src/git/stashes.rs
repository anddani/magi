use git2::Repository;

use crate::{
    errors::MagiResult,
    model::{Line, LineContent, SectionType},
};

use super::StashEntry;

/// Returns lines representing the stash stack.
/// Returns an empty Vec if there are no stashes.
pub fn get_lines(repository: &Repository) -> MagiResult<Vec<Line>> {
    let reflog = match repository.reflog("refs/stash") {
        Ok(r) => r,
        Err(_) => return Ok(vec![]), // No stash ref means no stashes exist
    };

    if reflog.is_empty() {
        return Ok(vec![]);
    }

    let mut lines = Vec::new();

    lines.push(Line {
        content: LineContent::SectionHeader {
            title: "Stashes".to_string(),
            count: Some(reflog.len()),
        },
        section: Some(SectionType::Stashes),
    });

    for (index, entry) in reflog.iter().enumerate() {
        let message = entry.message().unwrap_or("").to_string();
        lines.push(Line {
            content: LineContent::Stash(StashEntry { index, message }),
            section: Some(SectionType::Stashes),
        });
    }

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    #[test]
    fn test_get_lines_no_stashes() {
        let test_repo = TestRepo::new();
        let lines = get_lines(&test_repo.repo).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn test_get_lines_with_stash() {
        let test_repo = TestRepo::new();
        test_repo.create_file("dirty.txt");

        // Create a stash via git command
        let workdir = test_repo.repo.workdir().unwrap().to_path_buf();
        std::process::Command::new("git")
            .args([
                "-C",
                workdir.to_str().unwrap(),
                "stash",
                "push",
                "-u",
                "-m",
                "test stash",
            ])
            .output()
            .unwrap();

        let lines = get_lines(&test_repo.repo).unwrap();

        // Should have header + 1 stash entry
        assert_eq!(lines.len(), 2);

        // First line is the section header
        assert!(matches!(
            lines[0].content,
            LineContent::SectionHeader { ref title, count: Some(1) } if title == "Stashes"
        ));

        // Second line is the stash entry
        if let LineContent::Stash(ref entry) = lines[1].content {
            assert_eq!(entry.index, 0);
        } else {
            panic!("Expected Stash content");
        }
    }
}
