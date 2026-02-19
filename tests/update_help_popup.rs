use magi::{
    model::PopupContent,
    msg::{Message, update::update},
};

use crate::utils::create_test_model_with_lines;

mod utils;

#[test]
fn test_show_help_sets_popup() {
    let mut model = create_test_model_with_lines(10);

    // Popup should be None initially
    assert!(model.popup.is_none());

    // Show help
    update(&mut model, Message::ShowPopup(PopupContent::Help));

    // Popup should now be Help
    assert_eq!(model.popup, Some(PopupContent::Help));
}

#[test]
fn test_dismiss_popup_clears_help() {
    let mut model = create_test_model_with_lines(10);

    // Show help first
    update(&mut model, Message::ShowPopup(PopupContent::Help));
    assert_eq!(model.popup, Some(PopupContent::Help));

    // Dismiss the popup
    update(&mut model, Message::DismissPopup);

    // Popup should be cleared
    assert!(model.popup.is_none());
}

#[test]
fn test_show_help_returns_none() {
    let mut model = create_test_model_with_lines(10);

    // ShowHelp should not trigger a follow-up message
    let follow_up = update(&mut model, Message::ShowPopup(PopupContent::Help));
    assert_eq!(follow_up, None);
}
