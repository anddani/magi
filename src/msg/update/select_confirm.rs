use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand, SelectContext, SelectResult},
        Model,
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let result =
        if let Some(PopupContent::Command(PopupContentCommand::Select(ref state))) = model.popup {
            if let Some(item) = state.selected_item() {
                SelectResult::Selected(item.to_string())
            } else {
                SelectResult::NoneSelected
            }
        } else {
            return None;
        };

    // Store the result for the caller to retrieve
    model.select_result = Some(result.clone());

    // Dismiss the popup
    model.popup = None;

    // Check context and return appropriate follow-up message
    let context = model.select_context.take();
    match (context, result) {
        (Some(SelectContext::CheckoutBranch), SelectResult::Selected(branch)) => {
            Some(Message::CheckoutBranch(branch))
        }
        _ => None,
    }
}
