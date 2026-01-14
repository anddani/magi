use crate::{
    model::Model,
    msg::{util::visible_lines_between, Message},
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Scroll viewport up by one visible line (C-y in Vim)
    if model.ui_model.scroll_offset == 0 {
        return None;
    }

    let viewport_height = model.ui_model.viewport_height;
    if viewport_height == 0 {
        return None;
    }

    // Find the previous visible line before current scroll_offset
    let mut new_scroll = model.ui_model.scroll_offset;
    while new_scroll > 0 {
        new_scroll -= 1;
        if !model.ui_model.lines[new_scroll].is_hidden(&model.ui_model.collapsed_sections) {
            break;
        }
    }

    model.ui_model.scroll_offset = new_scroll;

    // If cursor is now below viewport, move it to the bottom visible line
    let visible_before_cursor = visible_lines_between(
        &model.ui_model.lines,
        model.ui_model.scroll_offset,
        model.ui_model.cursor_position,
        &model.ui_model.collapsed_sections,
    );

    if visible_before_cursor >= viewport_height {
        // Find the last visible line in the viewport
        let mut new_cursor = model.ui_model.scroll_offset;
        let mut visible_count = 0;
        let max_pos = model.ui_model.lines.len().saturating_sub(1);
        while new_cursor < max_pos && visible_count < viewport_height - 1 {
            new_cursor += 1;
            if !model.ui_model.lines[new_cursor].is_hidden(&model.ui_model.collapsed_sections) {
                visible_count += 1;
            }
        }
        // Make sure we're on a visible line
        while new_cursor > 0
            && model.ui_model.lines[new_cursor].is_hidden(&model.ui_model.collapsed_sections)
        {
            new_cursor -= 1;
        }
        model.ui_model.cursor_position = new_cursor;
    }
    None
}
