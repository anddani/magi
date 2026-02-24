use crate::{
    git::{config::get_push_remote, push::get_current_branch, push::get_upstream_branch},
    model::{
        Model,
        popup::{FetchPopupState, PopupContent, PopupContentCommand},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get the upstream branch if set
    let upstream = get_upstream_branch(&model.workdir).ok().flatten();

    // Get push_remote from git config: branch.<name>.pushRemote
    let push_remote = get_current_branch(&model.workdir)
        .ok()
        .flatten()
        .and_then(|branch| get_push_remote(&model.git_info.repository, &branch));

    let state = FetchPopupState {
        upstream,
        push_remote,
    };

    model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(state)));
    None
}
