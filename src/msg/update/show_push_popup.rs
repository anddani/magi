use crate::{
    git::push::{get_current_branch, get_remotes, get_upstream_branch},
    model::{
        popup::{PopupContent, PopupContentCommand, PushPopupState},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let Some(repo_path) = model.git_info.repository.workdir() else {
        model.popup = Some(PopupContent::Error {
            message: "Repository working directory not found".to_string(),
        });
        return None;
    };

    // Get the current branch name
    let local_branch = get_current_branch(repo_path).ok().flatten();

    let Some(local_branch) = local_branch else {
        // Can't push in detached HEAD state or if we can't get branch name
        model.popup = Some(PopupContent::Error {
            message: "Cannot push: not on a branch".to_string(),
        });
        return None;
    };

    // Get configured remotes
    let remotes = get_remotes(&model.git_info.repository);

    if remotes.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "Cannot push: no remotes configured".to_string(),
        });
        return None;
    }

    // Use the first remote as the default
    let default_remote = remotes.into_iter().next().unwrap();

    // Get the upstream branch if set
    let upstream = get_upstream_branch(repo_path).ok().flatten();

    let state = PushPopupState {
        local_branch,
        upstream,
        default_remote,
        input_mode: false,
        input_text: String::new(),
    };

    model.popup = Some(PopupContent::Command(PopupContentCommand::Push(state)));
    None
}
