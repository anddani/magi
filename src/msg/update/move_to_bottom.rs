use crate::{
    model::Model,
    msg::{Message, util::visible_lines_between},
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Find the last visible line
    for i in (0..model.ui_model.lines.len()).rev() {
        if !model.ui_model.lines[i].is_hidden(&model.ui_model.collapsed_sections) {
            model.ui_model.cursor_position = i;

            // Adjust scroll so cursor is visible at bottom of viewport
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
