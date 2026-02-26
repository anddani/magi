use crate::{
    model::{
        Model,
        popup::{InputContext, InputPopupState, PopupContent},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let state = InputPopupState::new(
        "Stash index message".to_string(),
        InputContext::StashIndexMessage,
    );
    model.popup = Some(PopupContent::Input(state));
    None
}
