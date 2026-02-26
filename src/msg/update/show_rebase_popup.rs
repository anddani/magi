use crate::{
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

    let state = RebasePopupState { branch };
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(state)));
    None
}
