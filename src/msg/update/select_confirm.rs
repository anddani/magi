use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand, SelectResult},
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
    model.select_result = Some(result);

    // Dismiss the popup
    model.popup = None;
    None
}
