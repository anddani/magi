use crossterm::event::KeyCode;
use magi::{
    git::{git_cmd, test_repo::TestRepo},
    keys::handle_key,
    model::{
        LineContent, ToastStyle, ViewMode,
        popup::{MergePopupState, PopupContent, PopupContentCommand},
        select_popup::OnSelect,
    },
    msg::{MergeCommand, Message, OptionsSource, ShowSelectPopupConfig, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, expect_error_popup, expect_select_popup, find_line, key};

// ── ShowMergePopup — key binding ───────────────────────────────────────────────

#[test]
fn test_m_key_shows_merge_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let model = create_model_from_test_repo(&test_repo);

    let result = handle_key(key(KeyCode::Char('m')), &model);
    assert_eq!(result, Some(Message::ShowMergePopup));
}

// ── ShowMergePopup — state ─────────────────────────────────────────────────────

#[test]
fn test_show_merge_popup_sets_state() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowMergePopup);

    assert_eq!(result, None);
    if let Some(PopupContent::Command(PopupContentCommand::Merge(state))) = &model.popup {
        assert!(!state.in_progress);
    } else {
        panic!("Expected Merge popup");
    }
}

// ── Merge popup keys — normal mode ────────────────────────────────────────────

#[test]
fn test_m_in_merge_popup_shows_select() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: false },
    )));

    let result = handle_key(key(KeyCode::Char('m')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Merge branch".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::MergeElsewhere,
        }))
    );
}

#[test]
fn test_e_in_merge_popup_shows_select_with_edit_message() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: false },
    )));

    let result = handle_key(key(KeyCode::Char('e')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Merge branch (edit message)".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::MergeEditMessage,
        }))
    );
}

#[test]
fn test_n_in_merge_popup_shows_select_with_no_commit() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: false },
    )));

    let result = handle_key(key(KeyCode::Char('n')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Merge branch (no commit)".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::MergeNoCommit,
        }))
    );
}

#[test]
fn test_a_in_merge_popup_shows_select_with_absorb() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: false },
    )));

    let result = handle_key(key(KeyCode::Char('a')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Absorb branch".to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::MergeAbsorb,
        }))
    );
}

#[test]
fn test_p_in_merge_popup_shows_select_with_preview() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: false },
    )));

    let result = handle_key(key(KeyCode::Char('p')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Preview merge".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::MergePreview,
        }))
    );
}

#[test]
fn test_s_in_merge_popup_shows_select_with_squash() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: false },
    )));

    let result = handle_key(key(KeyCode::Char('s')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Squash merge".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::MergeSquash,
        }))
    );
}

#[test]
fn test_q_dismisses_merge_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: false },
    )));

    let result = handle_key(key(KeyCode::Char('q')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

#[test]
fn test_esc_dismisses_merge_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: false },
    )));

    let result = handle_key(key(KeyCode::Esc), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── Merge popup keys — in_progress mode ──────────────────────────────────────

#[test]
fn test_m_in_merge_popup_in_progress_continues() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: true },
    )));

    let result = handle_key(key(KeyCode::Char('m')), &model);
    assert_eq!(result, Some(Message::Merge(MergeCommand::Continue)));
}

#[test]
fn test_a_in_merge_popup_in_progress_aborts() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: true },
    )));

    let result = handle_key(key(KeyCode::Char('a')), &model);
    assert_eq!(result, Some(Message::Merge(MergeCommand::Abort)));
}

#[test]
fn test_q_dismisses_merge_popup_in_progress() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: true },
    )));

    let result = handle_key(key(KeyCode::Char('q')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── Cursor-line suggestion for MergeElsewhere ─────────────────────────────────

#[test]
fn test_merge_elsewhere_cursor_on_branch_ref_prioritizes_it() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "initial", "Initial commit");

    // Create another branch pointing at this commit
    test_repo.create_branch("feature-branch");

    // Make a second commit so current branch is ahead
    test_repo.commit_file("file.txt", "second", "Second commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Find a commit line that has "feature-branch" as a ref
    let branch_pos = find_line(
        &model,
        |c| matches!(c, LineContent::Commit(info) if info.refs.iter().any(|r| r.name == "feature-branch")),
    );

    if let Some(pos) = branch_pos {
        model.ui_model.cursor_position = pos;
        update(
            &mut model,
            Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: "Merge branch".to_string(),
                source: OptionsSource::LocalAndRemoteBranches,
                on_select: OnSelect::MergeElsewhere,
            }),
        );

        let state = expect_select_popup(&model);
        assert_eq!(
            state.all_options[0], "feature-branch",
            "feature-branch should be first because cursor is on its commit"
        );
    }
}

