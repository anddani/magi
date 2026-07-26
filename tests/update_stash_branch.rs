use magi::{
    git::test_repo::TestRepo,
    model::{
        LineContent,
        popup::{InputContext, InputPopupState, PopupContent, PopupContentCommand},
        select_popup::OnSelect,
    },
    msg::{
        InputMessage, Message, OptionsSource, SelectMessage, ShowSelectPopupConfig, StashCommand,
        update::update,
    },
};

mod utils;
use utils::{create_model_from_test_repo, find_line};

fn branch_stash_config() -> ShowSelectPopupConfig {
    ShowSelectPopupConfig {
        title: "Branch stash".to_string(),
        source: OptionsSource::Stashes,
        on_select: OnSelect::BranchStash,
    }
}

/// Creates a stash containing a tracked-file modification.
fn create_tracked_stash(test_repo: &TestRepo) {
    test_repo.commit_file("test.txt", "initial", "Initial commit");
    test_repo.write_file_content("test.txt", "changed");
    test_repo.create_stash("test stash");
}

#[test]
fn test_branch_stash_on_stash_entry_shows_input() {
    let test_repo = TestRepo::new();
    create_tracked_stash(&test_repo);

    let mut model = create_model_from_test_repo(&test_repo);

    // Place the cursor on the stash entry
    let stash_pos =
        find_line(&model, |c| matches!(c, LineContent::Stash(_))).expect("Should find stash entry");
    model.ui_model.cursor_position = stash_pos;

    // Press 'b' (via ShowSelectPopup with BranchStash)
    let result = update(&mut model, Message::ShowSelectPopup(branch_stash_config()));
    assert_eq!(result, None);

    // Cursor is on a stash, so skip the select popup and ask for the branch name
    match &model.popup {
        Some(PopupContent::Input(state)) => {
            assert_eq!(
                state.context,
                InputContext::StashBranch {
                    stash_ref: "stash@{0}".to_string()
                }
            );
        }
        other => panic!("Expected input popup, got {:?}", other),
    }
}

#[test]
fn test_branch_stash_elsewhere_opens_select_popup_then_input() {
    let test_repo = TestRepo::new();
    create_tracked_stash(&test_repo);

    let mut model = create_model_from_test_repo(&test_repo);

    // Cursor at the top (not on a stash line)
    model.ui_model.cursor_position = 0;

    let result = update(&mut model, Message::ShowSelectPopup(branch_stash_config()));
    assert_eq!(result, None);

    // A select popup listing the stashes should be shown
    match &model.popup {
        Some(PopupContent::Command(PopupContentCommand::Select(state))) => {
            assert_eq!(state.title, "Branch stash");
            assert_eq!(state.on_select, OnSelect::BranchStash);
            assert!(state.all_options[0].starts_with("stash@{0}"));
        }
        other => panic!("Expected select popup, got {:?}", other),
    }

    // Confirm the selection: asks for the branch name
    let result = update(&mut model, Message::Select(SelectMessage::Confirm));
    assert_eq!(result, None);

    match &model.popup {
        Some(PopupContent::Input(state)) => {
            assert_eq!(
                state.context,
                InputContext::StashBranch {
                    stash_ref: "stash@{0}".to_string()
                }
            );
        }
        other => panic!("Expected input popup, got {:?}", other),
    }
}

#[test]
fn test_branch_stash_without_stashes_shows_error() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.cursor_position = 0;

    let result = update(&mut model, Message::ShowSelectPopup(branch_stash_config()));
    assert_eq!(result, None);

    match &model.popup {
        Some(PopupContent::Error { message }) => {
            assert_eq!(message, "No stashes found");
        }
        other => panic!("Expected error popup, got {:?}", other),
    }
}

#[test]
fn test_branch_name_confirm_returns_stash_branch_command() {
    let test_repo = TestRepo::new();
    create_tracked_stash(&test_repo);

    let mut model = create_model_from_test_repo(&test_repo);

    // Simulate the branch name input popup with text entered
    model.popup = Some(PopupContent::Input(InputPopupState::with_text(
        InputContext::StashBranch {
            stash_ref: "stash@{0}".to_string(),
        },
        "feature-from-stash",
    )));

    let result = update(&mut model, Message::Input(InputMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::Branch {
            stash_ref: "stash@{0}".to_string(),
            branch_name: "feature-from-stash".to_string(),
        }))
    );
}

#[test]
fn test_branch_name_empty_input_keeps_popup() {
    let test_repo = TestRepo::new();
    create_tracked_stash(&test_repo);

    let mut model = create_model_from_test_repo(&test_repo);

    model.popup = Some(PopupContent::Input(InputPopupState::new(
        InputContext::StashBranch {
            stash_ref: "stash@{0}".to_string(),
        },
    )));

    let result = update(&mut model, Message::Input(InputMessage::Confirm));

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Input(state)) if state.context
        == InputContext::StashBranch {
            stash_ref: "stash@{0}".to_string()
        }),
        "Empty input should keep the input popup open"
    );
}
