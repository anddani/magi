use crate::{
    git::rebase::rebase_in_progress,
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

    let state = RebasePopupState {
        branch,
        in_progress,
    };
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(state)));
    None
}
