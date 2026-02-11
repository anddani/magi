use crate::{
    git::push::get_remotes,
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
        select_popup::SelectContext,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get configured remotes
    let remotes = get_remotes(&model.git_info.repository);

    if remotes.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No remotes configured".to_string(),
        });
        return None;
    }

    // Set the context so select_confirm knows what to do with the result
    model.select_context = Some(SelectContext::PushAllTags);

    // Show the select popup
    let state = SelectPopupState::new("Push tags to".to_string(), remotes);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    None
}
