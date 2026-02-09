use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch));
    None
}
