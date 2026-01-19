use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Enter input mode in the push popup
    if let Some(PopupContent::Command(PopupContentCommand::Push(ref mut state))) = model.popup {
        state.input_mode = true;
    }
    None
}
