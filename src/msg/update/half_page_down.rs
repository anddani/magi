use crate::{
    model::Model,
    msg::{Message, util::visible_lines_between},
};

pub fn update(model: &mut Model) -> Option<Message> {
    let half_page = model.ui_model.viewport_height / 2;
    let max_pos = model.ui_model.lines.len().saturating_sub(1);
    // Move down by half_page visible lines
    let mut visible_count = 0;
    let mut new_pos = model.ui_model.cursor_position;
    while new_pos < max_pos && visible_count < half_page {
        new_pos += 1;
        if !model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections) {
            visible_count += 1;
        }
    }
    // Make sure we land on a visible line (try forward first, then backward)
    while new_pos < max_pos
        && model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections)
    {
        new_pos += 1;
    }
    // If still on a hidden line (reached end), search backward
    while new_pos > 0 && model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections)
    {
        new_pos -= 1;
    }
    model.ui_model.cursor_position = new_pos;
    // Scroll down if cursor moves below viewport
    let viewport_height = model.ui_model.viewport_height;
    if viewport_height > 0 {
        let visible_before_cursor = visible_lines_between(
            &model.ui_model.lines,
            model.ui_model.scroll_offset,
            model.ui_model.cursor_position,
            &model.ui_model.collapsed_sections,
        );
        if visible_before_cursor >= viewport_height {
            let mut new_scroll = model.ui_model.cursor_position;
            let mut scroll_visible_count = 0;
            while new_scroll > 0 && scroll_visible_count < viewport_height - 1 {
                new_scroll -= 1;
                if !model.ui_model.lines[new_scroll].is_hidden(&model.ui_model.collapsed_sections) {
                    scroll_visible_count += 1;
                }
            }
            model.ui_model.scroll_offset = new_scroll;
        }
    }
    None
}
