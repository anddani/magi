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
                // Use the selected item from filtered results
                SelectResult::Selected(item.to_string())
            } else if !state.input_text.is_empty() {
                // No matches, but user entered text - use the input text directly
                // This allows entering arbitrary values like git hashes
                SelectResult::Selected(state.input_text.clone())
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
        (Some(SelectContext::PushUpstream), SelectResult::Selected(upstream)) => {
            Some(Message::PushToRemote(upstream))
        }
        (Some(SelectContext::FetchUpstream), SelectResult::Selected(upstream)) => {
            Some(Message::FetchFromRemote(upstream))
        }
        _ => None,
    }
}
