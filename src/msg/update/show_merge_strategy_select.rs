use crate::{
    i18n,
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::Message,
};

/// The merge strategies offered by `git merge --strategy=`, as in Magit.
/// `git revert --strategy=` accepts the same values.
pub const MERGE_STRATEGIES: [&str; 5] = ["resolve", "recursive", "octopus", "ours", "subtree"];

pub fn update(model: &mut Model) -> Option<Message> {
    model.arg_mode = false;

    // Selecting the argument when a value is already set clears it
    if let Some(strategy) = model
        .arguments
        .as_mut()
        .and_then(|a| a.merge_strategy_mut())
        && strategy.is_some()
    {
        *strategy = None;
        return None;
    }

    let options = MERGE_STRATEGIES.iter().map(|s| s.to_string()).collect();
    let select_state = SelectPopupState::new(
        i18n::t().select_merge_strategy.to_string(),
        options,
        OnSelect::MergeStrategy,
    );
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        select_state,
    )));
    None
}
