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
    let Some(PopupContent::Command(PopupContentCommand::Revert(mut state))) = model.popup.take()
    else {
        return None;
    };
    model.arg_mode = false;

    // Selecting the argument when a value is already set clears it
    if state.gpg_sign.is_some() {
        state.gpg_sign = None;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(state)));
        return None;
    }

    // The select popup falls back to the typed text when nothing matches,
    // so an empty key list (gpg unavailable) still allows manual entry
    let keys = list_secret_keys(&model.workdir);
    let select_state = SelectPopupState::new(
        i18n::t().select_revert_gpg_sign.to_string(),
        keys,
        OnSelect::RevertGpgSign {
            revert_state: state,
        },
    );
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        select_state,
    )));
    None
}
