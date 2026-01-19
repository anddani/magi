use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Fill input with suggested text (remote/branch)
    if let Some(PopupContent::Command(PopupContentCommand::Push(ref mut state))) = model.popup {
        state.input_text = format!("{}/{}", state.default_remote, state.local_branch);
    }
    None
}