#[test]
fn test_merge_preview_cursor_on_branch_ref_prioritizes_it() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "initial", "Initial commit");

    // Create another branch pointing at this commit
    test_repo.create_branch("feature-branch");

    // Make a second commit so current branch is ahead
    test_repo.commit_file("file.txt", "second", "Second commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Find a commit line that has "feature-branch" as a ref
    let branch_pos = find_line(
        &model,
        |c| matches!(c, LineContent::Commit(info) if info.refs.iter().any(|r| r.name == "feature-branch")),
    );

    if let Some(pos) = branch_pos {
        model.ui_model.cursor_position = pos;
        update(
            &mut model,
            Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: "Preview merge".to_string(),
                source: OptionsSource::LocalAndRemoteBranches,
                on_select: OnSelect::MergePreview,
            }),
        );

        let state = expect_select_popup(&model);
        assert_eq!(
            state.all_options[0], "feature-branch",
            "feature-branch should be first because cursor is on its commit"
        );
    }
}

#[test]
fn test_merge_squash_cursor_on_branch_ref_prioritizes_it() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "initial", "Initial commit");

    // Create another branch pointing at this commit
    test_repo.create_branch("feature-branch");

    // Make a second commit so current branch is ahead
    test_repo.commit_file("file.txt", "second", "Second commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Find a commit line that has "feature-branch" as a ref
    let branch_pos = find_line(
        &model,
        |c| matches!(c, LineContent::Commit(info) if info.refs.iter().any(|r| r.name == "feature-branch")),
    );

    if let Some(pos) = branch_pos {
        model.ui_model.cursor_position = pos;
        update(
            &mut model,
            Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: "Squash merge".to_string(),
                source: OptionsSource::LocalAndRemoteBranches,
                on_select: OnSelect::MergeSquash,
            }),
        );

        let state = expect_select_popup(&model);
        assert_eq!(
            state.all_options[0], "feature-branch",
            "feature-branch should be first because cursor is on its commit"
        );
    }
}

#[test]
fn test_merge_elsewhere_no_cursor_suggestion_shows_normal_order() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "initial", "Initial commit");

    // Create another branch
    test_repo.create_branch("other-branch");

    let mut model = create_model_from_test_repo(&test_repo);

    // Position cursor on a non-branch line
    model.ui_model.cursor_position = 0;

    update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Merge branch".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::MergeElsewhere,
        }),
    );

    let state = expect_select_popup(&model);
    assert!(
        !state.all_options.is_empty(),
        "Should have branches to merge"
    );
    // other-branch should be present somewhere in the list
    assert!(
        state.all_options.contains(&"other-branch".to_string()),
        "other-branch should be in the options"
    );
}

// ── MergeCommand::Branch — execution ──────────────────────────────────────────

/// `git merge` opens an editor for the merge commit message.
/// Point core.editor at `true` so the test does not hang waiting for input.
fn disable_editor(test_repo: &TestRepo) {
    test_repo
        .repo
        .config()
        .unwrap()
        .set_str("core.editor", "true")
        .unwrap();
}

/// Creates `feature` diverging from `main`: both branches get one commit
/// after the shared base. Leaves the repo checked out on `main`.
fn setup_divergent_branches(
    test_repo: &TestRepo,
    main_file: (&str, &str),
    feature_file: (&str, &str),
) {
    test_repo.commit_file("base.txt", "base\n", "Base commit");
    test_repo.create_branch("feature");
    test_repo.commit_file(main_file.0, main_file.1, "Main commit");
    let checkout = |branch: &str| {
        assert!(
            git_cmd(test_repo.repo_path(), &["checkout", branch])
                .output()
                .unwrap()
                .status
                .success()
        );
    };
    checkout("feature");
    test_repo.commit_file(feature_file.0, feature_file.1, "Feature commit");
    checkout("main");
}

