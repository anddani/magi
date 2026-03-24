use crate::{
    model::{
        Model,
        popup::{InputContext, InputPopupState, PopupContent, PopupContentCommand},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let Some(PopupContent::Command(PopupContentCommand::Revert(revert_state))) = model.popup.take()
    else {
        return None;
    };
    let prefill = revert_state.mainline.clone().unwrap_or_default();
    let state = InputPopupState {
        input_text: prefill,
        context: InputContext::RevertMainline { revert_state },
    };
    model.popup = Some(PopupContent::Input(state));
    None
}
