use crate::{
    model::{
        Model,
        popup::{InputContext, InputPopupState, PopupContent},
    },
    msg::{Message, StashType},
};

pub fn update(model: &mut Model, stash_type: StashType) -> Option<Message> {
    let state = InputPopupState::new(
        stash_type.title().to_string(),
        InputContext::Stash(stash_type),
    );
    model.popup = Some(PopupContent::Input(state));
    None
}
