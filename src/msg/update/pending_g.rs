use crate::{model::Model, msg::Message};

pub fn update(model: &mut Model) -> Option<Message> {
    model.pending_g = true;
    None
}
