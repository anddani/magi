use std::collections::HashSet;

use magi::config::Theme;
use magi::git::stage::stage_files;
use magi::git::test_repo::TestRepo;
use magi::git::GitInfo;
use magi::model::arguments::{Argument, Arguments, PushArgument};
use magi::model::popup::{PopupContent, PopupContentCommand};
use magi::model::{
    DiffHunk, DiffLine, DiffLineType, FileChange, FileStatus, Line, LineContent, Model,
    RunningState, SectionType, UiModel,
};
use magi::msg::update::update;
use magi::msg::util::visible_lines_between;
use magi::msg::Message;
use std::fs;

/// Creates a test model with default values. Lines are populated from the git repo.
fn create_test_model() -> Model {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();
    let git_info = GitInfo::new_from_path(repo_path).unwrap();
    // Get lines while TestRepo is still alive (temp directory exists)
    let lines = git_info.get_lines().unwrap();
    Model {
        git_info,
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
    }
}

#[test]
fn test_refresh_message() {
    // This test needs the TestRepo to stay alive since Refresh reads from the git repo
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();
    let git_info = GitInfo::new_from_path(repo_path).unwrap();
    let lines = git_info.get_lines().unwrap();

    let mut model = Model {
        git_info,
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
    };

    // Clear the lines to simulate outdated state
    model.ui_model.lines.clear();
    assert!(model.ui_model.lines.is_empty());

    // Send refresh message
    update(&mut model, Message::Refresh);

    // Verify that lines were refreshed
    assert!(!model.ui_model.lines.is_empty());
    assert_eq!(model.running_state, RunningState::Running);
}

#[test]
fn test_quit_message() {
    let mut model = create_test_model();

    // Send quit message
    update(&mut model, Message::Quit);

    // Verify that running state changed to Done
    assert_eq!(model.running_state, RunningState::Done);
}

fn create_test_lines(count: usize) -> Vec<Line> {
    (0..count)
        .map(|_| Line {
            content: LineContent::EmptyLine,
            section: None,
        })
        .collect()
}

