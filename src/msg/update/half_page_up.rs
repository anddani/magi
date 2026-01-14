use crate::{model::Model, msg::Message};

pub fn update(model: &mut Model) -> Option<Message> {
    let half_page = model.ui_model.viewport_height / 2;
    // Move up by half_page visible lines
    let mut visible_count = 0;
    let mut new_pos = model.ui_model.cursor_position;
    while new_pos > 0 && visible_count < half_page {
        new_pos -= 1;
        if !model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections) {
            visible_count += 1;
        }
    }
    // Make sure we land on a visible line (try backward first, then forward)
    while new_pos > 0 && model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections)
    {
        new_pos -= 1;
    }
    // If still on a hidden line (reached beginning), search forward
    let max_pos = model.ui_model.lines.len().saturating_sub(1);
    while new_pos < max_pos
        && model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections)
    {
        new_pos += 1;
    }
    model.ui_model.cursor_position = new_pos;
    // Scroll up if cursor moves above viewport
    if model.ui_model.cursor_position < model.ui_model.scroll_offset {
        model.ui_model.scroll_offset = model.ui_model.cursor_position;
    }
    None
}
