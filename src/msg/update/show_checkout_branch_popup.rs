use crate::{
    git::checkout::get_branches,
    model::{
        popup::{PopupContent, PopupContentCommand, SelectPopupState},
        select_popup::SelectContext,
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get all branches (local first, then remote)
    let branches = get_branches(&model.git_info.repository);

    if branches.is_empty() {
        model.popup = Some(PopupContent::Error {
            message: "No branches found".to_string(),
        });
        return None;
    }

    // Set the context so select_confirm knows what to do with the result
    model.select_context = Some(SelectContext::CheckoutBranch);

    // Show the select popup
    let state = SelectPopupState::new("Checkout".to_string(), branches);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(state)));

    None
}
