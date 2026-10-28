use crate::{
    model::{
        InputField, Model,
        arguments::DEFAULT_LOG_LIMIT,
        popup::{InputContext, InputPopupState, PopupContent},
    },
    msg::Message,
};

/// Open the input popup for entering the `-n` commit limit, prefilled with the
/// current limit. The log popup is left in `model.popup` conceptually via the
/// arguments (the limit lives in `model.arguments`), so on confirm we write the
/// new value back and reopen the log popup.
pub fn update(model: &mut Model) -> Option<Message> {
    let current = model
        .arguments
        .as_ref()
        .and_then(|a| a.log_limit())
        .unwrap_or(DEFAULT_LOG_LIMIT);
    let state = InputPopupState {
        input: InputField::from_text(current.to_string()),
        context: InputContext::LogLimit,
    };
    model.popup = Some(PopupContent::Input(state));
    None
}
