use crossterm::event::KeyCode;
use magi::{
    git::{git_cmd, test_repo::TestRepo},
    keys::handle_key,
    model::{
        LineContent, ToastStyle,
        popup::{MergePopupState, PopupContent, PopupContentCommand},
        select_popup::OnSelect,
    },
    msg::{MergeCommand, Message, OptionsSource, ShowSelectPopupConfig, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, expect_select_popup, find_line, key};

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
fn test_merge_edit_message_with_conflicts_shows_warning_toast() {
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
    let toast = model.toast.expect("Expected a toast after failed merge");
    assert_eq!(toast.style, ToastStyle::Warning);
    assert_eq!(toast.message, "Merge aborted");
    // The merge stays in progress so it can be resolved or aborted.
    assert!(test_repo.repo.path().join("MERGE_HEAD").exists());
}

#[test]
fn test_merge_branch_message_with_conflicts_shows_warning_toast() {
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
    let toast = model.toast.expect("Expected a toast after failed merge");
    assert_eq!(toast.style, ToastStyle::Warning);
    assert_eq!(toast.message, "Merge aborted");
    // The merge stays in progress so it can be resolved or aborted.
    assert!(test_repo.repo.path().join("MERGE_HEAD").exists());
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
