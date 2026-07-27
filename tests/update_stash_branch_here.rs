use magi::{
    git::{checkout::get_branches, test_repo::TestRepo},
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

fn branch_stash_here_config() -> ShowSelectPopupConfig {
    ShowSelectPopupConfig {
        title: "Branch stash here".to_string(),
        source: OptionsSource::Stashes,
        on_select: OnSelect::BranchStashHere,
    }
}

/// Creates a stash containing a tracked-file modification.
fn create_tracked_stash(test_repo: &TestRepo) {
    test_repo.commit_file("test.txt", "initial", "Initial commit");
    test_repo.write_file_content("test.txt", "changed");
    test_repo.create_stash("test stash");
}

#[test]
fn test_branch_stash_here_on_stash_entry_shows_input() {
    let test_repo = TestRepo::new();
    create_tracked_stash(&test_repo);

    let mut model = create_model_from_test_repo(&test_repo);

    // Place the cursor on the stash entry
    let stash_pos =
        find_line(&model, |c| matches!(c, LineContent::Stash(_))).expect("Should find stash entry");
    model.ui_model.cursor_position = stash_pos;

    // Press 'B' (via ShowSelectPopup with BranchStashHere)
    let result = update(
        &mut model,
        Message::ShowSelectPopup(branch_stash_here_config()),
    );
    assert_eq!(result, None);

    // Cursor is on a stash, so skip the select popup and ask for the branch name
    match &model.popup {
        Some(PopupContent::Input(state)) => {
            assert_eq!(
                state.context,
                InputContext::StashBranchHere {
                    stash_ref: "stash@{0}".to_string()
                }
            );
        }
        other => panic!("Expected input popup, got {:?}", other),
    }
}

#[test]
fn test_branch_stash_here_elsewhere_opens_select_popup_then_input() {
    let test_repo = TestRepo::new();
    create_tracked_stash(&test_repo);

    let mut model = create_model_from_test_repo(&test_repo);

    // Cursor at the top (not on a stash line)
    model.ui_model.cursor_position = 0;

    let result = update(
        &mut model,
        Message::ShowSelectPopup(branch_stash_here_config()),
    );
    assert_eq!(result, None);

    // A select popup listing the stashes should be shown
    match &model.popup {
        Some(PopupContent::Command(PopupContentCommand::Select(state))) => {
            assert_eq!(state.title, "Branch stash here");
            assert_eq!(state.on_select, OnSelect::BranchStashHere);
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
                InputContext::StashBranchHere {
                    stash_ref: "stash@{0}".to_string()
                }
            );
        }
        other => panic!("Expected input popup, got {:?}", other),
    }
}

#[test]
fn test_branch_stash_here_without_stashes_shows_error() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.cursor_position = 0;

    let result = update(
        &mut model,
        Message::ShowSelectPopup(branch_stash_here_config()),
    );
    assert_eq!(result, None);

    match &model.popup {
        Some(PopupContent::Error { message }) => {
            assert_eq!(message, "No stashes found");
        }
        other => panic!("Expected error popup, got {:?}", other),
    }
}

#[test]
fn test_branch_name_confirm_returns_stash_branch_here_command() {
    let test_repo = TestRepo::new();
    create_tracked_stash(&test_repo);

    let mut model = create_model_from_test_repo(&test_repo);

    // Simulate the branch name input popup with text entered
    model.popup = Some(PopupContent::Input(InputPopupState::with_text(
        InputContext::StashBranchHere {
            stash_ref: "stash@{0}".to_string(),
        },
        "feature-from-stash",
    )));

    let result = update(&mut model, Message::Input(InputMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::BranchHere {
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
        InputContext::StashBranchHere {
            stash_ref: "stash@{0}".to_string(),
        },
    )));

    let result = update(&mut model, Message::Input(InputMessage::Confirm));

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Input(state)) if state.context
        == InputContext::StashBranchHere {
            stash_ref: "stash@{0}".to_string()
        }),
        "Empty input should keep the input popup open"
    );
}

#[test]
fn test_branch_here_creates_branch_at_head_and_starts_pop() {
    let test_repo = TestRepo::new();
    create_tracked_stash(&test_repo);
    // A second commit after the stash, so HEAD differs from the stash's base
    test_repo.commit_file("later.txt", "later", "Later commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Stash(StashCommand::BranchHere {
            stash_ref: "stash@{0}".to_string(),
            branch_name: "stash-here-branch".to_string(),
        }),
    );
    assert_eq!(result, None);

    // The branch is created and checked out at the current HEAD (the later
    // commit), and the stash pop runs as a PTY command.
    let branches = get_branches(&model.git_info.repository);
    assert!(branches.iter().any(|b| b == "stash-here-branch"));

    let head = model.git_info.repository.head().unwrap();
    assert_eq!(head.shorthand().unwrap(), "stash-here-branch");

    assert!(model.pty_state.is_some());
}

#[test]
fn test_branch_here_with_existing_branch_shows_error() {
    let test_repo = TestRepo::new();
    create_tracked_stash(&test_repo);

    let mut model = create_model_from_test_repo(&test_repo);

    // The current branch name already exists, so checkout -b fails
    let current = model
        .git_info
        .repository
        .head()
        .unwrap()
        .shorthand()
        .unwrap()
        .to_string();

    let result = update(
        &mut model,
        Message::Stash(StashCommand::BranchHere {
            stash_ref: "stash@{0}".to_string(),
            branch_name: current,
        }),
    );
    assert_eq!(result, None);

    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup, got {:?}",
        model.popup
    );
    assert!(model.pty_state.is_none());
}
