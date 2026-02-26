use magi::{
    git::test_repo::TestRepo,
    model::{
        LineContent,
        popup::{ConfirmAction, ConfirmPopupState, PopupContent},
    },
    msg::{Message, StashCommand, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

#[test]
fn test_drop_stash_on_section_header_shows_confirm_all() {
    let test_repo = TestRepo::new();
    test_repo.create_file("test.txt");
    test_repo.create_stash("test stash 1");
    test_repo.create_file("test2.txt");
    test_repo.create_stash("test stash 2");

    let mut model = create_model_from_test_repo(&test_repo);

    // Find the Stashes section header
    let stashes_header_pos = model
        .ui_model
        .lines
        .iter()
        .position(|line| {
            matches!(
                &line.content,
                LineContent::SectionHeader { title, .. } if title == "Stashes"
            )
        })
        .expect("Should find Stashes header");

    model.ui_model.cursor_position = stashes_header_pos;

    // Press 'k' key to drop stash (via ShowSelectDialog -> StashDrop)
    let result = update(
        &mut model,
        Message::ShowSelectDialog(magi::msg::SelectDialog::StashDrop),
    );

    // Should show confirmation popup for dropping all stashes
    assert!(matches!(
        model.popup,
        Some(PopupContent::Confirm(ConfirmPopupState {
            message: _,
            on_confirm: ConfirmAction::DropStash(ref stash_ref)
        })) if stash_ref == "all"
    ));

    if let Some(PopupContent::Confirm(state)) = &model.popup {
        assert_eq!(state.message, "Drop all stashes?");
    }

    assert_eq!(result, None);
}

#[test]
fn test_drop_stash_on_stash_entry_shows_confirm_single() {
    let test_repo = TestRepo::new();
    test_repo.create_file("test.txt");
    test_repo.create_stash("test stash");

    let mut model = create_model_from_test_repo(&test_repo);

    // Find the first stash entry
    let stash_pos = model
        .ui_model
        .lines
        .iter()
        .position(|line| matches!(&line.content, LineContent::Stash(_)))
        .expect("Should find stash entry");

    model.ui_model.cursor_position = stash_pos;

    // Press 'k' key to drop stash
    let result = update(
        &mut model,
        Message::ShowSelectDialog(magi::msg::SelectDialog::StashDrop),
    );

    // Should show confirmation popup for dropping single stash
    assert!(matches!(
        model.popup,
        Some(PopupContent::Confirm(ConfirmPopupState {
            message: _,
            on_confirm: ConfirmAction::DropStash(ref stash_ref)
        })) if stash_ref.starts_with("stash@{")
    ));

    if let Some(PopupContent::Confirm(state)) = &model.popup {
        assert!(state.message.starts_with("Drop stash@{0}"));
    }

    assert_eq!(result, None);
}

#[test]
fn test_confirm_drop_stash_triggers_stash_command() {
    let test_repo = TestRepo::new();
    test_repo.create_file("test.txt");
    test_repo.create_stash("test stash");

    let mut model = create_model_from_test_repo(&test_repo);

    // Set up confirmation popup
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: "Drop stash@{0}?".to_string(),
        on_confirm: ConfirmAction::DropStash("stash@{0}".to_string()),
    }));

    // Confirm the drop
    let result = update(
        &mut model,
        Message::ConfirmDropStash("stash@{0}".to_string()),
    );

    // Popup should be cleared
    assert_eq!(model.popup, None);

    // Should return StashCommand::Drop message
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::Drop("stash@{0}".to_string())))
    );
}

#[test]
fn test_confirm_drop_all_stashes_triggers_clear_command() {
    let test_repo = TestRepo::new();
    test_repo.create_file("test.txt");
    test_repo.create_stash("test stash");

    let mut model = create_model_from_test_repo(&test_repo);

    // Set up confirmation popup for dropping all
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: "Drop all stashes?".to_string(),
        on_confirm: ConfirmAction::DropStash("all".to_string()),
    }));

    // Confirm the drop
    let result = update(&mut model, Message::ConfirmDropStash("all".to_string()));

    // Popup should be cleared
    assert_eq!(model.popup, None);

    // Should return StashCommand::Drop with "all"
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::Drop("all".to_string())))
    );
}
