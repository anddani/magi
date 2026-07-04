use crossterm::event::KeyCode;
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::{
        LineContent,
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
