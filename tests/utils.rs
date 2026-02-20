#![allow(unused)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use magi::{
    config::Theme,
    git::{GitInfo, stage::stage_files, test_repo::TestRepo},
    model::{
        DiffHunk, DiffLine, DiffLineType, FileChange, FileStatus, Line, LineContent, Model,
        RunningState, SectionType, UiModel, ViewMode,
    },
};

/// Creates a test model with default values. Lines are populated from the git repo.
pub fn create_test_model() -> Model {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();
    let git_info = GitInfo::new_from_path(repo_path).unwrap();
    let workdir = repo_path.to_path_buf();
    // Get lines while TestRepo is still alive (temp directory exists)
    let lines = git_info.get_lines().unwrap();
    Model {
        git_info,
        workdir,
        running_state: RunningState::Running,
        ui_model: UiModel {
            lines,
            ..Default::default()
        },
        theme: Theme::default(),
        popup: None,
        toast: None,
        select_result: None,
        select_context: None,
        pty_state: None,
        arg_mode: false,
        pending_g: false,
        arguments: None,
        open_pr_branch: None,
        view_mode: ViewMode::Status,
        cursor_reposition_context: None,
    }
}

pub fn create_test_lines(count: usize) -> Vec<Line> {
    (0..count)
        .map(|_| Line {
            content: LineContent::EmptyLine,
            section: None,
        })
        .collect()
}

pub fn create_test_model_with_lines(count: usize) -> Model {
    let mut model = create_test_model();
    model.ui_model = UiModel {
        lines: create_test_lines(count),
        cursor_position: 0,
        scroll_offset: 0,
        viewport_height: 10,
        ..Default::default()
    };
    model
}

///
/// vUntracked files (2)
///  a.txt
///  b.txt
///
/// vUnstaged changes (1)
/// vmodified foo.rs
///
pub fn create_section_lines() -> Vec<Line> {
    vec![
        // 0: Section header
        Line {
            content: LineContent::SectionHeader {
                title: "Untracked files".to_string(),
                count: Some(2),
            },
            section: Some(SectionType::UntrackedFiles),
        },
        // 1: Untracked file
        Line {
            content: LineContent::UntrackedFile("a.txt".to_string()),
            section: Some(SectionType::UntrackedFiles),
        },
        // 2: Untracked file
        Line {
            content: LineContent::UntrackedFile("b.txt".to_string()),
            section: Some(SectionType::UntrackedFiles),
        },
        // 3: Empty line
        Line {
            content: LineContent::EmptyLine,
            section: None,
        },
        // 4: Section header
        Line {
            content: LineContent::SectionHeader {
                title: "Unstaged changes".to_string(),
                count: Some(1),
            },
            section: Some(SectionType::UnstagedChanges),
        },
        // 5: File
        Line {
            content: LineContent::UnstagedFile(FileChange {
                path: "foo.rs".to_string(),
                status: FileStatus::Modified,
            }),
            section: Some(SectionType::UnstagedFile {
                path: "foo.rs".to_string(),
            }),
        },
        // 6: Hunk (would be hidden when file is collapsed)
        Line {
            content: LineContent::EmptyLine, // Simplified for testing
            section: Some(SectionType::UnstagedHunk {
                path: "foo.rs".to_string(),
                hunk_index: 0,
            }),
        },
    ]
}

/// Helper to create a model from a TestRepo with pre-populated lines from git.
pub fn create_model_from_test_repo(test_repo: &TestRepo) -> Model {
    let repo_path = test_repo.repo.workdir().unwrap();
    let git_info = GitInfo::new_from_path(repo_path).unwrap();
    let workdir = repo_path.to_path_buf();
    let lines = git_info.get_lines().unwrap();

    // Initialize collapsed sections with default collapsed items (UnstagedFile, StagedFile)
    let collapsed_sections = lines
        .iter()
        .filter_map(|line| line.section.clone())
        .filter(|section| section.default_collapsed())
        .collect();

    Model {
        git_info,
        workdir,
        running_state: RunningState::Running,
        ui_model: UiModel {
            lines,
            cursor_position: 0,
            scroll_offset: 0,
            viewport_height: 40,
            collapsed_sections,
            ..Default::default()
        },
        theme: Theme::default(),
        popup: None,
        toast: None,
        select_result: None,
        select_context: None,
        pty_state: None,
        arg_mode: false,
        pending_g: false,
        arguments: None,
        open_pr_branch: None,
        view_mode: ViewMode::Status,
        cursor_reposition_context: None,
    }
}

/// Create lines simulating two files with many diff lines each
pub fn create_two_file_lines() -> Vec<Line> {
    let mut lines = Vec::new();

    // 0: Section header
    lines.push(Line {
        content: LineContent::SectionHeader {
            title: "Unstaged changes".to_string(),
            count: Some(2),
        },
        section: Some(SectionType::UnstagedChanges),
    });

    // 1: First file header
    lines.push(Line {
        content: LineContent::UnstagedFile(FileChange {
            path: "file1.rs".to_string(),
            status: FileStatus::Modified,
        }),
        section: Some(SectionType::UnstagedFile {
            path: "file1.rs".to_string(),
        }),
    });

    // 2: First file hunk header
    lines.push(Line {
        content: LineContent::DiffHunk(DiffHunk {
            header: "@@ -1,20 +1,25 @@".to_string(),
            hunk_index: 0,
        }),
        section: Some(SectionType::UnstagedHunk {
            path: "file1.rs".to_string(),
            hunk_index: 0,
        }),
    });

    // 3-22: First file diff lines (20 lines)
    for i in 0..20 {
        lines.push(Line {
            content: LineContent::DiffLine(DiffLine {
                content: format!(" context line {}", i),
                line_type: DiffLineType::Context,
            }),
            section: Some(SectionType::UnstagedHunk {
                path: "file1.rs".to_string(),
                hunk_index: 0,
            }),
        });
    }

    // 23: Second file header
    lines.push(Line {
        content: LineContent::UnstagedFile(FileChange {
            path: "file2.rs".to_string(),
            status: FileStatus::Modified,
        }),
        section: Some(SectionType::UnstagedFile {
            path: "file2.rs".to_string(),
        }),
    });

    // 24: Second file hunk header
    lines.push(Line {
        content: LineContent::DiffHunk(DiffHunk {
            header: "@@ -1,10 +1,15 @@".to_string(),
            hunk_index: 0,
        }),
        section: Some(SectionType::UnstagedHunk {
            path: "file2.rs".to_string(),
            hunk_index: 0,
        }),
    });

    // 25-34: Second file diff lines (10 lines)
    for i in 0..10 {
        lines.push(Line {
            content: LineContent::DiffLine(DiffLine {
                content: format!(" context line {}", i),
                line_type: DiffLineType::Context,
            }),
            section: Some(SectionType::UnstagedHunk {
                path: "file2.rs".to_string(),
                hunk_index: 0,
            }),
        });
    }

    lines
}