fn create_test_model_with_lines(count: usize) -> Model {
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

#[test]
fn test_move_down() {
    let mut model = create_test_model_with_lines(5);

    update(&mut model, Message::MoveDown);
    assert_eq!(model.ui_model.cursor_position, 1);

    update(&mut model, Message::MoveDown);
    assert_eq!(model.ui_model.cursor_position, 2);
}

#[test]
fn test_move_up() {
    let mut model = create_test_model_with_lines(5);
    model.ui_model.cursor_position = 2;

    update(&mut model, Message::MoveUp);
    assert_eq!(model.ui_model.cursor_position, 1);

    update(&mut model, Message::MoveUp);
    assert_eq!(model.ui_model.cursor_position, 0);
}

#[test]
fn test_move_up_at_top() {
    let mut model = create_test_model_with_lines(5);
    model.ui_model.cursor_position = 0;

    update(&mut model, Message::MoveUp);
    assert_eq!(model.ui_model.cursor_position, 0);
}

#[test]
fn test_move_down_at_bottom() {
    let mut model = create_test_model_with_lines(5);
    model.ui_model.cursor_position = 4;

    update(&mut model, Message::MoveDown);
    assert_eq!(model.ui_model.cursor_position, 4);
}

#[test]
fn test_scroll_down_when_cursor_leaves_viewport() {
    let mut model = create_test_model_with_lines(10);
    model.ui_model.cursor_position = 2;
    model.ui_model.viewport_height = 3;

    // Move to position 3, which is outside viewport (0-2)
    update(&mut model, Message::MoveDown);
    assert_eq!(model.ui_model.cursor_position, 3);
    assert_eq!(model.ui_model.scroll_offset, 1);
}

#[test]
fn test_scroll_up_when_cursor_leaves_viewport() {
    let mut model = create_test_model_with_lines(10);
    model.ui_model.cursor_position = 5;
    model.ui_model.scroll_offset = 5;
    model.ui_model.viewport_height = 3;

    // Move to position 4, which is above scroll_offset
    update(&mut model, Message::MoveUp);
    assert_eq!(model.ui_model.cursor_position, 4);
    assert_eq!(model.ui_model.scroll_offset, 4);
}

#[test]
fn test_half_page_down() {
    let mut model = create_test_model_with_lines(20);
    model.ui_model.viewport_height = 10;

    update(&mut model, Message::HalfPageDown);
    assert_eq!(model.ui_model.cursor_position, 5); // half of 10
}

#[test]
fn test_half_page_up() {
    let mut model = create_test_model_with_lines(20);
    model.ui_model.cursor_position = 10;
    model.ui_model.scroll_offset = 5;
    model.ui_model.viewport_height = 10;

    update(&mut model, Message::HalfPageUp);
    assert_eq!(model.ui_model.cursor_position, 5); // 10 - 5
}

#[test]
fn test_half_page_up_at_top() {
    let mut model = create_test_model_with_lines(20);
    model.ui_model.viewport_height = 10;

    update(&mut model, Message::HalfPageUp);
    assert_eq!(model.ui_model.cursor_position, 0); // stays at 0
    assert_eq!(model.ui_model.scroll_offset, 0);
}

#[test]
fn test_half_page_down_at_bottom() {
    let mut model = create_test_model_with_lines(20);
    model.ui_model.cursor_position = 19;
    model.ui_model.scroll_offset = 10;
    model.ui_model.viewport_height = 10;

    update(&mut model, Message::HalfPageDown);
    assert_eq!(model.ui_model.cursor_position, 19); // stays at max
}

#[test]
fn test_half_page_down_clamps_to_max() {
    let mut model = create_test_model_with_lines(20);
    model.ui_model.cursor_position = 17;
    model.ui_model.scroll_offset = 10;
    model.ui_model.viewport_height = 10;

    // 17 + 5 = 22, but max is 19
    update(&mut model, Message::HalfPageDown);
    assert_eq!(model.ui_model.cursor_position, 19);
}

#[test]
fn test_half_page_up_clamps_to_zero() {
    let mut model = create_test_model_with_lines(20);
    model.ui_model.cursor_position = 2;
    model.ui_model.viewport_height = 10;

    // 2 - 5 would be negative, should clamp to 0
    update(&mut model, Message::HalfPageUp);
    assert_eq!(model.ui_model.cursor_position, 0);
}

#[test]
fn test_half_page_down_scrolls_viewport() {
    let mut model = create_test_model_with_lines(30);
    model.ui_model.cursor_position = 8;
    model.ui_model.viewport_height = 10;

    // Cursor at 8, move down 5 -> 13, which is outside viewport (0-9)
    update(&mut model, Message::HalfPageDown);
    assert_eq!(model.ui_model.cursor_position, 13);
    assert_eq!(model.ui_model.scroll_offset, 4); // 13 - 10 + 1
}

#[test]
fn test_half_page_up_scrolls_viewport() {
    let mut model = create_test_model_with_lines(30);
    model.ui_model.cursor_position = 12;
    model.ui_model.scroll_offset = 10;
    model.ui_model.viewport_height = 10;

    // Cursor at 12, scroll at 10, move up 5 -> 7, which is above scroll_offset
    update(&mut model, Message::HalfPageUp);
    assert_eq!(model.ui_model.cursor_position, 7);
    assert_eq!(model.ui_model.scroll_offset, 7);
}

#[test]
fn test_half_page_with_small_viewport() {
    let mut model = create_test_model_with_lines(10);
    model.ui_model.cursor_position = 5;
    model.ui_model.scroll_offset = 3;
    model.ui_model.viewport_height = 2;

    // Half of 2 is 1
    update(&mut model, Message::HalfPageDown);
    assert_eq!(model.ui_model.cursor_position, 6);

    update(&mut model, Message::HalfPageUp);
    assert_eq!(model.ui_model.cursor_position, 5);
}

#[test]
fn test_half_page_with_zero_viewport() {
    let mut model = create_test_model_with_lines(10);
    model.ui_model.cursor_position = 5;
    model.ui_model.viewport_height = 0;

    // Half of 0 is 0, cursor shouldn't move
    update(&mut model, Message::HalfPageDown);
    assert_eq!(model.ui_model.cursor_position, 5);

    update(&mut model, Message::HalfPageUp);
    assert_eq!(model.ui_model.cursor_position, 5);
}

#[test]
fn test_half_page_with_empty_lines() {
    let mut model = create_test_model_with_lines(0);
    model.ui_model.viewport_height = 10;

    // With no lines, cursor should stay at 0
    update(&mut model, Message::HalfPageDown);
    assert_eq!(model.ui_model.cursor_position, 0);

    update(&mut model, Message::HalfPageUp);
    assert_eq!(model.ui_model.cursor_position, 0);
}

fn create_section_lines() -> Vec<Line> {
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

#[test]
fn test_toggle_section_on_header() {
    let mut model = create_test_model_with_lines(0);
    model.ui_model.lines = create_section_lines();
    model.ui_model.cursor_position = 0; // On section header

    // Initially not collapsed
    assert!(!model
        .ui_model
        .collapsed_sections
        .contains(&SectionType::UntrackedFiles));

    // Toggle should collapse
    update(&mut model, Message::ToggleSection);
    assert!(model
        .ui_model
        .collapsed_sections
        .contains(&SectionType::UntrackedFiles));

    // Toggle again should expand
    update(&mut model, Message::ToggleSection);
    assert!(!model
        .ui_model
        .collapsed_sections
        .contains(&SectionType::UntrackedFiles));
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
fn test_move_down_skips_hidden_lines() {
    let mut model = create_test_model_with_lines(0);
    model.ui_model.lines = create_section_lines();
    model.ui_model.cursor_position = 0; // On "Untracked files" header

    // Collapse the Untracked files section
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UntrackedFiles);

    // Move down should skip hidden lines (1, 2) and land on empty line (3)
    update(&mut model, Message::MoveDown);
    assert_eq!(model.ui_model.cursor_position, 3);
}

#[test]
fn test_move_up_skips_hidden_lines() {
    let mut model = create_test_model_with_lines(0);
    model.ui_model.lines = create_section_lines();
    model.ui_model.cursor_position = 3; // On empty line

    // Collapse the Untracked files section
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UntrackedFiles);

    // Move up should skip hidden lines (2, 1) and land on header (0)
    update(&mut model, Message::MoveUp);
    assert_eq!(model.ui_model.cursor_position, 0);
}

/// Create lines simulating two files with many diff lines each
fn create_two_file_lines() -> Vec<Line> {
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

#[test]
fn test_scroll_with_collapsed_file_does_not_over_scroll() {
    // This tests the bug where navigating from a collapsed file to the next file
    // caused the screen to scroll so the target file was at the top instead of bottom
    let mut model = create_test_model_with_lines(0);
    model.ui_model.lines = create_two_file_lines();
    model.ui_model.cursor_position = 1; // On first file header (file1.rs)

    // Collapse the first file - this hides lines 2-22 (hunk + 20 diff lines)
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: "file1.rs".to_string(),
        });

    // Move down should go to the second file header (line 23)
    update(&mut model, Message::MoveDown);
    assert_eq!(model.ui_model.cursor_position, 23);

    // With viewport_height=10, and only 3 visible lines before cursor
    // (line 0: section header, line 1: file1 header, line 23: file2 header)
    // the scroll_offset should NOT change since cursor is still in viewport
    // Visible lines from scroll_offset=0: 0, 1, 23 = only 3 lines before position 23
    // 3 < 10, so no scroll needed
    assert_eq!(
        model.ui_model.scroll_offset, 0,
        "scroll_offset should remain 0 since cursor is within viewport"
    );
}

