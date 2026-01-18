use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model, c: char) -> Option<Message> {
    // Add character to input text in push popup
    if let Some(PopupContent::Command(PopupContentCommand::Push(ref mut state))) = model.popup {
        state.input_text.push(c);
    }
    None
}
