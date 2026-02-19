use std::{collections::HashSet, fs};

use magi::{
    config::Theme,
    git::{GitInfo, stage::stage_files, test_repo::TestRepo},
    model::{
        DiffLine, DiffLineType, FileChange, FileStatus, Line, LineContent, Model, PopupContent,
        RunningState, SectionType, UiModel, ViewMode,
    },
    msg::{Message, update::update},
};

use crate::utils::{
    create_model_from_test_repo, create_section_lines, create_test_model,
    create_test_model_with_lines,
};

mod utils;

/// Create lines where both files are collapsed, leaving few visible lines
fn create_both_files_collapsed_lines() -> Vec<Line> {
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

    // 2-11: First file hunks (10 lines, will be hidden when collapsed)
    for i in 0..10 {
        lines.push(Line {
            content: LineContent::DiffLine(DiffLine {
                content: format!(" line {}", i),
                line_type: DiffLineType::Context,
            }),
            section: Some(SectionType::UnstagedHunk {
                path: "file1.rs".to_string(),
                hunk_index: 0,
            }),
        });
    }

    // 12: Second file header
    lines.push(Line {
        content: LineContent::UnstagedFile(FileChange {
            path: "file2.rs".to_string(),
            status: FileStatus::Modified,
        }),
        section: Some(SectionType::UnstagedFile {
            path: "file2.rs".to_string(),
        }),
    });

    // 13-22: Second file hunks (10 lines, will be hidden when collapsed)
    for i in 0..10 {
        lines.push(Line {
            content: LineContent::DiffLine(DiffLine {
                content: format!(" line {}", i),
                line_type: DiffLineType::Context,
            }),
            section: Some(SectionType::UnstagedHunk {
                path: "file2.rs".to_string(),
                hunk_index: 0,
            }),
        });
    }

    lines // Total: 23 lines (indices 0-22)
}

#[test]
fn test_unstaged_changes_should_default_collapsed_when_creating_model() {
    let test_repo = TestRepo::new();
    let file_name = "test.txt";
    test_repo.create_file(file_name);
    test_repo.stage_files(&[file_name]);
    test_repo.commit("Initial commit");
    test_repo.write_file_content(file_name, "Updated content");

    let model = create_model_from_test_repo(&test_repo);

    let is_default_collapsed = model.ui_model.collapsed_sections.iter().any(|cs| {
        cs == &SectionType::UnstagedFile {
            path: file_name.to_string(),
        }
    });

    assert!(is_default_collapsed);
}

#[test]
fn test_unstaged_changes_should_default_collapsed_when_refreshing() {
    let test_repo = TestRepo::new();
    let file_name = "test.txt";

    test_repo.create_file(file_name);
    test_repo.stage_files(&[file_name]);
    test_repo.commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let has_no_collapsed_sections = model.ui_model.collapsed_sections.is_empty();
    assert!(has_no_collapsed_sections);

    // Update file to bring it to unstaged changes
    test_repo.write_file_content(file_name, "Updated content");
    update(&mut model, Message::Refresh);
    let is_section_collapsed = model.ui_model.collapsed_sections.iter().any(|cs| {
        cs == &SectionType::UnstagedFile {
            path: file_name.to_string(),
        }
    });
    assert!(is_section_collapsed);
}

#[test]
fn test_toggle_section_on_header() {
    let mut model = create_test_model_with_lines(0);
    model.ui_model.lines = create_section_lines();
    model.ui_model.cursor_position = 0; // On section header

    // Initially not collapsed
    assert!(
        !model
            .ui_model
            .collapsed_sections
            .contains(&SectionType::UntrackedFiles)
    );

    // Toggle should collapse
    update(&mut model, Message::ToggleSection);
    assert!(
        model
            .ui_model
            .collapsed_sections
            .contains(&SectionType::UntrackedFiles)
    );

    // Toggle again should expand
    update(&mut model, Message::ToggleSection);
    assert!(
        !model
            .ui_model
            .collapsed_sections
            .contains(&SectionType::UntrackedFiles)
    );
}

