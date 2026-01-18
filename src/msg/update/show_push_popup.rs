use crate::{
    git::push::{get_current_branch, get_upstream_branch},
    model::{
        popup::{PopupContent, PopupContentCommand, PushPopupState},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get the current branch name
    let local_branch = if let Some(repo_path) = model.git_info.repository.workdir() {
        get_current_branch(repo_path).ok().flatten()
    } else {
        None
    };

    let Some(local_branch) = local_branch else {
        // Can't push in detached HEAD state or if we can't get branch name
        model.popup = Some(PopupContent::Error {
            message: "Cannot push: not on a branch".to_string(),
        });
        return None;
    };

    // Get the upstream branch if set
    let upstream = if let Some(repo_path) = model.git_info.repository.workdir() {
        get_upstream_branch(repo_path).ok().flatten()
    } else {
        None
    };

    let state = PushPopupState {
        local_branch,
        upstream,
        input_mode: false,
        input_text: String::new(),
    };

    model.popup = Some(PopupContent::Command(PopupContentCommand::Push(state)));
    None
}
