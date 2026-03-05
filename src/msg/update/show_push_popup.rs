use crate::{
    git::{
        config::get_push_remote,
        push::{get_current_branch, get_remotes, get_upstream_branch},
    },
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, PushPopupState},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get the upstream branch if set
    let upstream = get_upstream_branch(&model.workdir).ok().flatten();

    let push_remote = get_current_branch(&model.workdir)
        .ok()
        .flatten()
        .and_then(|branch| get_push_remote(&model.git_info.repository, &branch));

    let remotes = get_remotes(&model.git_info.repository);
    let sole_remote = if remotes.len() == 1 {
        remotes.into_iter().next()
    } else {
        None
    };

    let state = PushPopupState {
        upstream,
        push_remote,
        sole_remote,
    };

    model.popup = Some(PopupContent::Command(PopupContentCommand::Push(state)));
    None
}
