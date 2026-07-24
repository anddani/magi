use crossterm::event::KeyCode;
use magi::model::EditOp;
use magi::{
    git::{GitInfo, test_repo::TestRepo, worktree::worktree_add},
    keys::handle_key,
    model::popup::{InputContext, PopupContent, PopupContentCommand},
    msg::{
        InputMessage, Message, OnSelect, OptionsSource, SelectMessage, ShowSelectPopupConfig,
        update::update,
    },
};

mod utils;
use utils::{
    create_model_from_test_repo, expect_error_popup, expect_input_popup, expect_select_popup, key,
};

fn type_text(model: &mut magi::model::Model, text: &str) {
    for c in text.chars() {
        update(model, Message::Input(InputMessage::Edit(EditOp::Insert(c))));
    }
}

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

// ── Key binding: 'm' in worktree popup shows the worktree select ──────────────

#[test]
fn test_m_in_worktree_popup_shows_worktree_select() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Worktree));

    let result = handle_key(key(KeyCode::Char('m')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Move worktree".to_string(),
            source: OptionsSource::LinkedWorktrees,
            on_select: OnSelect::WorktreeMove,
        }))
    );
}

// ── The select popup lists linked worktrees (never the main one) ──────────────

#[test]
fn test_select_popup_lists_linked_worktrees_only() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    let worktree_path = add_linked_worktree(&test_repo, "feature");

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Move worktree".to_string(),
            source: OptionsSource::LinkedWorktrees,
            on_select: OnSelect::WorktreeMove,
        }),
    );

    let state = expect_select_popup(&model);
    assert_eq!(state.all_options.len(), 1);
    assert_eq!(
        std::path::Path::new(&state.all_options[0])
            .canonicalize()
            .unwrap(),
        std::path::Path::new(&worktree_path).canonicalize().unwrap()
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
            title: "Move worktree".to_string(),
            source: OptionsSource::LinkedWorktrees,
            on_select: OnSelect::WorktreeMove,
        }),
    );

    assert_eq!(expect_error_popup(&model), "No linked worktrees found");
}

// ── Selecting a worktree asks for the new path ────────────────────────────────

#[test]
fn test_select_worktree_dispatches_move_path_input() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    add_linked_worktree(&test_repo, "feature");

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Move worktree".to_string(),
            source: OptionsSource::LinkedWorktrees,
            on_select: OnSelect::WorktreeMove,
        }),
    );

    let selected = expect_select_popup(&model)
        .selected_item()
        .unwrap()
        .to_string();
    let result = update(&mut model, Message::Select(SelectMessage::Confirm));
    assert_eq!(
        result,
        Some(Message::ShowWorktreeMovePathInput { worktree: selected })
    );
}

// ── ShowWorktreeMovePathInput shows the input popup ───────────────────────────

#[test]
fn test_show_move_path_input_sets_input_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::ShowWorktreeMovePathInput {
            worktree: "/tmp/feature-wt".to_string(),
        },
    );

    assert_eq!(result, None);
    let state = expect_input_popup(&model);
    assert_eq!(
        state.context,
        InputContext::WorktreeMovePath {
            worktree: "/tmp/feature-wt".to_string(),
        }
    );
    assert!(
        state.title().contains("/tmp/feature-wt"),
        "Title should contain the worktree path, got: {}",
        state.title()
    );
}

// ── Confirming the path dispatches WorktreeMove ───────────────────────────────

#[test]
fn test_confirm_path_dispatches_worktree_move_message() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::ShowWorktreeMovePathInput {
            worktree: "/tmp/feature-wt".to_string(),
        },
    );

    type_text(&mut model, "../moved-wt");
    let result = update(&mut model, Message::Input(InputMessage::Confirm));
    assert_eq!(
        result,
        Some(Message::WorktreeMove {
            worktree: "/tmp/feature-wt".to_string(),
            path: "../moved-wt".to_string(),
        })
    );
}

// ── WorktreeMove execution ────────────────────────────────────────────────────

#[test]
fn test_worktree_move_moves_worktree_on_disk() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    let old_path = add_linked_worktree(&test_repo, "feature");

    let new_path = {
        let tmp = tempfile::tempdir().unwrap();
        tmp.path().to_str().unwrap().to_string()
    };

    let mut model = create_model_from_test_repo(&test_repo);
    let workdir_before = model.workdir.clone();
    let result = update(
        &mut model,
        Message::WorktreeMove {
            worktree: old_path.clone(),
            path: new_path.clone(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(!std::path::Path::new(&old_path).exists());
    assert!(std::path::Path::new(&new_path).exists());

    // The moved worktree still has its branch checked out
    let worktree_repo = git2::Repository::open(&new_path).unwrap();
    assert_eq!(
        worktree_repo.head().unwrap().shorthand().unwrap(),
        "feature"
    );

    // We were in the main worktree, so the model stays put
    assert_eq!(model.workdir, workdir_before);
}

#[test]
fn test_worktree_move_current_worktree_follows_to_new_path() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    let old_path = add_linked_worktree(&test_repo, "feature");

    let new_path = {
        let tmp = tempfile::tempdir().unwrap();
        tmp.path().to_str().unwrap().to_string()
    };

    // Model lives in the linked worktree that is about to be moved
    let mut model = create_model_from_test_repo(&test_repo);
    model.git_info = GitInfo::new_from_path(&old_path).unwrap();
    model.workdir = std::path::PathBuf::from(&old_path);

    let result = update(
        &mut model,
        Message::WorktreeMove {
            worktree: old_path.clone(),
            path: new_path.clone(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(
        model.workdir.canonicalize().unwrap(),
        std::path::Path::new(&new_path).canonicalize().unwrap()
    );
}

#[test]
fn test_worktree_move_main_worktree_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let new_path = {
        let tmp = tempfile::tempdir().unwrap();
        tmp.path().to_str().unwrap().to_string()
    };

    let mut model = create_model_from_test_repo(&test_repo);
    let workdir_before = model.workdir.clone();
    let main_path = workdir_before.to_str().unwrap().to_string();
    let result = update(
        &mut model,
        Message::WorktreeMove {
            worktree: main_path,
            path: new_path,
        },
    );

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup when moving the main worktree"
    );
    assert_eq!(model.workdir, workdir_before, "workdir should not change");
}

#[test]
fn test_worktree_move_nonexistent_worktree_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::WorktreeMove {
            worktree: "/nonexistent/worktree".to_string(),
            path: "/tmp/nowhere".to_string(),
        },
    );

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup for nonexistent worktree"
    );
}
