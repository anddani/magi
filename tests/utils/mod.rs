use magi::{
    config::Theme,
    git::{GitInfo, test_repo::TestRepo},
    model::{
        FileChange, FileStatus, Line, LineContent, Model, RunningState, SectionType, UiModel,
        ViewMode,
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
#[allow(unused)]
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
