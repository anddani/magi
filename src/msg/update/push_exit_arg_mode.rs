use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Exit argument selection mode in the push popup
    if let Some(PopupContent::Command(PopupContentCommand::Push(ref mut state))) = model.popup {
        state.arg_mode = false;
    }
    None
}
