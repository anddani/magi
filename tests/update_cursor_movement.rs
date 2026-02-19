use magi::msg::Message;
use magi::msg::update::update;

mod utils;

use crate::utils::create_test_model_with_lines;

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
