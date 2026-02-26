use magi::{
    git::test_repo::TestRepo,
    model::popup::{InputContext, PopupContent},
    msg::{Message, StashCommand, StashType, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

#[test]
fn test_show_stash_workspace_input_opens_input_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowStashInput(StashType::Workspace));

    assert_eq!(result, None);

    assert!(matches!(
        model.popup,
        Some(PopupContent::Input(ref state))
            if state.title == "Stash workspace message"
            && matches!(state.context, InputContext::Stash(StashType::Workspace))
    ));
}

#[test]
fn test_confirm_stash_workspace_input_with_message_triggers_stash_workspace() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    update(&mut model, Message::ShowStashInput(StashType::Workspace));

    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::InputChar('w')),
    );
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::InputChar('i')),
    );
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::InputChar('p')),
    );

    let result = update(&mut model, Message::Input(magi::msg::InputMessage::Confirm));

    assert_eq!(model.popup, None);
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::Push(
            StashType::Workspace,
            "wip".to_string()
        )))
    );
}

#[test]
fn test_confirm_stash_workspace_input_empty_triggers_stash_workspace_with_empty_message() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    update(&mut model, Message::ShowStashInput(StashType::Workspace));

    let result = update(&mut model, Message::Input(magi::msg::InputMessage::Confirm));

    assert_eq!(model.popup, None);
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::Push(
            StashType::Workspace,
            String::new()
        )))
    );
}
