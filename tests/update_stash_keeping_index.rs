use magi::model::EditOp;
use magi::{
    git::test_repo::TestRepo,
    model::popup::{InputContext, PopupContent},
    msg::{Message, StashCommand, StashType, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

#[test]
fn test_show_stash_keeping_index_input_opens_input_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowStashInput(StashType::KeepingIndex));

    assert_eq!(result, None);

    assert!(matches!(
        model.popup,
        Some(PopupContent::Input(ref state))
            if state.title() == "Stash keeping index message"
            && matches!(state.context, InputContext::Stash(StashType::KeepingIndex))
    ));
}

#[test]
fn test_confirm_stash_keeping_index_input_with_message_triggers_stash_keeping_index() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    update(&mut model, Message::ShowStashInput(StashType::KeepingIndex));

    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::Edit(EditOp::Insert('w'))),
    );
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::Edit(EditOp::Insert('i'))),
    );
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::Edit(EditOp::Insert('p'))),
    );

    let result = update(&mut model, Message::Input(magi::msg::InputMessage::Confirm));

    assert_eq!(model.popup, None);
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::Push(
            StashType::KeepingIndex,
            "wip".to_string()
        )))
    );
}

#[test]
fn test_confirm_stash_keeping_index_input_empty_triggers_stash_keeping_index_with_empty_message() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    update(&mut model, Message::ShowStashInput(StashType::KeepingIndex));

    let result = update(&mut model, Message::Input(magi::msg::InputMessage::Confirm));

    assert_eq!(model.popup, None);
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::Push(
            StashType::KeepingIndex,
            String::new()
        )))
    );
}