#[test]
fn test_visible_lines_between() {
    let lines = create_section_lines();
    let mut collapsed = HashSet::new();

    // No collapsed sections: all lines visible
    assert_eq!(visible_lines_between(&lines, 0, 3, &collapsed), 3);
    assert_eq!(visible_lines_between(&lines, 0, 7, &collapsed), 7);

    // Collapse UntrackedFiles section - hides lines 1, 2
    collapsed.insert(SectionType::UntrackedFiles);
    // Lines 0-3: line 0 visible, lines 1-2 hidden, so only 1 visible
    assert_eq!(visible_lines_between(&lines, 0, 3, &collapsed), 1);
    // Lines 0-7: lines 0, 3, 4, 5, 6 visible = 5
    assert_eq!(visible_lines_between(&lines, 0, 7, &collapsed), 5);
}

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
fn test_scroll_line_down() {
    let mut model = create_test_model_with_lines(20);

    // Scroll down once
    update(&mut model, Message::ScrollLineDown);
    assert_eq!(model.ui_model.scroll_offset, 1);
    assert_eq!(model.ui_model.cursor_position, 1); // Cursor moves with viewport

    // Scroll down again
    update(&mut model, Message::ScrollLineDown);
    assert_eq!(model.ui_model.scroll_offset, 2);
    assert_eq!(model.ui_model.cursor_position, 2);
}

