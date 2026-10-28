use crate::{
    model::{
        InputField, Model,
        popup::{InputContext, InputPopupState, PopupContent},
    },
    msg::Message,
};

/// Open the input popup for entering the `-S` occurrence search pattern,
/// prefilled with the current pattern. The pattern lives in `model.arguments`,
/// so on confirm we write the new value back and reopen the log popup.
pub fn update(model: &mut Model) -> Option<Message> {
    let current = model
        .arguments
        .as_ref()
        .and_then(|a| a.log_search_occurrences())
        .unwrap_or_default();
    let state = InputPopupState {
        input: InputField::from_text(current),
        context: InputContext::LogSearchOccurrences,
    };
    model.popup = Some(PopupContent::Input(state));
    None
}
