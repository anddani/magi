use crate::{model::Model, msg::Message};

pub fn update(model: &mut Model) -> Option<Message> {
    // Set the anchor to the current cursor position
    model.ui_model.visual_mode_anchor = Some(model.ui_model.cursor_position);
    None
}