#[test]
fn test_scroll_line_up() {
    let mut model = create_test_model_with_lines(20);
    model.ui_model.cursor_position = 15;
    model.ui_model.scroll_offset = 10;

    // Scroll up once
    update(&mut model, Message::ScrollLineUp);
    assert_eq!(model.ui_model.scroll_offset, 9);
    assert_eq!(model.ui_model.cursor_position, 15); // Cursor stays in place

    // Scroll up more times until cursor would leave viewport
    for _ in 0..6 {
        update(&mut model, Message::ScrollLineUp);
    }
    // scroll_offset should be 3, cursor should move to bottom of viewport
    assert_eq!(model.ui_model.scroll_offset, 3);
    // Cursor should be at bottom of viewport (scroll_offset + viewport_height - 1 = 3 + 10 - 1 = 12)
    assert_eq!(model.ui_model.cursor_position, 12);
}

#[test]
fn test_scroll_line_down_cursor_follows_top() {
    let mut model = create_test_model_with_lines(20);
    model.ui_model.viewport_height = 5;

    // Cursor at top of viewport, scroll down - cursor should follow
    update(&mut model, Message::ScrollLineDown);
    assert_eq!(model.ui_model.scroll_offset, 1);
    assert_eq!(model.ui_model.cursor_position, 1);
}

#[test]
fn test_scroll_line_up_cursor_follows_bottom() {
    let mut model = create_test_model_with_lines(20);
    model.ui_model.cursor_position = 14; // At bottom of viewport (10 + 5 - 1)
    model.ui_model.scroll_offset = 10;
    model.ui_model.viewport_height = 5;

    // Cursor at bottom of viewport, scroll up - cursor should stay in viewport
    update(&mut model, Message::ScrollLineUp);
    assert_eq!(model.ui_model.scroll_offset, 9);
    assert_eq!(model.ui_model.cursor_position, 13); // Follows bottom of viewport
}

#[test]
fn test_scroll_line_down_at_end() {
    let mut model = create_test_model_with_lines(10);
    model.ui_model.cursor_position = 9;
    model.ui_model.scroll_offset = 5; // Already scrolled down

    // Try to scroll past end
    update(&mut model, Message::ScrollLineDown);
    update(&mut model, Message::ScrollLineDown);
    update(&mut model, Message::ScrollLineDown);
    update(&mut model, Message::ScrollLineDown);
    // Should stop at max_pos (9) since viewport can't go beyond content
    assert!(model.ui_model.scroll_offset <= 9);
}

#[test]
fn test_scroll_line_up_at_start() {
    let mut model = create_test_model_with_lines(20);

    // Try to scroll up at top - should have no effect
    update(&mut model, Message::ScrollLineUp);
    assert_eq!(model.ui_model.scroll_offset, 0);
    assert_eq!(model.ui_model.cursor_position, 0);
}

#[test]
fn test_scroll_line_down_with_collapsed_sections() {
    let mut model = create_test_model_with_lines(0);
    model.ui_model.lines = create_section_lines();

    // Collapse untracked files (hides lines 1, 2)
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UntrackedFiles);

    // Scroll down should skip hidden lines
    update(&mut model, Message::ScrollLineDown);
    // Should land on line 3 (empty line) which is the next visible line
    assert_eq!(model.ui_model.scroll_offset, 3);
    assert_eq!(model.ui_model.cursor_position, 3);
}

