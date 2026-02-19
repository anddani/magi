use magi::{
    model::SectionType,
    msg::{Message, update::update},
};

use crate::utils::{create_section_lines, create_test_model_with_lines, create_two_file_lines};

mod utils;

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
