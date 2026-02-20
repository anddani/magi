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
            state.move_up();
        }
        Some(PopupContent::Command(PopupContentCommand::CommitSelect(state))) => {
            state.move_up();
        }
        _ => {}
    }
    None
}
