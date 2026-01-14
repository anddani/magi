use crate::{model::Model, msg::Message};

pub fn update(model: &mut Model) -> Option<Message> {
    // Scroll viewport down by one visible line (C-e in Vim)
    let max_pos = model.ui_model.lines.len().saturating_sub(1);
    let viewport_height = model.ui_model.viewport_height;
    if viewport_height == 0 {
        return None;
    }

    // Find the next visible line after current scroll_offset
    let mut new_scroll = model.ui_model.scroll_offset;
    while new_scroll < max_pos {
        new_scroll += 1;
        if !model.ui_model.lines[new_scroll].is_hidden(&model.ui_model.collapsed_sections) {
            break;
        }
    }

    // Only scroll if there's content below to show
    if new_scroll <= max_pos
        && !model.ui_model.lines[new_scroll].is_hidden(&model.ui_model.collapsed_sections)
    {
        model.ui_model.scroll_offset = new_scroll;

        // If cursor is now above viewport, move it to the top visible line
        if model.ui_model.cursor_position < model.ui_model.scroll_offset {
            model.ui_model.cursor_position = model.ui_model.scroll_offset;
        }
    }
    None
}
