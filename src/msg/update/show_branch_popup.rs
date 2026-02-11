use crate::{
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    model.popup = Some(PopupContent::Command(PopupContentCommand::Branch));
    None
}
