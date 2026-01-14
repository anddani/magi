use crate::{model::Model, msg::Message};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get the current line and check if it's a collapsible header
    if let Some(line) = model.ui_model.lines.get(model.ui_model.cursor_position) {
        if let Some(section) = line.collapsible_section() {
            // Toggle the section in collapsed_sections
            if model.ui_model.collapsed_sections.contains(&section) {
                model.ui_model.collapsed_sections.remove(&section);
            } else {
                model.ui_model.collapsed_sections.insert(section);
            }
        }
    }
    None
}
