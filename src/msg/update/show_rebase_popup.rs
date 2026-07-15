use crate::{
    git::{
        config::get_push_remote,
        push::{get_current_branch, get_remotes, get_upstream_branch},
        rebase::rebase_in_progress,
    },
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, RebasePopupState},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let branch = model
        .git_info
        .current_branch()
        .unwrap_or_else(|| "HEAD (detached)".to_string());

    let in_progress = rebase_in_progress(&model.workdir);

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

    let state = RebasePopupState {
        branch,
        in_progress,
        upstream,
        push_remote,
        sole_remote,
    };
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(state)));
    None
}
