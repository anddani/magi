use crate::{
    model::{popup::PopupContent, Model},
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    model.popup = Some(PopupContent::Help);
    None
}
