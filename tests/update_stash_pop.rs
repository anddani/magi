use magi::{
    git::test_repo::TestRepo,
    model::{
        LineContent,
        popup::{ConfirmAction, ConfirmPopupState, PopupContent, PopupContentCommand},
        select_popup::OnSelect,
    },
    msg::{Message, OptionsSource, ShowSelectPopupConfig, StashCommand, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

#[test]
fn test_pop_stash_on_stash_entry_shows_confirm() {
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

    // Press 'p' key to pop stash
    let result = update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Pop stash".to_string(),
            source: OptionsSource::Stashes,
            on_select: OnSelect::PopStash,
        }),
    );

    // Should show confirmation popup
    assert!(matches!(
        model.popup,
        Some(PopupContent::Confirm(ConfirmPopupState {
            message: _,
            on_confirm: ConfirmAction::PopStash(ref stash_ref)
        })) if stash_ref.starts_with("stash@{")
    ));

    if let Some(PopupContent::Confirm(state)) = &model.popup {
        assert!(state.message.starts_with("Pop stash@{0}"));
    }

    assert_eq!(result, None);
}

#[test]
fn test_pop_stash_not_on_entry_shows_select() {
    let test_repo = TestRepo::new();
    test_repo.create_file("test.txt");
    test_repo.create_stash("test stash 1");
    test_repo.create_file("test2.txt");
    test_repo.create_stash("test stash 2");

    let mut model = create_model_from_test_repo(&test_repo);

    // Set cursor to the Stashes section header
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

    // Press 'p' key to pop stash
    let result = update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Pop stash".to_string(),
            source: OptionsSource::Stashes,
            on_select: OnSelect::PopStash,
        }),
    );

    // Should show select popup
    assert!(matches!(
        model.popup,
        Some(PopupContent::Command(PopupContentCommand::Select(_)))
    ));

    if let Some(PopupContent::Command(PopupContentCommand::Select(state))) = &model.popup {
        assert_eq!(state.title, "Pop stash");
        assert_eq!(state.all_options.len(), 2);
    }

    assert_eq!(result, None);
}

#[test]
fn test_pop_stash_no_stashes_shows_error() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);

    // Press 'p' key to pop stash (no stashes exist)
    let result = update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Pop stash".to_string(),
            source: OptionsSource::Stashes,
            on_select: OnSelect::PopStash,
        }),
    );

    // Should show error popup
    assert!(matches!(
        model.popup,
        Some(PopupContent::Error { message: _ })
    ));

    if let Some(PopupContent::Error { message }) = &model.popup {
        assert_eq!(message, "No stashes found");
    }

    assert_eq!(result, None);
}

#[test]
fn test_confirm_pop_stash_triggers_stash_command() {
    let test_repo = TestRepo::new();
    test_repo.create_file("test.txt");
    test_repo.create_stash("test stash");

    let mut model = create_model_from_test_repo(&test_repo);

    // Set up confirmation popup
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: "Pop stash@{0}?".to_string(),
        on_confirm: ConfirmAction::PopStash("stash@{0}".to_string()),
    }));

    // Confirm the pop
    let result = update(
        &mut model,
        Message::ConfirmPopStash("stash@{0}".to_string()),
    );

    // Popup should be cleared
    assert_eq!(model.popup, None);

    // Should return StashCommand::Pop message
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::Pop("stash@{0}".to_string())))
    );
}
