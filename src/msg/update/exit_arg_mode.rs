use crate::{model::Model, msg::Message};

pub fn update(model: &mut Model) -> Option<Message> {
    model.arg_mode = false;
    None
}
