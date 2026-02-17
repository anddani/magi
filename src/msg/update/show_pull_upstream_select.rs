use crate::{
    git::{
        checkout::get_remote_branches_for_upstream,
        push::{get_current_branch, get_remotes},
    },
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
        select_popup::SelectContext,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get the current branch name for suggested upstream
    let local_branch = get_current_branch(&model.workdir)
        .ok()
        .flatten()
        .unwrap_or_default();

    // Get configured remotes
    let remotes = get_remotes(&model.git_info.repository);
    let default_remote = remotes.into_iter().next().unwrap_or_default();

    // Build suggested upstream
    let suggested = if !default_remote.is_empty() && !local_branch.is_empty() {
        Some(format!("{}/{}", default_remote, local_branch))
    } else {
        None
    };

    // Get remote branches with suggested upstream first
    let branches =
        get_remote_branches_for_upstream(&model.git_info.repository, suggested.as_deref());

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No remote branches found".to_string(),
        });
        return None;
    }

    // Set the context so select_confirm knows what to do with the result
    model.select_context = Some(SelectContext::PullUpstream);

    // Show the select popup
    let state = SelectPopupState::new("Pull from".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    None
}
