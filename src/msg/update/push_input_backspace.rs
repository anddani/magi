use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Remove last character from input text in push popup
    if let Some(PopupContent::Command(PopupContentCommand::Push(ref mut state))) = model.popup {
        state.input_text.pop();
    }
    None
}
