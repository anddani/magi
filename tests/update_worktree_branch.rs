use crossterm::event::KeyCode;
use magi::model::EditOp;
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::popup::{InputContext, PopupContent, PopupContentCommand},
    msg::{
        InputMessage, Message, OnSelect, OptionsSource, SelectMessage, ShowSelectPopupConfig,
        update::update,
    },
};

mod utils;
use utils::{create_model_from_test_repo, expect_input_popup, expect_select_popup, key};

fn type_text(model: &mut magi::model::Model, text: &str) {
    for c in text.chars() {
        update(model, Message::Input(InputMessage::Edit(EditOp::Insert(c))));
    }
}

// ── Key binding: 'c' in worktree popup shows starting point select ────────────

#[test]
fn test_c_in_worktree_popup_shows_starting_point_select() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Worktree));

    let result = handle_key(key(KeyCode::Char('c')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Create branch starting at".to_string(),
            source: OptionsSource::BranchesAndTags,
            on_select: OnSelect::WorktreeBranch,
        }))
    );
}

// ── Selecting a starting point asks for the branch name ───────────────────────

#[test]
fn test_select_starting_point_dispatches_branch_name_input() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Create branch starting at".to_string(),
            source: OptionsSource::BranchesAndTags,
            on_select: OnSelect::WorktreeBranch,
        }),
    );

    let state = expect_select_popup(&model);
    assert_eq!(state.selected_item(), Some("main"));

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));
    assert_eq!(
        result,
        Some(Message::ShowWorktreeBranchNameInput {
            starting_point: "main".to_string(),
        })
    );
}

// ── ShowWorktreeBranchNameInput shows input popup ──────────────────────────────

#[test]
fn test_show_branch_name_input_sets_input_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::ShowWorktreeBranchNameInput {
            starting_point: "main".to_string(),
        },
    );

    assert_eq!(result, None);
    let state = expect_input_popup(&model);
    assert_eq!(
        state.context,
        InputContext::WorktreeBranchName {
            starting_point: "main".to_string(),
        }
    );
    assert!(
        state.title().contains("branch"),
        "Title should mention branch, got: {}",
        state.title()
    );
}

// ── Confirming the branch name asks for the worktree path ─────────────────────

#[test]
fn test_confirm_branch_name_dispatches_path_input() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::ShowWorktreeBranchNameInput {
            starting_point: "main".to_string(),
        },
    );

    type_text(&mut model, "feature");
    let result = update(&mut model, Message::Input(InputMessage::Confirm));
    assert_eq!(
        result,
        Some(Message::ShowWorktreeBranchPathInput {
            starting_point: "main".to_string(),
            branch_name: "feature".to_string(),
        })
    );
}

#[test]
fn test_path_input_popup_title_contains_branch_name() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::ShowWorktreeBranchPathInput {
            starting_point: "main".to_string(),
            branch_name: "feature".to_string(),
        },
    );

    let state = expect_input_popup(&model);
    assert_eq!(
        state.context,
        InputContext::WorktreeBranchPath {
            starting_point: "main".to_string(),
            branch_name: "feature".to_string(),
        }
    );
    assert!(
        state.title().contains("feature"),
        "Title should contain the branch name, got: {}",
        state.title()
    );
}

// ── Confirming the path dispatches WorktreeBranch ──────────────────────────────

#[test]
fn test_confirm_path_dispatches_worktree_branch_message() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::ShowWorktreeBranchPathInput {
            starting_point: "main".to_string(),
            branch_name: "feature".to_string(),
        },
    );

    type_text(&mut model, "../feature-wt");
    let result = update(&mut model, Message::Input(InputMessage::Confirm));
    assert_eq!(
        result,
        Some(Message::WorktreeBranch {
            starting_point: "main".to_string(),
            branch_name: "feature".to_string(),
            path: "../feature-wt".to_string(),
        })
    );
}

// ── WorktreeBranch execution ───────────────────────────────────────────────────

#[test]
fn test_worktree_branch_creates_branch_and_worktree_and_switches() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let worktree_path = {
        let tmp = tempfile::tempdir().unwrap();
        tmp.path().to_str().unwrap().to_string()
    };

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::WorktreeBranch {
            starting_point: "main".to_string(),
            branch_name: "feature".to_string(),
            path: worktree_path.clone(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));

    // The new worktree exists with the new branch checked out
    let worktree_repo = git2::Repository::open(&worktree_path).unwrap();
    assert_eq!(
        worktree_repo.head().unwrap().shorthand().unwrap(),
        "feature"
    );

    // The model switched to the new worktree
    assert_eq!(
        model.workdir.canonicalize().unwrap(),
        std::path::Path::new(&worktree_path).canonicalize().unwrap()
    );

    // The original repo still exists and kept its branch
    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    assert!(repo.find_branch("feature", git2::BranchType::Local).is_ok());
}

#[test]
fn test_worktree_branch_duplicate_branch_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    test_repo.create_branch("feature");

    let worktree_path = {
        let tmp = tempfile::tempdir().unwrap();
        tmp.path().to_str().unwrap().to_string()
    };

    let mut model = create_model_from_test_repo(&test_repo);
    let workdir_before = model.workdir.clone();
    let result = update(
        &mut model,
        Message::WorktreeBranch {
            starting_point: "main".to_string(),
            branch_name: "feature".to_string(),
            path: worktree_path,
        },
    );

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup for duplicate branch name"
    );
    assert_eq!(model.workdir, workdir_before, "workdir should not change");
}

#[test]
fn test_worktree_branch_invalid_starting_point_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let worktree_path = {
        let tmp = tempfile::tempdir().unwrap();
        tmp.path().to_str().unwrap().to_string()
    };

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::WorktreeBranch {
            starting_point: "nonexistent-ref".to_string(),
            branch_name: "feature".to_string(),
            path: worktree_path,
        },
    );

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup for invalid starting point"
    );
}
