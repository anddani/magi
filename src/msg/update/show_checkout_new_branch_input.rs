use crate::{
    model::{
        Model,
        popup::{InputContext, InputPopupState, PopupContent},
    },
    msg::Message,
};

pub fn update(model: &mut Model, starting_point: String, checkout: bool) -> Option<Message> {
    // Show the input popup for the new branch name
    let state = InputPopupState::new(
        "Name for new branch".to_string(),
        InputContext::CreateNewBranch {
            starting_point,
            checkout,
        },
    );
    model.popup = Some(PopupContent::Input(state));

    None
}
