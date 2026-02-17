use crate::{
    model::{
        Model,
        popup::{InputContext, InputPopupState, PopupContent},
    },
    msg::Message,
};

pub fn update(model: &mut Model, old_name: String) -> Option<Message> {
    let state = InputPopupState::new(
        format!("Rename branch '{}' to:", old_name),
        InputContext::RenameBranch { old_name },
    );
    model.popup = Some(PopupContent::Input(state));

    None
}
