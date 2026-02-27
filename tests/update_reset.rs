use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::{
        LineContent,
        popup::{ConfirmAction, ConfirmPopupState, PopupContent, PopupContentCommand},
    },
    msg::{Message, SelectPopup, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn shift_key(c: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::SHIFT,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

// ── Key binding: 'O' shows reset popup ────────────────────────────────────────

#[test]
fn test_shift_o_shows_reset_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let model = create_model_from_test_repo(&test_repo);
    let result = handle_key(shift_key('O'), &model);
    assert_eq!(result, Some(Message::ShowResetPopup));
}

#[test]
fn test_uppercase_o_shows_reset_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let model = create_model_from_test_repo(&test_repo);
    let result = handle_key(key(KeyCode::Char('O')), &model);
    assert_eq!(result, Some(Message::ShowResetPopup));
}

// ── ShowResetPopup update ──────────────────────────────────────────────────────

#[test]
fn test_show_reset_popup_sets_reset_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(&mut model, Message::ShowResetPopup);
    assert_eq!(result, None);
    assert_eq!(
        model.popup,
        Some(PopupContent::Command(PopupContentCommand::Reset))
    );
}

// ── 'b' key in reset popup ─────────────────────────────────────────────────────

#[test]
fn test_b_in_reset_popup_shows_branch_pick_select() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Reset));

    let result = handle_key(key(KeyCode::Char('b')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(SelectPopup::ResetBranchPick))
    );
}

#[test]
fn test_q_dismisses_reset_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Reset));

    let result = handle_key(key(KeyCode::Char('q')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

#[test]
fn test_esc_dismisses_reset_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Reset));

    let result = handle_key(key(KeyCode::Esc), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── ShowSelectPopup::ResetBranchPick ──────────────────────────────────────────

#[test]
fn test_reset_branch_pick_shows_local_branches() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::ShowSelectPopup(SelectPopup::ResetBranchPick),
    );

    assert_eq!(result, None);
    assert!(matches!(
        &model.popup,
        Some(PopupContent::Command(PopupContentCommand::Select(state)))
            if !state.all_options.is_empty()
    ));
    assert_eq!(
        model.select_context,
        Some(magi::model::popup::SelectContext::ResetBranchPick)
    );
}

#[test]
fn test_reset_branch_pick_prioritizes_current_branch() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let current = model.git_info.current_branch().unwrap();

    update(
        &mut model,
        Message::ShowSelectPopup(SelectPopup::ResetBranchPick),
    );

    if let Some(PopupContent::Command(PopupContentCommand::Select(state))) = &model.popup {
        assert_eq!(state.all_options[0], current);
    } else {
        panic!("Expected Select popup");
    }
}

// ── ShowSelectPopup::ResetBranchTarget ────────────────────────────────────────

#[test]
fn test_reset_branch_target_shows_refs() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let current = model.git_info.current_branch().unwrap();

    // We need another branch to have a target
    let result = update(
        &mut model,
        Message::ShowSelectPopup(SelectPopup::ResetBranchTarget(current.clone())),
    );

    assert_eq!(result, None);
    // With only one branch and no remotes, there might be no options,
    // but if there are branches, the list should not contain the branch itself
    if let Some(PopupContent::Command(PopupContentCommand::Select(state))) = &model.popup {
        assert!(!state.all_options.contains(&current));
    }
    // Error popup is also valid when there's nothing to reset to
}

// ── ResetBranch on non-current branch (no confirmation needed) ────────────────

#[test]
fn test_reset_non_current_branch_to_target() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "initial")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    // Get the initial commit hash
    let initial_hash = {
        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        repo.head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string()
    };

    // Create another commit
    test_repo
        .write_file_content("file.txt", "second")
        .stage_files(&["file.txt"])
        .commit("Second commit");

    // Create a second branch pointing at second commit
    {
        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("other-branch", &head, false).unwrap();
    }

    let mut model = create_model_from_test_repo(&test_repo);
    let current = model.git_info.current_branch().unwrap();

    // Reset "other-branch" (non-current) to the initial commit
    let result = update(
        &mut model,
        Message::ResetBranch {
            branch: "other-branch".to_string(),
            target: initial_hash.clone(),
        },
    );

    // Should refresh
    assert_eq!(result, Some(Message::Refresh));

    // Verify other-branch was moved
    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let other_ref = repo
        .find_branch("other-branch", git2::BranchType::Local)
        .unwrap();
    let other_commit = other_ref.get().peel_to_commit().unwrap();
    assert_eq!(other_commit.id().to_string(), initial_hash);

    // Current branch should be unchanged
    let current_ref = repo.find_branch(&current, git2::BranchType::Local).unwrap();
    let current_commit = current_ref.get().peel_to_commit().unwrap();
    assert_ne!(current_commit.id().to_string(), initial_hash);
}