#[test]
fn test_toggle_section_on_file() {
    let mut model = create_test_model_with_lines(0);
    model.ui_model.lines = create_section_lines();
    model.ui_model.cursor_position = 5; // On file (foo.rs)

    let file_section = SectionType::UnstagedFile {
        path: "foo.rs".to_string(),
    };

    // Initially not collapsed
    assert!(!model.ui_model.collapsed_sections.contains(&file_section));

    // Toggle should collapse the file's section
    update(&mut model, Message::ToggleSection);
    assert!(model.ui_model.collapsed_sections.contains(&file_section));
}

#[test]
fn test_toggle_section_on_non_header_does_nothing() {
    let mut model = create_test_model_with_lines(0);
    model.ui_model.lines = create_section_lines();
    model.ui_model.cursor_position = 1; // On untracked file (not a collapsible header)

    // Toggle on non-header should do nothing
    update(&mut model, Message::ToggleSection);
    assert!(model.ui_model.collapsed_sections.is_empty());
}

#[test]
fn test_half_page_down_with_collapsed_sections() {
    // This tests the bug where Ctrl+d with collapsed sections
    // causes cursor to land on a hidden line
    let mut model = create_test_model_with_lines(0);
    model.ui_model.lines = create_both_files_collapsed_lines();
    model.ui_model.cursor_position = 1; // On first file header

    // Collapse both files
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: "file1.rs".to_string(),
        });
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: "file2.rs".to_string(),
        });

    // Visible lines are: 0 (header), 1 (file1), 12 (file2)
    // half_page = 5, but only 2 visible lines after cursor
    // Cursor should land on line 12 (file2), the last visible line after cursor

    update(&mut model, Message::HalfPageDown);

    // Cursor must be on a visible line
    assert!(
        !model.ui_model.lines[model.ui_model.cursor_position]
            .is_hidden(&model.ui_model.collapsed_sections),
        "Cursor should be on a visible line, but landed on hidden line at position {}",
        model.ui_model.cursor_position
    );

    // Should land on file2 header (line 12)
    assert_eq!(
        model.ui_model.cursor_position, 12,
        "Cursor should land on file2 header"
    );
}

#[test]
fn test_half_page_up_with_collapsed_sections() {
    // Same test but for HalfPageUp
    let mut model = create_test_model_with_lines(0);
    model.ui_model.lines = create_both_files_collapsed_lines();
    model.ui_model.cursor_position = 12; // On second file header

    // Collapse both files
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: "file1.rs".to_string(),
        });
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: "file2.rs".to_string(),
        });

    // Visible lines are: 0 (header), 1 (file1), 12 (file2)
    // half_page = 5, but only 2 visible lines before cursor

    update(&mut model, Message::HalfPageUp);

    // Cursor must be on a visible line
    assert!(
        !model.ui_model.lines[model.ui_model.cursor_position]
            .is_hidden(&model.ui_model.collapsed_sections),
        "Cursor should be on a visible line, but landed on hidden line at position {}",
        model.ui_model.cursor_position
    );
}

#[test]
fn test_commit_without_staged_changes_shows_toast() {
    // This test verifies that trying to commit without staged changes
    // shows a toast instead of launching the editor
    let mut model = create_test_model();

    // Send commit message (no staged changes in test repo)
    update(&mut model, Message::Commit);

    // Should show a toast warning about no staged changes
    assert!(
        model.toast.is_some(),
        "Should show a toast when no staged changes"
    );
}

#[test]
fn test_dismiss_popup_clears_popup() {
    let mut model = create_test_model();
    model.popup = Some(PopupContent::Error {
        message: "Test error".to_string(),
    });

    // Popup should be present
    assert!(model.popup.is_some());

    // Send dismiss message
    update(&mut model, Message::DismissPopup);

    // Popup should be cleared
    assert!(model.popup.is_none());
}

