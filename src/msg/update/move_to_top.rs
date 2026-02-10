use crate::{model::Model, msg::Message};

pub fn update(model: &mut Model) -> Option<Message> {
    // Find the first visible line
    for (i, line) in model.ui_model.lines.iter().enumerate() {
        if !line.is_hidden(&model.ui_model.collapsed_sections) {
            model.ui_model.cursor_position = i;
            model.ui_model.scroll_offset = i;
            break;
        }
    }
    None
}
