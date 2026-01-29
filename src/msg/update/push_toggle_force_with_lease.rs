use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Toggle force-with-lease and exit argument mode
    if let Some(PopupContent::Command(PopupContentCommand::Push(ref mut state))) = model.popup {
        state.force_with_lease = !state.force_with_lease;
        state.arg_mode = false;
    }
    None
}
