use crate::{
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    if let Some(PopupContent::Command(PopupContentCommand::Select(ref mut state))) = model.popup {
        state.input_text.pop();
        state.update_filter();
    }
    None
}
