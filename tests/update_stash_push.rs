use std::collections::HashSet;

use magi::{
    git::test_repo::TestRepo,
    model::{
        arguments::{Arguments, StashPushArgument},
        popup::{MultiSelectPopupState, OnMultiSelect, PopupContent, PopupContentCommand},
    },
    msg::{Message, MultiSelectMessage, StashCommand, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

/// The nested stash-push popup is opened straight from the stash popup via
/// `ShowPopup`, so here we focus on the file-picker and confirmation flow.

#[test]
fn test_show_stash_push_files_lists_modified_files() {
    let test_repo = TestRepo::new();
    // Modify the tracked file created by TestRepo::new() so it shows up as modified.
    test_repo.write_file_content("initial_file.txt", "modified content");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowStashPushFiles);
    assert_eq!(result, None);

    match model.popup {
        Some(PopupContent::Command(PopupContentCommand::MultiSelect(ref state))) => {
            assert!(
                state.all_options.iter().any(|f| f == "initial_file.txt"),
                "expected initial_file.txt in options, got {:?}",
                state.all_options
            );
            assert!(state.checked.is_empty());
        }
        other => panic!("expected multi-select popup, got {:?}", other),
    }
}

#[test]
fn test_show_stash_push_files_without_changes_shows_error() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowStashPushFiles);
    assert_eq!(result, None);

    assert!(
        matches!(model.popup, Some(PopupContent::Error { .. })),
        "expected error popup, got {:?}",
        model.popup
    );
}

#[test]
fn test_show_stash_push_files_snapshots_toggled_flags() {
    let test_repo = TestRepo::new();
    test_repo.write_file_content("initial_file.txt", "modified content");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::StashPushArguments(HashSet::from([
        StashPushArgument::KeepIndex,
        StashPushArgument::IncludeUntracked,
    ])));

    update(&mut model, Message::ShowStashPushFiles);

    match model.popup {
        Some(PopupContent::Command(PopupContentCommand::MultiSelect(ref state))) => {
            let OnMultiSelect::StashPushFiles { flags } = &state.on_confirm;
            let mut flags = flags.clone();
            flags.sort();
            assert_eq!(
                flags,
                vec![
                    "--include-untracked".to_string(),
                    "--keep-index".to_string()
                ]
            );
        }
        other => panic!("expected multi-select popup, got {:?}", other),
    }
}

#[test]
fn test_multi_select_toggle_and_confirm_produces_push_files() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    let state = MultiSelectPopupState::new(
        "Select files".to_string(),
        vec!["a.rs".to_string(), "b.rs".to_string()],
        OnMultiSelect::StashPushFiles {
            flags: vec!["--keep-index".to_string()],
        },
    );
    model.popup = Some(PopupContent::Command(PopupContentCommand::MultiSelect(
        state,
    )));

    // Toggle the file under the cursor (a.rs), then confirm.
    update(
        &mut model,
        Message::MultiSelect(MultiSelectMessage::ToggleCurrent),
    );
    let result = update(
        &mut model,
        Message::MultiSelect(MultiSelectMessage::Confirm),
    );

    assert_eq!(model.popup, None);
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::PushFiles {
            files: vec!["a.rs".to_string()],
            flags: vec!["--keep-index".to_string()],
        }))
    );
}

#[test]
fn test_multi_select_confirm_without_selection_stashes_everything() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    let state = MultiSelectPopupState::new(
        "Select files".to_string(),
        vec!["a.rs".to_string(), "b.rs".to_string()],
        OnMultiSelect::StashPushFiles { flags: vec![] },
    );
    model.popup = Some(PopupContent::Command(PopupContentCommand::MultiSelect(
        state,
    )));

    let result = update(
        &mut model,
        Message::MultiSelect(MultiSelectMessage::Confirm),
    );

    assert_eq!(model.popup, None);
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::PushFiles {
            files: vec![],
            flags: vec![],
        }))
    );
}
