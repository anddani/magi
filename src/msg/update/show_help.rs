use crate::{
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    model.popup = Some(PopupContent::Help);
    None
}
