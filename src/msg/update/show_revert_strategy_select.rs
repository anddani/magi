use crate::{
    i18n,
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::Message,
};

/// The merge strategies offered by `git revert --strategy=`, as in Magit.
pub const REVERT_STRATEGIES: [&str; 5] = ["resolve", "recursive", "octopus", "ours", "subtree"];

pub fn update(model: &mut Model) -> Option<Message> {
    let Some(PopupContent::Command(PopupContentCommand::Revert(mut state))) = model.popup.take()
    else {
        return None;
    };
    model.equals_arg_mode = false;

    // Selecting the argument when a value is already set clears it
    if state.strategy.is_some() {
        state.strategy = None;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(state)));
        return None;
    }

    let options = REVERT_STRATEGIES.iter().map(|s| s.to_string()).collect();
    let select_state = SelectPopupState::new(
        i18n::t().select_revert_strategy.to_string(),
        options,
        OnSelect::RevertStrategy {
            revert_state: state,
        },
    );
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        select_state,
    )));
    None
}
