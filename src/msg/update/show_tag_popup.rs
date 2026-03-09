use crate::{
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, TagPopupState},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let state = TagPopupState {};
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag(state)));
    None
}
