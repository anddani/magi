use crossterm::event::KeyCode;
use magi::{
    git::{GitInfo, test_repo::TestRepo, worktree::worktree_add},
    keys::handle_key,
    model::popup::{ConfirmAction, ConfirmPopupState, PopupContent, PopupContentCommand},
    msg::{Message, OnSelect, OptionsSource, SelectMessage, ShowSelectPopupConfig, update::update},
};

mod utils;
use utils::{
    assert_no_popup, create_model_from_test_repo, expect_confirm_popup, expect_error_popup,
    expect_select_popup, key,
};

/// Creates a linked worktree for `branch` and returns its path.
fn add_linked_worktree(test_repo: &TestRepo, branch: &str) -> String {
    test_repo.create_branch(branch);
    let path = {
        let tmp = tempfile::tempdir().unwrap();
        tmp.path().to_str().unwrap().to_string()
    };
    worktree_add(test_repo.repo_path(), &path, branch).unwrap();
    path
}

// ── Key binding: 'k' in worktree popup shows the worktree select ──────────────

#[test]
fn test_k_in_worktree_popup_shows_worktree_select() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Worktree));

    let result = handle_key(key(KeyCode::Char('k')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Delete worktree".to_string(),
            source: OptionsSource::LinkedWorktrees,
            on_select: OnSelect::WorktreeDelete,
        }))
    );
}

// ── Without linked worktrees the popup shows an error ─────────────────────────

#[test]
fn test_no_linked_worktrees_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Delete worktree".to_string(),
            source: OptionsSource::LinkedWorktrees,
            on_select: OnSelect::WorktreeDelete,
        }),
    );

    assert_eq!(expect_error_popup(&model), "No linked worktrees found");
}

// ── Selecting a worktree asks for confirmation ────────────────────────────────

#[test]
fn test_select_worktree_shows_confirm_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    add_linked_worktree(&test_repo, "feature");

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Delete worktree".to_string(),
            source: OptionsSource::LinkedWorktrees,
            on_select: OnSelect::WorktreeDelete,
        }),
    );

    let selected = expect_select_popup(&model)
        .selected_item()
        .unwrap()
        .to_string();
    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(result, None);
    let state = expect_confirm_popup(&model);
    assert_eq!(state.message, format!("Delete worktree \"{selected}\"?"));
    assert_eq!(state.on_confirm, ConfirmAction::DeleteWorktree(selected));
}

#[test]
fn test_select_dirty_worktree_warns_about_uncommitted_changes() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    let worktree_path = add_linked_worktree(&test_repo, "feature");
    std::fs::write(
        std::path::Path::new(&worktree_path).join("dirty.txt"),
        "uncommitted",
    )
    .unwrap();

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Delete worktree".to_string(),
            source: OptionsSource::LinkedWorktrees,
            on_select: OnSelect::WorktreeDelete,
        }),
    );
    update(&mut model, Message::Select(SelectMessage::Confirm));

    let state = expect_confirm_popup(&model);
    assert!(
        state.message.contains("despite uncommitted changes"),
        "Expected warning about uncommitted changes, got: {}",
        state.message
    );
}

// ── Confirming dispatches ConfirmDeleteWorktree ───────────────────────────────

#[test]
fn test_confirm_key_dispatches_confirm_delete_worktree() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: "Delete worktree \"/tmp/feature-wt\"?".to_string(),
        on_confirm: ConfirmAction::DeleteWorktree("/tmp/feature-wt".to_string()),
    }));

    let result = handle_key(key(KeyCode::Char('y')), &model);
    assert_eq!(
        result,
        Some(Message::ConfirmDeleteWorktree(
            "/tmp/feature-wt".to_string()
        ))
    );

    // 'n' dismisses instead
    let result = handle_key(key(KeyCode::Char('n')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── ConfirmDeleteWorktree execution ───────────────────────────────────────────

#[test]
fn test_confirm_dismisses_popup_after_successful_delete() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    let worktree_path = add_linked_worktree(&test_repo, "feature");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: format!("Delete worktree \"{worktree_path}\"?"),
        on_confirm: ConfirmAction::DeleteWorktree(worktree_path.clone()),
    }));

    // Full flow: Enter on the confirm popup dispatches the delete message,
    // which must dismiss the popup on success.
    let msg = handle_key(key(KeyCode::Enter), &model).unwrap();
    let result = update(&mut model, msg);

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);
    assert!(!std::path::Path::new(&worktree_path).exists());
}

#[test]
fn test_delete_worktree_removes_worktree_on_disk() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    let worktree_path = add_linked_worktree(&test_repo, "feature");

    let mut model = create_model_from_test_repo(&test_repo);
    let workdir_before = model.workdir.clone();
    let result = update(
        &mut model,
        Message::ConfirmDeleteWorktree(worktree_path.clone()),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(!std::path::Path::new(&worktree_path).exists());
    assert!(
        magi::git::worktree::list_linked_worktrees(&workdir_before).is_empty(),
        "The worktree should no longer be registered"
    );

    // We were in the main worktree, so the model stays put
    assert_eq!(model.workdir, workdir_before);
}

#[test]
fn test_delete_dirty_worktree_removes_worktree_on_disk() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    let worktree_path = add_linked_worktree(&test_repo, "feature");
    std::fs::write(
        std::path::Path::new(&worktree_path).join("dirty.txt"),
        "uncommitted",
    )
    .unwrap();

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::ConfirmDeleteWorktree(worktree_path.clone()),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(!std::path::Path::new(&worktree_path).exists());
}

#[test]
fn test_delete_current_worktree_falls_back_to_main() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    let worktree_path = add_linked_worktree(&test_repo, "feature");

    // Model lives in the linked worktree that is about to be deleted
    let mut model = create_model_from_test_repo(&test_repo);
    let main_workdir = model.workdir.clone();
    model.git_info = GitInfo::new_from_path(&worktree_path).unwrap();
    model.workdir = std::path::PathBuf::from(&worktree_path);

    let result = update(
        &mut model,
        Message::ConfirmDeleteWorktree(worktree_path.clone()),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(!std::path::Path::new(&worktree_path).exists());
    assert_eq!(
        model.workdir.canonicalize().unwrap(),
        main_workdir.canonicalize().unwrap()
    );
}

#[test]
fn test_delete_main_worktree_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let workdir_before = model.workdir.clone();
    let main_path = workdir_before.to_str().unwrap().to_string();
    let result = update(&mut model, Message::ConfirmDeleteWorktree(main_path));

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup when deleting the main worktree"
    );
    assert!(workdir_before.exists(), "Main worktree must survive");
}

#[test]
fn test_delete_nonexistent_worktree_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::ConfirmDeleteWorktree("/nonexistent/worktree".to_string()),
    );

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup for nonexistent worktree"
    );
}