#[test]
fn test_merge_branch_message_creates_merge_commit() {
    let test_repo = TestRepo::new();
    disable_editor(&test_repo);
    // Divergent branches touching different files: merges cleanly.
    setup_divergent_branches(
        &test_repo,
        ("main.txt", "main content\n"),
        ("feature.txt", "feature content\n"),
    );

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Merge(MergeCommand::Branch("feature".to_string())),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.popup.is_none());
    // The merge runs directly (TUI suspended), not in a background PTY.
    assert!(model.pty_state.is_none());
    let toast = model.toast.expect("Expected a toast after merging");
    assert_eq!(toast.style, ToastStyle::Success);
    assert!(
        toast.message.starts_with("Merge: Merge branch 'feature'"),
        "unexpected toast message: {}",
        toast.message
    );

    let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head.parent_count(), 2);
    assert!(test_repo.repo_path().join("main.txt").exists());
    assert!(test_repo.repo_path().join("feature.txt").exists());
}

// ── MergeCommand::EditMessage — execution ─────────────────────────────────────

#[test]
fn test_merge_edit_message_creates_merge_commit_even_when_fast_forward() {
    let test_repo = TestRepo::new();
    disable_editor(&test_repo);
    test_repo.commit_file("base.txt", "base\n", "Base commit");

    // Put a commit on feature only, so a plain merge would fast-forward.
    assert!(
        git_cmd(test_repo.repo_path(), &["checkout", "-b", "feature"])
            .output()
            .unwrap()
            .status
            .success()
    );
    test_repo.commit_file("feature.txt", "feature content\n", "Feature commit");
    assert!(
        git_cmd(test_repo.repo_path(), &["checkout", "main"])
            .output()
            .unwrap()
            .status
            .success()
    );

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Merge(MergeCommand::EditMessage("feature".to_string())),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.popup.is_none());
    // The merge runs directly (TUI suspended), not in a background PTY.
    assert!(model.pty_state.is_none());
    let toast = model.toast.expect("Expected a toast after merging");
    assert_eq!(toast.style, ToastStyle::Success);
    assert!(
        toast.message.starts_with("Merge: Merge branch 'feature'"),
        "unexpected toast message: {}",
        toast.message
    );

    // --no-ff forces a merge commit even when fast-forward is possible.
    let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head.parent_count(), 2);
    assert!(test_repo.repo_path().join("feature.txt").exists());
}

#[test]
fn test_merge_edit_message_with_conflicts_shows_conflict_dialog() {
    let test_repo = TestRepo::new();
    disable_editor(&test_repo);
    // Both branches modify the same file: merging conflicts.
    setup_divergent_branches(
        &test_repo,
        ("base.txt", "main change\n"),
        ("base.txt", "feature change\n"),
    );

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Merge(MergeCommand::EditMessage("feature".to_string())),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.pty_state.is_none());
    // A toast is easy to miss: conflicts must surface as an error dialog.
    assert!(model.toast.is_none());
    let message = expect_error_popup(&model);
    assert!(
        message.contains("Merge of 'feature' stopped due to conflicts"),
        "unexpected error message: {}",
        message
    );
    assert!(
        message.contains("base.txt"),
        "conflicted file missing from message: {}",
        message
    );
    // The merge stays in progress so it can be resolved or aborted.
    assert!(test_repo.repo.path().join("MERGE_HEAD").exists());
}

#[test]
fn test_merge_branch_message_with_conflicts_shows_conflict_dialog() {
    let test_repo = TestRepo::new();
    disable_editor(&test_repo);
    // Both branches modify the same file: merging conflicts.
    setup_divergent_branches(
        &test_repo,
        ("base.txt", "main change\n"),
        ("base.txt", "feature change\n"),
    );

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Merge(MergeCommand::Branch("feature".to_string())),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.pty_state.is_none());
    // A toast is easy to miss: conflicts must surface as an error dialog.
    assert!(model.toast.is_none());
    let message = expect_error_popup(&model);
    assert!(
        message.contains("Merge of 'feature' stopped due to conflicts"),
        "unexpected error message: {}",
        message
    );
    assert!(
        message.contains("base.txt"),
        "conflicted file missing from message: {}",
        message
    );
    // The merge stays in progress so it can be resolved or aborted.
    assert!(test_repo.repo.path().join("MERGE_HEAD").exists());
}

