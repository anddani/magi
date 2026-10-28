use crate::{
    git::gpg::list_secret_keys,
    i18n,
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
    if let Some(gpg_sign) = model
        .arguments
        .as_mut()
        .and_then(|a| a.merge_gpg_sign_mut())
        && gpg_sign.is_some()
    {
        *gpg_sign = None;
        return None;
    }

    // The select popup falls back to the typed text when nothing matches,
    // so an empty key list (gpg unavailable) still allows manual entry
    let keys = list_secret_keys(&model.workdir);
    let select_state = SelectPopupState::new(
        i18n::t().select_merge_gpg_sign.to_string(),
        keys,
        OnSelect::MergeGpgSign,
    );
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        select_state,
    )));
    None
}
