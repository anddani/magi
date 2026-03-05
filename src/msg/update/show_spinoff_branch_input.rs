use crate::{
    model::{
        Model,
        popup::{InputContext, InputPopupState, PopupContent},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let state = InputPopupState::new(
        "Name for new spin-off branch".to_string(),
        InputContext::SpinoffBranch,
    );
    model.popup = Some(PopupContent::Input(state));
    None
}