#[test]
fn test_merge_continue_with_unresolved_conflicts_shows_conflict_dialog() {
    let test_repo = TestRepo::new();
    disable_editor(&test_repo);
    setup_divergent_branches(
        &test_repo,
        ("base.txt", "main change\n"),
        ("base.txt", "feature change\n"),
    );

    let mut model = create_model_from_test_repo(&test_repo);

    // Start the merge; it stops on the conflict.
    update(
        &mut model,
        Message::Merge(MergeCommand::Branch("feature".to_string())),
    );
    assert!(test_repo.repo.path().join("MERGE_HEAD").exists());

    // Continuing without resolving must explain what is blocking it.
    let result = update(&mut model, Message::Merge(MergeCommand::Continue));

    assert_eq!(result, Some(Message::Refresh));
    let message = expect_error_popup(&model);
    assert!(
        message.contains("Cannot continue the merge"),
        "unexpected error message: {}",
        message
    );
    assert!(
        message.contains("base.txt"),
        "conflicted file missing from message: {}",
        message
    );
    assert!(test_repo.repo.path().join("MERGE_HEAD").exists());
}

// ── MergeCommand::Absorb — execution ──────────────────────────────────────────

#[test]
fn test_absorb_creates_merge_commit_and_deletes_branch() {
    let test_repo = TestRepo::new();
    // Divergent branches touching different files: merges cleanly.
    setup_divergent_branches(
        &test_repo,
        ("main.txt", "main content\n"),
        ("feature.txt", "feature content\n"),
    );

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Merge(MergeCommand::Absorb("feature".to_string())),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.popup.is_none());
    // --no-edit never opens an editor, so the merge runs directly.
    assert!(model.pty_state.is_none());
    let toast = model.toast.expect("Expected a toast after absorbing");
    assert_eq!(toast.style, ToastStyle::Success);
    assert!(
        toast.message.starts_with("Absorb: Merge branch 'feature'"),
        "unexpected toast message: {}",
        toast.message
    );

    let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head.parent_count(), 2);
    assert!(test_repo.repo_path().join("feature.txt").exists());
    // The absorbed branch is gone.
    assert!(
        test_repo
            .repo
            .find_branch("feature", git2::BranchType::Local)
            .is_err()
    );
}

#[test]
fn test_absorb_with_conflicts_shows_conflict_dialog_and_keeps_branch() {
    let test_repo = TestRepo::new();
    // Both branches modify the same file: merging conflicts.
    setup_divergent_branches(
        &test_repo,
        ("base.txt", "main change\n"),
        ("base.txt", "feature change\n"),
    );

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Merge(MergeCommand::Absorb("feature".to_string())),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.pty_state.is_none());
    // A toast is easy to miss: conflicts must surface as an error dialog.
    assert!(model.toast.is_none());
    let message = expect_error_popup(&model);
    assert!(
        message.contains("Absorb of 'feature' stopped due to conflicts"),
        "unexpected error message: {}",
        message
    );
    assert!(
        message.contains("base.txt"),
        "conflicted file missing from message: {}",
        message
    );
    // The merge stays in progress and the branch survives so the merge can
    // be resolved or aborted.
    assert!(test_repo.repo.path().join("MERGE_HEAD").exists());
    assert!(
        test_repo
            .repo
            .find_branch("feature", git2::BranchType::Local)
            .is_ok()
    );
}

// ── MergeCommand::NoCommit — execution ────────────────────────────────────────

/// Waits for the background PTY merge to finish by polling the repo state.
fn wait_for<F: Fn() -> bool>(condition: F, description: &str) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if condition() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    panic!("Timed out waiting for: {}", description);
}

