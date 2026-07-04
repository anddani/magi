use crossterm::event::KeyCode;
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::{
        popup::{PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::{Message, OptionsSource, SelectMessage, ShowSelectPopupConfig, update::update},
};

mod utils;
use utils::{assert_no_popup, create_model_from_test_repo, expect_error_popup, key};

// ── Key binding — 'b' in branch popup ─────────────────────────────────────────

#[test]
fn test_b_in_branch_popup_shows_checkout_picker() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Branch));

    let result = handle_key(key(KeyCode::Char('b')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Checkout".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::CheckoutBranch,
        }))
    );
}

// ── Select routing — branch pick → CheckoutBranch message ─────────────────────

#[test]
fn test_select_branch_routes_to_checkout_branch_message() {
    let test_repo = TestRepo::new();
    test_repo.create_branch("feature");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Checkout".to_string(),
            vec!["feature".to_string()],
            OnSelect::CheckoutBranch,
        ),
    )));

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(result, Some(Message::CheckoutBranch("feature".to_string())));
}

// ── Execution ─────────────────────────────────────────────────────────────────

#[test]
fn test_checkout_branch_moves_head_to_branch() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("a.txt", "content a", "Commit A");

    // Create a branch at the current commit, then advance main past it
    test_repo.create_branch("feature");
    test_repo.commit_file("b.txt", "content b", "Commit B");
    assert_ne!(test_repo.head_hash(), test_repo.branch_hash("feature"));

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::CheckoutBranch("feature".to_string()));

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);

    let head = test_repo.repo.head().unwrap();
    assert_eq!(head.shorthand().unwrap(), "feature");
    assert_eq!(test_repo.head_hash(), test_repo.branch_hash("feature"));
}

#[test]
fn test_checkout_nonexistent_branch_shows_error_popup() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::CheckoutBranch("nonexistent-branch".to_string()),
    );

    assert_eq!(result, None);
    expect_error_popup(&model);

    // Still on the original branch
    let head = test_repo.repo.head().unwrap();
    assert_eq!(head.shorthand().unwrap(), "main");
}

#[test]
fn test_checkout_with_conflicting_local_changes_shows_error_popup() {
    let test_repo = TestRepo::new();

    // 'feature' points at a commit where conflict.txt has different content
    test_repo.commit_file("conflict.txt", "v1", "Add conflict v1");
    test_repo.create_branch("feature");
    test_repo.commit_file("conflict.txt", "v2", "Change conflict to v2");

    // Dirty the working tree so checkout would overwrite local changes
    test_repo.write_file_content("conflict.txt", "dirty local edit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::CheckoutBranch("feature".to_string()));

    assert_eq!(result, None);
    expect_error_popup(&model);

    // Still on the original branch
    let head = test_repo.repo.head().unwrap();
    assert_eq!(head.shorthand().unwrap(), "main");
}
