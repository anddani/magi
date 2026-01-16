use crate::{
    model::{DialogContent, Model},
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    model.dialog = Some(DialogContent::Help);
    None
}
