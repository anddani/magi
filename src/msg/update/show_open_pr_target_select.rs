use crate::{
    git::checkout::get_local_branches,
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
        select_popup::SelectContext,
    },
    msg::Message,
};

/// Shows a select popup to pick the target branch for a PR.
/// Stores `branch` (the source) on the model so it can be retrieved after target selection.
pub fn update(model: &mut Model, branch: String) -> Option<Message> {
    let branches: Vec<String> = get_local_branches(&model.git_info.repository)
        .into_iter()
        .filter(|b| b != &branch)
        .collect();

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No other local branches found".to_string(),
        });
        return None;
    }

    model.open_pr_branch = Some(branch);
    model.select_context = Some(SelectContext::OpenPrTarget);

    let state = SelectPopupState::new("Open PR to".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    None
}
