use crate::{model::Model, msg::Message};

pub fn update(model: &mut Model) -> Option<Message> {
    // Clear the visual mode anchor
    model.ui_model.visual_mode_anchor = None;
    None
}
