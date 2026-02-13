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
    let remotes = get_remotes(&model.git_info.repository);

    if remotes.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No remotes configured".to_string(),
        });
        return None;
    }

    model.select_context = Some(SelectContext::FetchElsewhere);

    let state = SelectPopupState::new("Fetch from".to_string(), remotes);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    None
}