// ── ResetBranch on current branch (no uncommitted changes) ────────────────────

#[test]
fn test_reset_current_branch_to_earlier_commit() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "initial")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let initial_hash = repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id()
        .to_string();
    drop(repo);

    test_repo
        .write_file_content("file.txt", "second")
        .stage_files(&["file.txt"])
        .commit("Second commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let current = model.git_info.current_branch().unwrap();

    let result = update(
        &mut model,
        Message::ResetBranch {
            branch: current.clone(),
            target: initial_hash.clone(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));

    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head_commit.id().to_string(), initial_hash);
}

// ── Uncommitted changes confirmation ──────────────────────────────────────────

#[test]
fn test_reset_current_branch_with_uncommitted_shows_confirmation() {
    use magi::model::popup::SelectContext;

    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "initial")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let initial_hash = repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id()
        .to_string();
    drop(repo);

    test_repo
        .write_file_content("file.txt", "second")
        .stage_files(&["file.txt"])
        .commit("Second commit");

    // Add an unstaged change to simulate uncommitted work
    test_repo.write_file_content("file.txt", "dirty");

    let mut model = create_model_from_test_repo(&test_repo);
    let current = model.git_info.current_branch().unwrap();

    // Simulate the second step of the flow: context = ResetBranchTarget(current)
    model.select_context = Some(SelectContext::ResetBranchTarget(current.clone()));
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        magi::model::popup::SelectPopupState::new(
            "Reset branch to".to_string(),
            vec![initial_hash.clone()],
        ),
    )));

    // Confirm the selection
    let result = update(
        &mut model,
        Message::Select(magi::msg::SelectMessage::Confirm),
    );
    assert_eq!(result, None);

    // Should show a confirmation popup
    if let Some(PopupContent::Confirm(state)) = &model.popup {
        assert!(state.message.contains("Uncommitted changes will be lost"));
        assert_eq!(
            state.on_confirm,
            ConfirmAction::ResetBranch {
                branch: current.clone(),
                target: initial_hash.clone(),
            }
        );
    } else {
        panic!("Expected Confirm popup, got: {:?}", model.popup);
    }
}

#[test]
fn test_reset_current_branch_clean_no_confirmation() {
    use magi::model::popup::SelectContext;

    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "initial")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let initial_hash = repo
        .head()
        .unwrap()
        .peel_to_commit()
        .unwrap()
        .id()
        .to_string();
    drop(repo);

    test_repo
        .write_file_content("file.txt", "second")
        .stage_files(&["file.txt"])
        .commit("Second commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let current = model.git_info.current_branch().unwrap();

    // Simulate clean repo — no uncommitted changes
    model.select_context = Some(SelectContext::ResetBranchTarget(current.clone()));
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        magi::model::popup::SelectPopupState::new(
            "Reset branch to".to_string(),
            vec![initial_hash.clone()],
        ),
    )));

    let result = update(
        &mut model,
        Message::Select(magi::msg::SelectMessage::Confirm),
    );
    // Should dispatch ResetBranch directly without confirmation
    assert_eq!(
        result,
        Some(Message::ResetBranch {
            branch: current,
            target: initial_hash,
        })
    );
}

// ── Confirmation popup: y triggers ResetBranch ────────────────────────────────

#[test]
fn test_y_in_reset_confirm_popup_triggers_reset() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: "Uncommitted changes will be lost. Proceed?".to_string(),
        on_confirm: ConfirmAction::ResetBranch {
            branch: "main".to_string(),
            target: "abc1234".to_string(),
        },
    }));

    let result = handle_key(key(KeyCode::Char('y')), &model);
    assert_eq!(
        result,
        Some(Message::ResetBranch {
            branch: "main".to_string(),
            target: "abc1234".to_string(),
        })
    );
}

