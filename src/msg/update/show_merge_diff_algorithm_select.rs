use crate::{
    i18n,
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::Message,
};

/// The diff algorithms offered by `-Xdiff-algorithm=`, as in Magit.
pub const DIFF_ALGORITHMS: [&str; 4] = ["default", "minimal", "patience", "histogram"];

pub fn update(model: &mut Model) -> Option<Message> {
    model.arg_mode = false;

    // Selecting the argument when a value is already set clears it
    if let Some(diff_algorithm) = model
        .arguments
        .as_mut()
        .and_then(|a| a.merge_diff_algorithm_mut())
        && diff_algorithm.is_some()
    {
        *diff_algorithm = None;
        return None;
    }

    let options = DIFF_ALGORITHMS.iter().map(|s| s.to_string()).collect();
    let select_state = SelectPopupState::new(
        i18n::t().select_merge_diff_algorithm.to_string(),
        options,
        OnSelect::MergeDiffAlgorithm,
    );
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        select_state,
    )));
    None
}
