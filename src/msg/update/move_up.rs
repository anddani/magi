use crate::{model::Model, msg::Message};

pub fn update(model: &mut Model) -> Option<Message> {
    // Find the previous visible line
    let mut new_pos = model.ui_model.cursor_position;
    while new_pos > 0 {
        new_pos -= 1;
        if !model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections) {
            model.ui_model.cursor_position = new_pos;
            // Scroll up if cursor moves above viewport
            if model.ui_model.cursor_position < model.ui_model.scroll_offset {
                model.ui_model.scroll_offset = model.ui_model.cursor_position;
            }
            break;
        }
    }
    None
}