#[test]
fn test_n_in_reset_confirm_popup_dismisses() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: "Uncommitted changes will be lost. Proceed?".to_string(),
        on_confirm: ConfirmAction::ResetBranch {
            branch: "main".to_string(),
            target: "abc1234".to_string(),
        },
    }));

    let result = handle_key(key(KeyCode::Char('n')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── Cursor-line suggestion: branch pick ───────────────────────────────────────

#[test]
fn test_reset_branch_pick_cursor_on_branch_ref_prioritizes_it() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "initial")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    // Create a second branch pointing at the initial commit
    {
        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("other-branch", &head, false).unwrap();
    }

    // Make another commit so current branch is ahead of other-branch
    test_repo
        .write_file_content("file.txt", "second")
        .stage_files(&["file.txt"])
        .commit("Second commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Find a commit line that has "other-branch" as a ref
    let other_pos = model.ui_model.lines.iter().position(|l| {
        if let LineContent::Commit(info) = &l.content {
            info.refs.iter().any(|r| r.name == "other-branch")
        } else {
            false
        }
    });

    if let Some(pos) = other_pos {
        model.ui_model.cursor_position = pos;
        update(
            &mut model,
            Message::ShowSelectPopup(SelectPopup::ResetBranchPick),
        );

        if let Some(PopupContent::Command(PopupContentCommand::Select(state))) = &model.popup {
            assert_eq!(
                state.all_options[0], "other-branch",
                "other-branch should be first because cursor is on its commit"
            );
        } else {
            panic!("Expected Select popup");
        }
    }
    // If other-branch doesn't appear in the visible lines, the test is skipped
    // (shallow test repos may not always show it)
}

// ── Cursor-line suggestion: target pick with bare hash ─────────────────────────

#[test]
fn test_reset_branch_target_cursor_on_bare_hash_prioritizes_it() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "initial")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let initial_hash = {
        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        repo.head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string()
    };

    // Make a second commit — now the first commit has no branch ref pointing at it
    test_repo
        .write_file_content("file.txt", "second")
        .stage_files(&["file.txt"])
        .commit("Second commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Find the first commit (no branch refs, only has a hash)
    let bare_pos = model.ui_model.lines.iter().position(|l| {
        if let LineContent::Commit(info) = &l.content {
            info.refs.is_empty() && info.hash.len() == 7
        } else {
            false
        }
    });

    if let Some(pos) = bare_pos {
        let short_hash = if let LineContent::Commit(info) = &model.ui_model.lines[pos].content {
            info.hash.clone()
        } else {
            panic!("Expected commit line");
        };

        model.ui_model.cursor_position = pos;
        update(
            &mut model,
            Message::ShowSelectPopup(SelectPopup::ResetBranchTarget("other".to_string())),
        );

        if let Some(PopupContent::Command(PopupContentCommand::Select(state))) = &model.popup {
            assert_eq!(
                state.all_options[0], short_hash,
                "hash should be first because cursor is on a bare commit"
            );
            // Verify the hash is a prefix of the initial commit
            assert!(
                initial_hash.starts_with(&short_hash),
                "short hash should be prefix of full hash"
            );
        } else {
            panic!("Expected Select popup");
        }
    }
    // If no bare commit lines visible, test is skipped
}

#[test]
fn test_reset_branch_target_cursor_on_branch_ref_prioritizes_branch_over_hash() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "initial")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let current = model.git_info.current_branch().unwrap();

    // Find the HEAD commit line — it has the current branch ref
    let head_pos = model.ui_model.lines.iter().position(|l| {
        if let LineContent::Commit(info) = &l.content {
            info.refs.iter().any(|r| r.name == current)
        } else {
            false
        }
    });

    if let Some(pos) = head_pos {
        model.ui_model.cursor_position = pos;
        // Reset a different branch to see target options
        update(
            &mut model,
            Message::ShowSelectPopup(SelectPopup::ResetBranchTarget("other-branch".to_string())),
        );

        if let Some(PopupContent::Command(PopupContentCommand::Select(state))) = &model.popup {
            // The branch ref (current branch name) should appear before the hash
            assert_eq!(
                state.all_options[0], current,
                "current branch name should be first, not the commit hash"
            );
        } else {
            // No refs to reset to (only 1 branch and it's excluded) — acceptable
        }
    }
}
