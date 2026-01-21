use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    if let Some(PopupContent::Command(PopupContentCommand::Select(ref mut state))) = model.popup {
        state.move_down();
    }
    None
}