#[test]
fn test_merge_no_commit_runs_in_pty() {
    let test_repo = TestRepo::new();
    // Divergent branches touching different files: merges cleanly.
    setup_divergent_branches(
        &test_repo,
        ("main.txt", "main content\n"),
        ("feature.txt", "feature content\n"),
    );

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Merge(MergeCommand::NoCommit("feature".to_string())),
    );

    // --no-commit never opens an editor, so the merge runs in a background
    // PTY instead of suspending the TUI.
    assert_eq!(result, None);
    assert!(model.popup.is_none());
    assert!(model.pty_state.is_some());
}

#[test]
fn test_merge_no_commit_stages_merge_without_committing() {
    let test_repo = TestRepo::new();
    setup_divergent_branches(
        &test_repo,
        ("main.txt", "main content\n"),
        ("feature.txt", "feature content\n"),
    );
    let head_before = test_repo.head_hash();

    let mut model = create_model_from_test_repo(&test_repo);

    update(
        &mut model,
        Message::Merge(MergeCommand::NoCommit("feature".to_string())),
    );

    // The merge pauses before committing, leaving MERGE_HEAD in place.
    wait_for(
        || test_repo.repo.path().join("MERGE_HEAD").exists(),
        "MERGE_HEAD to appear",
    );
    assert_eq!(test_repo.head_hash(), head_before, "HEAD must not move");
    assert!(test_repo.repo_path().join("feature.txt").exists());
}

