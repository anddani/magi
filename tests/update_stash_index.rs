use magi::{
    git::test_repo::TestRepo,
    model::popup::{InputContext, PopupContent},
    msg::{Message, StashCommand, StashType, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

#[test]
fn test_show_stash_index_input_opens_input_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowStashInput(StashType::Index));

    assert_eq!(result, None);

    assert!(matches!(
        model.popup,
        Some(PopupContent::Input(ref state))
            if state.title() == "Stash index message"
            && matches!(state.context, InputContext::Stash(StashType::Index))
    ));
}

#[test]
fn test_confirm_stash_index_input_with_message_triggers_stash_index() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    update(&mut model, Message::ShowStashInput(StashType::Index));

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

    let result = update(&mut model, Message::Input(magi::msg::InputMessage::Confirm));

    assert_eq!(model.popup, None);
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::Push(
            StashType::Index,
            "my stash".to_string()
        )))
    );
}

#[test]
fn test_confirm_stash_index_input_empty_triggers_stash_index_with_empty_message() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    update(&mut model, Message::ShowStashInput(StashType::Index));

    let result = update(&mut model, Message::Input(magi::msg::InputMessage::Confirm));

    assert_eq!(model.popup, None);
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::Push(
            StashType::Index,
            String::new()
        )))
    );
}
