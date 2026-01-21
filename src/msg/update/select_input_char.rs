use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model, c: char) -> Option<Message> {
    if let Some(PopupContent::Command(PopupContentCommand::Select(ref mut state))) = model.popup {
        state.input_text.push(c);
        state.update_filter();
    }
    None
}