#[test]
fn test_collapsed_state_preserved_when_staging_all() {
    // Create a test repo with a modified file
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Modify the tracked file to create unstaged changes
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "modified content").unwrap();

    // Create GitInfo from test repo
    let git_info = GitInfo::new_from_path(repo_path).unwrap();
    let lines = git_info.get_lines().unwrap();

    // Find the unstaged file section and collapse it
    let mut collapsed_sections = HashSet::new();
    for line in &lines {
        if let Some(section) = &line.section {
            if let SectionType::UnstagedFile { path } = section {
                if path == "test.txt" {
                    collapsed_sections.insert(section.clone());
                }
            }
        }
    }

    // Verify we found and collapsed the file
    assert!(
        collapsed_sections.contains(&SectionType::UnstagedFile {
            path: "test.txt".to_string()
        }),
        "Should have found and collapsed the unstaged file"
    );

    let workdir = repo_path.to_path_buf();
    let mut model = Model {
        git_info,
        workdir,
        running_state: RunningState::Running,
        ui_model: UiModel {
            lines,
            cursor_position: 0,
            scroll_offset: 0,
            viewport_height: 20,
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
    };

    // Stage all modified files
    let follow_up = update(&mut model, Message::StageAllModified);
    assert_eq!(follow_up, Some(Message::Refresh));

    // Process the refresh
    update(&mut model, Message::Refresh);

    // Verify the file is now in staged section and still collapsed
    assert!(
        model
            .ui_model
            .collapsed_sections
            .contains(&SectionType::StagedFile {
                path: "test.txt".to_string()
            }),
        "Staged file should be collapsed after StageAllModified"
    );

    // The old unstaged file section should be cleaned up
    assert!(
        !model
            .ui_model
            .collapsed_sections
            .contains(&SectionType::UnstagedFile {
                path: "test.txt".to_string()
            }),
        "Old unstaged file section should be cleaned up"
    );
}

#[test]
fn test_collapsed_state_preserved_when_unstaging_all() {
    // Create a test repo
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Modify and stage the file
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "modified content").unwrap();
    stage_files(repo_path, &["test.txt"]).unwrap();

    // Create GitInfo from test repo
    let git_info = GitInfo::new_from_path(repo_path).unwrap();
    let lines = git_info.get_lines().unwrap();

    // Find the staged file section and collapse it
    let mut collapsed_sections = HashSet::new();
    for line in &lines {
        if let Some(section) = &line.section {
            if let SectionType::StagedFile { path } = section {
                if path == "test.txt" {
                    collapsed_sections.insert(section.clone());
                }
            }
        }
    }

    // Verify we found and collapsed the file
    assert!(
        collapsed_sections.contains(&SectionType::StagedFile {
            path: "test.txt".to_string()
        }),
        "Should have found and collapsed the staged file"
    );

    let workdir = repo_path.to_path_buf();
    let mut model = Model {
        git_info,
        workdir,
        running_state: RunningState::Running,
        ui_model: UiModel {
            lines,
            cursor_position: 0,
            scroll_offset: 0,
            viewport_height: 20,
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
    };

    // Unstage all files
    let follow_up = update(&mut model, Message::UnstageAll);
    assert_eq!(follow_up, Some(Message::Refresh));

    // Process the refresh
    update(&mut model, Message::Refresh);

    // Verify the file is now in unstaged section and still collapsed
    assert!(
        model
            .ui_model
            .collapsed_sections
            .contains(&SectionType::UnstagedFile {
                path: "test.txt".to_string()
            }),
        "Unstaged file should be collapsed after UnstageAll"
    );

    // The old staged file section should be cleaned up
    assert!(
        !model
            .ui_model
            .collapsed_sections
            .contains(&SectionType::StagedFile {
                path: "test.txt".to_string()
            }),
        "Old staged file section should be cleaned up"
    );
}

#[test]
fn test_expanded_state_preserved_when_staging() {
    // Create a test repo with a modified file
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Modify the tracked file
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "modified content").unwrap();

    // Create GitInfo from test repo
    let git_info = GitInfo::new_from_path(repo_path).unwrap();
    let lines = git_info.get_lines().unwrap();

    // Don't collapse the file - leave it expanded
    let collapsed_sections = HashSet::new();

    let workdir = repo_path.to_path_buf();
    let mut model = Model {
        git_info,
        workdir,
        running_state: RunningState::Running,
        ui_model: UiModel {
            lines,
            cursor_position: 0,
            scroll_offset: 0,
            viewport_height: 20,
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
    };

    // Stage all modified files
    let follow_up = update(&mut model, Message::StageAllModified);
    assert_eq!(follow_up, Some(Message::Refresh));

    // Process the refresh
    update(&mut model, Message::Refresh);

    // Verify the file is NOT collapsed (stayed expanded)
    assert!(
        !model
            .ui_model
            .collapsed_sections
            .contains(&SectionType::StagedFile {
                path: "test.txt".to_string()
            }),
        "File should remain expanded when moving"
    );
}
