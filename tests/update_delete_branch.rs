use crossterm::event::KeyCode;
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::{
        popup::{ConfirmAction, ConfirmPopupState, PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::{Message, OptionsSource, SelectMessage, ShowSelectPopupConfig, update::update},
};

mod utils;
use utils::{
    assert_no_popup, create_model_from_test_repo, expect_confirm_popup, expect_error_popup, key,
};

fn git(workdir: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new("git")
        .arg("-C")
        .arg(workdir)
        .args(args)
        .output()
        .unwrap()
}

// ── Key binding — 'x' in branch popup ─────────────────────────────────────────

#[test]
fn test_x_in_branch_popup_shows_delete_branch_picker() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Branch));

    let result = handle_key(key(KeyCode::Char('x')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Delete branch".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::DeleteBranch,
        }))
    );
}

// ── Select routing — branch pick → DeleteBranch message ───────────────────────

#[test]
fn test_select_branch_routes_to_delete_branch_message() {
    let test_repo = TestRepo::new();
    test_repo.create_branch("feature");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Delete branch".to_string(),
            vec!["feature".to_string()],
            OnSelect::DeleteBranch,
        ),
    )));

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(result, Some(Message::DeleteBranch("feature".to_string())));
}

// ── Confirmation popup ────────────────────────────────────────────────────────

#[test]
fn test_delete_branch_shows_confirmation_popup() {
    let test_repo = TestRepo::new();
    test_repo.create_branch("feature");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::DeleteBranch("feature".to_string()));

    assert_eq!(result, None);
    let state = expect_confirm_popup(&model);
    assert!(
        state.message.contains("feature"),
        "Confirm message should mention the branch, got: {}",
        state.message
    );
    assert_eq!(
        state.on_confirm,
        ConfirmAction::DeleteBranch("feature".to_string())
    );
}

#[test]
fn test_y_in_confirm_popup_routes_to_confirm_delete_branch() {
    let test_repo = TestRepo::new();
    test_repo.create_branch("feature");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: "Are you sure you want to delete 'feature' (y/n)?".to_string(),
        on_confirm: ConfirmAction::DeleteBranch("feature".to_string()),
    }));

    let result = handle_key(key(KeyCode::Char('y')), &model);
    assert_eq!(
        result,
        Some(Message::ConfirmDeleteBranch("feature".to_string()))
    );
}

#[test]
fn test_n_in_confirm_popup_dismisses_popup() {
    let test_repo = TestRepo::new();
    test_repo.create_branch("feature");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: "Are you sure you want to delete 'feature' (y/n)?".to_string(),
        on_confirm: ConfirmAction::DeleteBranch("feature".to_string()),
    }));

    let result = handle_key(key(KeyCode::Char('n')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── Execution ─────────────────────────────────────────────────────────────────

#[test]
fn test_confirm_delete_branch_removes_branch_ref() {
    let test_repo = TestRepo::new();
    test_repo.create_branch("feature");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::ConfirmDeleteBranch("feature".to_string()),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);
    assert!(
        test_repo
            .repo
            .find_branch("feature", git2::BranchType::Local)
            .is_err(),
        "Branch 'feature' should be deleted"
    );
}

#[test]
fn test_confirm_delete_current_branch_detaches_head_and_deletes() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);

    // 'main' is the currently checked-out branch; the handler detaches HEAD
    // before force-deleting it rather than reporting an error.
    let result = update(&mut model, Message::ConfirmDeleteBranch("main".to_string()));

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);
    assert!(
        test_repo
            .repo
            .find_branch("main", git2::BranchType::Local)
            .is_err(),
        "Branch 'main' should be deleted"
    );
    assert!(
        test_repo.repo.head_detached().unwrap(),
        "HEAD should be detached after deleting the checked-out branch"
    );
}

#[test]
fn test_confirm_delete_unmerged_branch_force_deletes() {
    let test_repo = TestRepo::new();
    let workdir = test_repo.repo.workdir().unwrap().to_path_buf();

    // Create a branch with a commit that is not merged into main
    git(&workdir, &["checkout", "-b", "unmerged"]);
    test_repo.commit_file("unmerged.txt", "unmerged content", "Unmerged commit");
    git(&workdir, &["checkout", "main"]);

    let mut model = create_model_from_test_repo(&test_repo);

    // The handler deletes with `git branch -D`, so unmerged branches are
    // force-deleted without any warning.
    let result = update(
        &mut model,
        Message::ConfirmDeleteBranch("unmerged".to_string()),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);
    assert!(
        test_repo
            .repo
            .find_branch("unmerged", git2::BranchType::Local)
            .is_err(),
        "Unmerged branch should be force-deleted"
    );
}

#[test]
fn test_confirm_delete_nonexistent_branch_shows_error_popup() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::ConfirmDeleteBranch("does-not-exist".to_string()),
    );

    assert_eq!(result, None);
    expect_error_popup(&model);
}
