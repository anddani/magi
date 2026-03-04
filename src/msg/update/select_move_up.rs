use crate::{
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    if let Some(PopupContent::Command(PopupContentCommand::Select(state))) = &mut model.popup {
        state.move_up();
    }
    None
}
