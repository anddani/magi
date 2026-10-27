use crossterm::event::KeyCode;
use magi::{
    git::{GitInfo, test_repo::TestRepo, worktree::worktree_add},
    keys::handle_key,
    model::{
        LineContent,
        popup::{PopupContent, PopupContentCommand},
    },
    msg::{Message, OnSelect, OptionsSource, SelectMessage, ShowSelectPopupConfig, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, expect_error_popup, expect_select_popup, find_line, key};

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

// ── Key binding: 'g' in worktree popup shows the worktree select ──────────────

#[test]
fn test_g_in_worktree_popup_shows_worktree_select() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Worktree));

    let result = handle_key(key(KeyCode::Char('g')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Visit worktree".to_string(),
            source: OptionsSource::OtherWorktrees,
            on_select: OnSelect::WorktreeVisit,
        }))
    );
}

// ── The select popup lists all worktrees except the current one ───────────────

#[test]
fn test_select_popup_lists_other_worktrees() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    let worktree_path = add_linked_worktree(&test_repo, "feature");

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Visit worktree".to_string(),
            source: OptionsSource::OtherWorktrees,
            on_select: OnSelect::WorktreeVisit,
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

// ── The worktree of the branch under the cursor is suggested first ────────────

#[test]
fn test_select_popup_prioritizes_worktree_of_cursor_branch() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    // Two linked worktrees on two branches, each pointing at a distinct commit.
    let feature_path = add_linked_worktree(&test_repo, "feature");
    test_repo.commit_file("file.txt", "second", "Second commit");
    let feature2_path = add_linked_worktree(&test_repo, "feature2");
    test_repo.commit_file("file.txt", "third", "Third commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Put the cursor on the commit labelled with "feature2".
    let pos = find_line(
        &model,
        |c| matches!(c, LineContent::Commit(info) if info.refs.iter().any(|r| r.name == "feature2")),
    )
    .expect("a commit line carrying the feature2 ref");
    model.ui_model.cursor_position = pos;

    update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Visit worktree".to_string(),
            source: OptionsSource::OtherWorktrees,
            on_select: OnSelect::WorktreeVisit,
        }),
    );

    let state = expect_select_popup(&model);
    // feature2's worktree is suggested first; feature's is still listed.
    assert_eq!(
        std::path::Path::new(&state.all_options[0])
            .canonicalize()
            .unwrap(),
        std::path::Path::new(&feature2_path).canonicalize().unwrap(),
        "the worktree of the branch under the cursor should be first"
    );
    assert!(
        state
            .all_options
            .iter()
            .any(|o| std::path::Path::new(o).canonicalize().unwrap()
                == std::path::Path::new(&feature_path).canonicalize().unwrap()),
        "the other worktree should still be present"
    );
}

// ── From a linked worktree the main working tree is a valid target ────────────

#[test]
fn test_select_popup_includes_main_worktree_when_in_linked() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    let worktree_path = add_linked_worktree(&test_repo, "feature");

    // Model lives in the linked worktree
    let mut model = create_model_from_test_repo(&test_repo);
    model.git_info = GitInfo::new_from_path(&worktree_path).unwrap();
    model.workdir = std::path::PathBuf::from(&worktree_path);

    update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Visit worktree".to_string(),
            source: OptionsSource::OtherWorktrees,
            on_select: OnSelect::WorktreeVisit,
        }),
    );

    let state = expect_select_popup(&model);
    assert_eq!(state.all_options.len(), 1);
    assert_eq!(
        std::path::Path::new(&state.all_options[0])
            .canonicalize()
            .unwrap(),
        test_repo.repo_path().canonicalize().unwrap()
    );
}

// ── Without other worktrees the popup shows an error ──────────────────────────

#[test]
fn test_no_other_worktrees_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Visit worktree".to_string(),
            source: OptionsSource::OtherWorktrees,
            on_select: OnSelect::WorktreeVisit,
        }),
    );

    assert_eq!(expect_error_popup(&model), "No other worktrees found");
}

// ── Selecting a worktree dispatches VisitWorktree ─────────────────────────────

#[test]
fn test_select_worktree_dispatches_visit_worktree() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    add_linked_worktree(&test_repo, "feature");

    let mut model = create_model_from_test_repo(&test_repo);
    update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Visit worktree".to_string(),
            source: OptionsSource::OtherWorktrees,
            on_select: OnSelect::WorktreeVisit,
        }),
    );

    let selected = expect_select_popup(&model)
        .selected_item()
        .unwrap()
        .to_string();
    let result = update(&mut model, Message::Select(SelectMessage::Confirm));
    assert_eq!(result, Some(Message::VisitWorktree(selected)));
}

// ── VisitWorktree execution ───────────────────────────────────────────────────

#[test]
fn test_visit_worktree_switches_workdir() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    let worktree_path = add_linked_worktree(&test_repo, "feature");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(&mut model, Message::VisitWorktree(worktree_path.clone()));

    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(
        model.workdir.canonicalize().unwrap(),
        std::path::Path::new(&worktree_path).canonicalize().unwrap()
    );
    // The visited worktree's branch is now the current one
    assert_eq!(model.git_info.current_branch().as_deref(), Some("feature"));
}

#[test]
fn test_visit_main_worktree_from_linked() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    let worktree_path = add_linked_worktree(&test_repo, "feature");

    // Model lives in the linked worktree
    let mut model = create_model_from_test_repo(&test_repo);
    model.git_info = GitInfo::new_from_path(&worktree_path).unwrap();
    model.workdir = std::path::PathBuf::from(&worktree_path);

    let main_path = test_repo.repo_path().to_str().unwrap().to_string();
    let result = update(&mut model, Message::VisitWorktree(main_path));

    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(
        model.workdir.canonicalize().unwrap(),
        test_repo.repo_path().canonicalize().unwrap()
    );
    assert_eq!(model.git_info.current_branch().as_deref(), Some("main"));
}

#[test]
fn test_visit_nonexistent_worktree_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let workdir_before = model.workdir.clone();
    let result = update(
        &mut model,
        Message::VisitWorktree("/nonexistent/worktree".to_string()),
    );

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup for nonexistent worktree"
    );
    assert_eq!(model.workdir, workdir_before, "workdir should not change");
}
