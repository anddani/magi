use crate::{
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    match &mut model.popup {
        Some(PopupContent::Command(PopupContentCommand::Select(state))) => {
            state.input_text.pop();
            state.update_filter();
        }
        Some(PopupContent::Command(PopupContentCommand::CommitSelect(state))) => {
            state.input_text.pop();
            state.update_filter();
        }
        _ => {}
    }
    None
}
