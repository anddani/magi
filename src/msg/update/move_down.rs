use crate::{
    model::Model,
    msg::{util::visible_lines_between, Message},
};
pub fn update(model: &mut Model) -> Option<Message> {
    let max_pos = model.ui_model.lines.len().saturating_sub(1);
    // Find the next visible line
    let mut new_pos = model.ui_model.cursor_position;
    while new_pos < max_pos {
        new_pos += 1;
        if !model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections) {
            model.ui_model.cursor_position = new_pos;
            // Scroll down if cursor moves below viewport
            let viewport_height = model.ui_model.viewport_height;
            if viewport_height > 0 {
                // Count visible lines from scroll_offset to cursor (exclusive)
                let visible_before_cursor = visible_lines_between(
                    &model.ui_model.lines,
                    model.ui_model.scroll_offset,
                    model.ui_model.cursor_position,
                    &model.ui_model.collapsed_sections,
                );
                // If visible lines before cursor >= viewport_height, cursor is below viewport
                if visible_before_cursor >= viewport_height {
                    // Scroll so cursor is at bottom of viewport
                    // Walk back from cursor to find scroll position with (viewport_height - 1)
                    // visible lines before cursor
                    let mut new_scroll = model.ui_model.cursor_position;
                    let mut visible_count = 0;
                    while new_scroll > 0 && visible_count < viewport_height - 1 {
                        new_scroll -= 1;
                        if !model.ui_model.lines[new_scroll]
                            .is_hidden(&model.ui_model.collapsed_sections)
                        {
                            visible_count += 1;
                        }
                    }
                    model.ui_model.scroll_offset = new_scroll;
                }
            }
            break;
        }
    }
    None
}
