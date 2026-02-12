use crate::{
    git::push::get_local_tags,
    model::{
        Model,
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
        select_popup::SelectContext,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get local tags
    let tags = get_local_tags(&model.git_info.repository);

    if tags.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No tags to push".to_string(),
        });
        return None;
    }

    // Set the context so select_confirm knows what to do with the result
    model.select_context = Some(SelectContext::PushTag);

    // Show the select popup
    let state = SelectPopupState::new("Push tag".to_string(), tags);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    None
}
