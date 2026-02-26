use magi::{
    git::test_repo::TestRepo,
    model::popup::{InputContext, PopupContent},
    msg::{Message, StashCommand, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

#[test]
fn test_show_stash_index_input_opens_input_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowStashIndexInput);

    assert_eq!(result, None);

    assert!(matches!(
        model.popup,
        Some(PopupContent::Input(ref state))
            if state.title == "Stash index message"
            && matches!(state.context, InputContext::StashIndexMessage)
    ));
}

#[test]
fn test_confirm_stash_index_input_with_message_triggers_stash_index() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    // Open the index input popup
    update(&mut model, Message::ShowStashIndexInput);

    // Type a message
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::InputChar('m')),
    );
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::InputChar('y')),
    );
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::InputChar(' ')),
    );
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::InputChar('s')),
    );
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::InputChar('t')),
    );
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::InputChar('a')),
    );
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::InputChar('s')),
    );
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::InputChar('h')),
    );

    // Confirm
    let result = update(&mut model, Message::Input(magi::msg::InputMessage::Confirm));

    assert_eq!(model.popup, None);
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::StashIndex(
            "my stash".to_string()
        )))
    );
}

#[test]
fn test_confirm_stash_index_input_empty_triggers_stash_index_with_empty_message() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    // Open the index input popup
    update(&mut model, Message::ShowStashIndexInput);

    // Confirm with empty input (allowed, git uses default message)
    let result = update(&mut model, Message::Input(magi::msg::InputMessage::Confirm));

    assert_eq!(model.popup, None);
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::StashIndex(String::new())))
    );
}