#[test]
fn test_merge_no_commit_fast_forward_still_stops_before_committing() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("base.txt", "base\n", "Base commit");

    // Put a commit on feature only, so a plain merge would fast-forward.
    assert!(
        git_cmd(test_repo.repo_path(), &["checkout", "-b", "feature"])
            .output()
            .unwrap()
            .status
            .success()
    );
    test_repo.commit_file("feature.txt", "feature content\n", "Feature commit");
    assert!(
        git_cmd(test_repo.repo_path(), &["checkout", "main"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let head_before = test_repo.head_hash();

    let mut model = create_model_from_test_repo(&test_repo);

    update(
        &mut model,
        Message::Merge(MergeCommand::NoCommit("feature".to_string())),
    );

    // --no-ff prevents the fast-forward, so the merge pauses uncommitted.
    wait_for(
        || test_repo.repo.path().join("MERGE_HEAD").exists(),
        "MERGE_HEAD to appear",
    );
    assert_eq!(test_repo.head_hash(), head_before, "HEAD must not move");
    assert!(test_repo.repo_path().join("feature.txt").exists());
}

// ── MergeCommand::Squash — execution ──────────────────────────────────────────

#[test]
fn test_squash_merge_runs_in_pty() {
    let test_repo = TestRepo::new();
    // Divergent branches touching different files: merges cleanly.
    setup_divergent_branches(
        &test_repo,
        ("main.txt", "main content\n"),
        ("feature.txt", "feature content\n"),
    );

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Merge(MergeCommand::Squash("feature".to_string())),
    );

    // --squash never opens an editor, so the merge runs in a background
    // PTY instead of suspending the TUI.
    assert_eq!(result, None);
    assert!(model.popup.is_none());
    assert!(model.pty_state.is_some());
}

#[test]
fn test_squash_merge_stages_changes_without_committing() {
    let test_repo = TestRepo::new();
    setup_divergent_branches(
        &test_repo,
        ("main.txt", "main content\n"),
        ("feature.txt", "feature content\n"),
    );
    let head_before = test_repo.head_hash();

    let mut model = create_model_from_test_repo(&test_repo);

    update(
        &mut model,
        Message::Merge(MergeCommand::Squash("feature".to_string())),
    );

    // --squash stages the merged changes and stops, leaving SQUASH_MSG in
    // place instead of creating a merge commit (no MERGE_HEAD either).
    wait_for(
        || test_repo.repo.path().join("SQUASH_MSG").exists(),
        "SQUASH_MSG to appear",
    );
    assert_eq!(test_repo.head_hash(), head_before, "HEAD must not move");
    assert!(test_repo.repo_path().join("feature.txt").exists());
    assert!(
        !test_repo.repo.path().join("MERGE_HEAD").exists(),
        "squash merge must not record a merge in progress"
    );
}

#[test]
fn test_squash_merge_fast_forward_still_stops_before_committing() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("base.txt", "base\n", "Base commit");

    // Put a commit on feature only, so a plain merge would fast-forward.
    assert!(
        git_cmd(test_repo.repo_path(), &["checkout", "-b", "feature"])
            .output()
            .unwrap()
            .status
            .success()
    );
    test_repo.commit_file("feature.txt", "feature content\n", "Feature commit");
    assert!(
        git_cmd(test_repo.repo_path(), &["checkout", "main"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let head_before = test_repo.head_hash();

    let mut model = create_model_from_test_repo(&test_repo);

    update(
        &mut model,
        Message::Merge(MergeCommand::Squash("feature".to_string())),
    );

    // Even a fast-forwardable squash merge stages the changes uncommitted.
    wait_for(
        || test_repo.repo_path().join("feature.txt").exists(),
        "feature.txt to appear in the worktree",
    );
    assert_eq!(test_repo.head_hash(), head_before, "HEAD must not move");
}

// ── MergeCommand::Preview — execution ─────────────────────────────────────────

#[test]
fn test_preview_merge_enters_preview_mode_with_diff() {
    let test_repo = TestRepo::new();
    setup_divergent_branches(
        &test_repo,
        ("main.txt", "main content\n"),
        ("feature.txt", "feature content\n"),
    );
    let head_before = test_repo.head_hash();

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Merge(MergeCommand::Preview("feature".to_string())),
    );

    assert_eq!(result, None);
    assert!(model.popup.is_none());
    assert_eq!(model.view_mode, ViewMode::Preview);
    assert_eq!(model.ui_model.cursor_position, 0);

    let contents: Vec<&str> = model
        .ui_model
        .lines
        .iter()
        .filter_map(|l| match &l.content {
            LineContent::PreviewLine { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(contents[0], "Preview merge of feature into main");
    assert!(
        contents.contains(&"+feature content"),
        "expected incoming change in preview: {:?}",
        contents
    );

    // Previewing must not perform the merge.
    assert_eq!(test_repo.head_hash(), head_before, "HEAD must not move");
    assert!(!test_repo.repo_path().join("feature.txt").exists());
    assert!(!test_repo.repo.path().join("MERGE_HEAD").exists());
}

#[test]
fn test_preview_merge_exit_returns_to_previous_view() {
    let test_repo = TestRepo::new();
    setup_divergent_branches(
        &test_repo,
        ("main.txt", "main content\n"),
        ("feature.txt", "feature content\n"),
    );

    let mut model = create_model_from_test_repo(&test_repo);
    let cursor_before = model.ui_model.cursor_position;

    update(
        &mut model,
        Message::Merge(MergeCommand::Preview("feature".to_string())),
    );
    assert_eq!(model.view_mode, ViewMode::Preview);

    update(&mut model, Message::ExitPreview);

    assert_eq!(model.view_mode, ViewMode::Status);
    assert_eq!(model.ui_model.cursor_position, cursor_before);
}

#[test]
fn test_preview_merge_with_conflicts_shows_conflict_note() {
    let test_repo = TestRepo::new();
    setup_divergent_branches(
        &test_repo,
        ("base.txt", "main change\n"),
        ("base.txt", "feature change\n"),
    );

    let mut model = create_model_from_test_repo(&test_repo);

    update(
        &mut model,
        Message::Merge(MergeCommand::Preview("feature".to_string())),
    );

    assert_eq!(model.view_mode, ViewMode::Preview);
    let contents: Vec<&str> = model
        .ui_model
        .lines
        .iter()
        .filter_map(|l| match &l.content {
            LineContent::PreviewLine { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        contents.iter().any(|c| c.contains("conflicts")),
        "expected a conflict note: {:?}",
        contents
    );
    // Previewing a conflicting merge must not start a merge.
    assert!(!test_repo.repo.path().join("MERGE_HEAD").exists());
}

#[test]
fn test_preview_merge_unknown_branch_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("base.txt", "base\n", "Base commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Merge(MergeCommand::Preview("no-such-branch".to_string())),
    );

    assert_eq!(result, None);
    assert_eq!(model.view_mode, ViewMode::Status);
    let message = expect_error_popup(&model);
    assert!(
        message.contains("Cannot preview merge of 'no-such-branch'"),
        "unexpected error message: {}",
        message
    );
}
