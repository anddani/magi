use magi::msg::{Message, update::update};

use crate::utils::create_test_model_with_lines;

mod utils;

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
