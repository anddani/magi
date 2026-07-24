use crate::{
    git::gpg::list_secret_keys,
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    model.arg_mode = false;

    // Selecting the argument when a value is already set clears it
    if let Some(local_user) = model
        .arguments
        .as_mut()
        .and_then(|a| a.tag_local_user_mut())
        && local_user.is_some()
    {
        *local_user = None;
        return None;
    }

    // The select popup falls back to the typed text when nothing matches,
    // so an empty key list (gpg unavailable) still allows manual entry
    let keys = list_secret_keys(&model.workdir);
    let select_state = SelectPopupState::new("Sign as".to_string(), keys, OnSelect::TagSignAs);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        select_state,
    )));
    None
}
