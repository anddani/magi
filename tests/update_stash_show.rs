use magi::{
    git::test_repo::TestRepo,
    model::{
        LineContent, ViewMode,
        popup::{PopupContent, PopupContentCommand},
        select_popup::OnSelect,
    },
    msg::{Message, OptionsSource, SelectMessage, ShowSelectPopupConfig, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, find_line};

fn show_stash_config() -> ShowSelectPopupConfig {
    ShowSelectPopupConfig {
        title: "Show stash".to_string(),
        source: OptionsSource::Stashes,
        on_select: OnSelect::ShowStash,
    }
}

/// Creates a stash containing a tracked-file modification, so that
/// `git stash show -p` produces a diff.
fn create_tracked_stash(test_repo: &TestRepo) {
    test_repo.commit_file("test.txt", "initial", "Initial commit");
    test_repo.write_file_content("test.txt", "changed");
    test_repo.create_stash("test stash");
}

#[test]
fn test_show_stash_on_stash_entry_shows_diff_immediately() {
    let test_repo = TestRepo::new();
    create_tracked_stash(&test_repo);

    let mut model = create_model_from_test_repo(&test_repo);

    // Place the cursor on the stash entry
    let stash_pos =
        find_line(&model, |c| matches!(c, LineContent::Stash(_))).expect("Should find stash entry");
    model.ui_model.cursor_position = stash_pos;

    // Press 'v' (via ShowSelectPopup with ShowStash)
    let result = update(&mut model, Message::ShowSelectPopup(show_stash_config()));

    // Cursor is on a stash, so no popup: show the diff directly
    assert_eq!(result, Some(Message::ShowStashDiff(0)));

    // Process the resulting message: enters preview mode with the stash diff
    let result = update(&mut model, Message::ShowStashDiff(0));
    assert_eq!(result, None);
    assert_eq!(model.popup, None);
    assert_eq!(model.view_mode, ViewMode::Preview);
    assert!(
        model
            .ui_model
            .lines
            .iter()
            .any(|l| matches!(&l.content, LineContent::PreviewLine { .. })),
        "Expected PreviewLine entries"
    );
}

#[test]
fn test_show_stash_elsewhere_opens_select_popup_then_shows_diff() {
    let test_repo = TestRepo::new();
    create_tracked_stash(&test_repo);

    let mut model = create_model_from_test_repo(&test_repo);

    // Cursor at the top (not on a stash line)
    model.ui_model.cursor_position = 0;

    let result = update(&mut model, Message::ShowSelectPopup(show_stash_config()));
    assert_eq!(result, None);

    // A select popup listing the stashes should be shown
    match &model.popup {
        Some(PopupContent::Command(PopupContentCommand::Select(state))) => {
            assert_eq!(state.title, "Show stash");
            assert_eq!(state.on_select, OnSelect::ShowStash);
            assert!(state.all_options[0].starts_with("stash@{0}"));
        }
        other => panic!("Expected select popup, got {:?}", other),
    }

    // Confirm the selection: routes to ShowStashDiff
    let result = update(&mut model, Message::Select(SelectMessage::Confirm));
    assert_eq!(result, Some(Message::ShowStashDiff(0)));

    let result = update(&mut model, Message::ShowStashDiff(0));
    assert_eq!(result, None);
    assert_eq!(model.view_mode, ViewMode::Preview);
    assert!(
        model
            .ui_model
            .lines
            .iter()
            .any(|l| matches!(&l.content, LineContent::PreviewLine { .. })),
        "Expected PreviewLine entries"
    );
}

#[test]
fn test_show_stash_without_stashes_shows_error() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.cursor_position = 0;

    let result = update(&mut model, Message::ShowSelectPopup(show_stash_config()));
    assert_eq!(result, None);

    match &model.popup {
        Some(PopupContent::Error { message }) => {
            assert_eq!(message, "No stashes found");
        }
        other => panic!("Expected error popup, got {:?}", other),
    }
}

#[test]
fn test_show_stash_diff_closes_command_popup() {
    let test_repo = TestRepo::new();
    create_tracked_stash(&test_repo);

    let mut model = create_model_from_test_repo(&test_repo);

    // Simulate the stash command popup still being open
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = update(&mut model, Message::ShowStashDiff(0));
    assert_eq!(result, None);
    assert_eq!(model.popup, None);
    assert_eq!(model.view_mode, ViewMode::Preview);
}