#[test]
fn test_scroll_with_zero_viewport() {
    let mut model = create_test_model_with_lines(10);
    model.ui_model.cursor_position = 5;
    model.ui_model.scroll_offset = 3;
    model.ui_model.viewport_height = 0;

    // With zero viewport, scrolling should do nothing
    let original_scroll = model.ui_model.scroll_offset;
    let original_cursor = model.ui_model.cursor_position;

    update(&mut model, Message::ScrollLineDown);
    assert_eq!(model.ui_model.scroll_offset, original_scroll);
    assert_eq!(model.ui_model.cursor_position, original_cursor);

    update(&mut model, Message::ScrollLineUp);
    assert_eq!(model.ui_model.scroll_offset, original_scroll);
    assert_eq!(model.ui_model.cursor_position, original_cursor);
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

    let mut model = Model {
        git_info,
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

    let mut model = Model {
        git_info,
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

    let mut model = Model {
        git_info,
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

// ============================================================================
// Visual Mode Tests
// ============================================================================

#[test]
fn test_enter_visual_mode() {
    let mut model = create_test_model_with_lines(10);
    model.ui_model.cursor_position = 5;

    // Visual mode should not be active initially
    assert!(!model.ui_model.is_visual_mode());
    assert_eq!(model.ui_model.visual_mode_anchor, None);

    // Enter visual mode
    update(&mut model, Message::EnterVisualMode);

    // Visual mode should now be active with anchor at cursor position
    assert!(model.ui_model.is_visual_mode());
    assert_eq!(model.ui_model.visual_mode_anchor, Some(5));
}

#[test]
fn test_exit_visual_mode() {
    let mut model = create_test_model_with_lines(10);
    model.ui_model.cursor_position = 5;

    // Enter visual mode first
    update(&mut model, Message::EnterVisualMode);
    assert!(model.ui_model.is_visual_mode());

    // Exit visual mode
    update(&mut model, Message::ExitVisualMode);

    // Visual mode should no longer be active
    assert!(!model.ui_model.is_visual_mode());
    assert_eq!(model.ui_model.visual_mode_anchor, None);
}

#[test]
fn test_visual_selection_range_cursor_after_anchor() {
    let mut model = create_test_model_with_lines(10);
    model.ui_model.cursor_position = 3;

    // Enter visual mode
    update(&mut model, Message::EnterVisualMode);

    // Move cursor down
    model.ui_model.cursor_position = 7;

    // Selection range should be (3, 7) - anchor to cursor
    let range = model.ui_model.visual_selection_range();
    assert_eq!(range, Some((3, 7)));
}

#[test]
fn test_visual_selection_range_cursor_before_anchor() {
    let mut model = create_test_model_with_lines(10);
    model.ui_model.cursor_position = 7;

    // Enter visual mode
    update(&mut model, Message::EnterVisualMode);

    // Move cursor up
    model.ui_model.cursor_position = 3;

    // Selection range should be (3, 7) - always ordered with start <= end
    let range = model.ui_model.visual_selection_range();
    assert_eq!(range, Some((3, 7)));
}

#[test]
fn test_visual_selection_range_same_position() {
    let mut model = create_test_model_with_lines(10);
    model.ui_model.cursor_position = 5;

    // Enter visual mode
    update(&mut model, Message::EnterVisualMode);

    // Selection range should be (5, 5) when cursor hasn't moved
    let range = model.ui_model.visual_selection_range();
    assert_eq!(range, Some((5, 5)));
}

#[test]
fn test_visual_selection_range_not_in_visual_mode() {
    let model = create_test_model_with_lines(10);

    // Not in visual mode, should return None
    assert!(!model.ui_model.is_visual_mode());
    assert_eq!(model.ui_model.visual_selection_range(), None);
}

#[test]
fn test_move_down_in_visual_mode_expands_selection() {
    let mut model = create_test_model_with_lines(10);
    model.ui_model.cursor_position = 3;

    // Enter visual mode
    update(&mut model, Message::EnterVisualMode);
    assert_eq!(model.ui_model.visual_selection_range(), Some((3, 3)));

    // Move down
    update(&mut model, Message::MoveDown);
    assert_eq!(model.ui_model.cursor_position, 4);
    assert_eq!(model.ui_model.visual_selection_range(), Some((3, 4)));

    // Move down again
    update(&mut model, Message::MoveDown);
    assert_eq!(model.ui_model.cursor_position, 5);
    assert_eq!(model.ui_model.visual_selection_range(), Some((3, 5)));
}

#[test]
fn test_move_up_in_visual_mode_expands_selection() {
    let mut model = create_test_model_with_lines(10);
    model.ui_model.cursor_position = 7;

    // Enter visual mode
    update(&mut model, Message::EnterVisualMode);
    assert_eq!(model.ui_model.visual_selection_range(), Some((7, 7)));

    // Move up
    update(&mut model, Message::MoveUp);
    assert_eq!(model.ui_model.cursor_position, 6);
    assert_eq!(model.ui_model.visual_selection_range(), Some((6, 7)));

    // Move up again
    update(&mut model, Message::MoveUp);
    assert_eq!(model.ui_model.cursor_position, 5);
    assert_eq!(model.ui_model.visual_selection_range(), Some((5, 7)));
}

#[test]
fn test_visual_mode_survives_cursor_movement() {
    let mut model = create_test_model_with_lines(20);
    model.ui_model.cursor_position = 10;
    model.ui_model.viewport_height = 10;

    // Enter visual mode
    update(&mut model, Message::EnterVisualMode);
    let anchor = model.ui_model.visual_mode_anchor;

    // Move cursor around
    update(&mut model, Message::MoveDown);
    update(&mut model, Message::MoveDown);
    update(&mut model, Message::HalfPageDown);
    update(&mut model, Message::MoveUp);

    // Visual mode should still be active with same anchor
    assert!(model.ui_model.is_visual_mode());
    assert_eq!(model.ui_model.visual_mode_anchor, anchor);
}

// ============================================================================
// Help Popup Tests
// ============================================================================

#[test]
fn test_show_help_sets_popup() {
    let mut model = create_test_model_with_lines(10);

    // Popup should be None initially
    assert!(model.popup.is_none());

    // Show help
    update(&mut model, Message::ShowHelp);

    // Popup should now be Help
    assert_eq!(model.popup, Some(PopupContent::Help));
}

#[test]
fn test_dismiss_popup_clears_help() {
    let mut model = create_test_model_with_lines(10);

    // Show help first
    update(&mut model, Message::ShowHelp);
    assert_eq!(model.popup, Some(PopupContent::Help));

    // Dismiss the popup
    update(&mut model, Message::DismissPopup);

    // Popup should be cleared
    assert!(model.popup.is_none());
}

#[test]
fn test_show_help_returns_none() {
    let mut model = create_test_model_with_lines(10);

    // ShowHelp should not trigger a follow-up message
    let follow_up = update(&mut model, Message::ShowHelp);
    assert_eq!(follow_up, None);
}

// ============================================================================
// Push Popup Argument Mode Tests
// ============================================================================

use magi::model::popup::PushPopupState;

fn create_push_popup_model() -> Model {
    let mut model = create_test_model();
    model.ui_model.viewport_height = 20;

    // Set up push popup state
    model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
        PushPopupState { upstream: None },
    )));

    model
}

#[test]
fn test_push_enter_arg_mode() {
    let mut model = create_push_popup_model();

    // Verify arg_mode starts false
    assert!(!model.arg_mode);

    // Enter arg mode
    update(&mut model, Message::EnterArgMode);

    // Verify arg_mode is now true
    assert!(model.arg_mode);
}

#[test]
fn test_push_exit_arg_mode() {
    let mut model = create_push_popup_model();

    // Set arg_mode to true first
    model.arg_mode = true;

    // Exit arg mode
    update(&mut model, Message::ExitArgMode);

    // Verify arg_mode is now false
    assert!(!model.arg_mode);
}

#[test]
fn test_push_toggle_force_with_lease_enables() {
    let mut model = create_push_popup_model();

    // Set arg_mode to true first (as would happen in real usage)
    model.arg_mode = true;
    assert!(model.arguments.is_none());

    // Toggle force_with_lease
    update(
        &mut model,
        Message::ToggleArgument(Argument::Push(PushArgument::ForceWithLease)),
    );

    // Verify force_with_lease is now enabled and arg_mode is false
    match &model.arguments {
        Some(Arguments::PushArguments(args)) => {
            assert!(args.contains(&PushArgument::ForceWithLease));
        }
        _ => panic!("Expected PushArguments"),
    }
    assert!(!model.arg_mode); // Should exit arg mode after toggle
}

#[test]
fn test_push_toggle_force_with_lease_disables() {
    let mut model = create_push_popup_model();

    // Set force_with_lease to true and arg_mode to true
    model.arg_mode = true;
    let mut args = HashSet::new();
    args.insert(PushArgument::ForceWithLease);
    model.arguments = Some(Arguments::PushArguments(args));

    // Toggle force_with_lease
    update(
        &mut model,
        Message::ToggleArgument(Argument::Push(PushArgument::ForceWithLease)),
    );

    // Verify force_with_lease is now disabled and arg_mode is false
    match &model.arguments {
        Some(Arguments::PushArguments(args)) => {
            assert!(!args.contains(&PushArgument::ForceWithLease));
        }
        _ => panic!("Expected PushArguments"),
    }
    assert!(!model.arg_mode); // Should exit arg mode after toggle
}
