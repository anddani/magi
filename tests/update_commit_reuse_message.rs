use crossterm::event::KeyCode;
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::{
        popup::{CommitPopupState, PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::{Message, SelectMessage, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, expect_select_popup, key};

fn open_commit_popup(model: &mut magi::model::Model) {
    model.popup = Some(PopupContent::Command(PopupContentCommand::Commit(
        CommitPopupState::default(),
    )));
}

fn commit_popup_with_reuse_message(rev: &str) -> PopupContent {
    PopupContent::Command(PopupContentCommand::Commit(CommitPopupState {
        author: None,
        reuse_message: Some(rev.to_string()),
    }))
}

/// Runs `git reset --hard HEAD` so the repo gets an ORIG_HEAD ref.
fn create_orig_head(test_repo: &TestRepo) {
    std::process::Command::new("git")
        .args(["reset", "--hard", "HEAD"])
        .current_dir(test_repo.repo_path())
        .output()
        .unwrap();
}

fn create_tag(test_repo: &TestRepo, name: &str) {
    std::process::Command::new("git")
        .args(["tag", name])
        .current_dir(test_repo.repo_path())
        .output()
        .unwrap();
}

// ── Key binding: 'C' in commit popup arg mode shows picker ────────────────────

#[test]
fn test_shift_c_in_arg_mode_shows_reuse_message_select() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    open_commit_popup(&mut model);
    model.arg_mode = true;

    let result = handle_key(key(KeyCode::Char('C')), &model);
    assert_eq!(result, Some(Message::ShowCommitReuseMessageSelect));
}

// ── Showing the picker ─────────────────────────────────────────────────────────

#[test]
fn test_show_select_lists_orig_head_first_when_it_exists() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    test_repo.create_branch("feature");
    create_tag(&test_repo, "v1.0.0");
    create_orig_head(&test_repo);

    let mut model = create_model_from_test_repo(&test_repo);
    open_commit_popup(&mut model);
    model.arg_mode = true;

    update(&mut model, Message::ShowCommitReuseMessageSelect);

    let state = expect_select_popup(&model);
    assert_eq!(
        state.all_options.first().map(String::as_str),
        Some("ORIG_HEAD")
    );
    assert!(state.all_options.contains(&"feature".to_string()));
    assert!(state.all_options.contains(&"v1.0.0".to_string()));
    assert_eq!(
        state.on_select,
        OnSelect::CommitReuseMessage { author: None }
    );
    assert!(!model.arg_mode);
}

#[test]
fn test_show_select_without_orig_head_lists_branches_and_tags_only() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");
    test_repo.create_branch("feature");

    let mut model = create_model_from_test_repo(&test_repo);
    open_commit_popup(&mut model);

    update(&mut model, Message::ShowCommitReuseMessageSelect);

    let state = expect_select_popup(&model);
    assert!(!state.all_options.contains(&"ORIG_HEAD".to_string()));
    assert!(state.all_options.contains(&"feature".to_string()));
}

#[test]
fn test_show_select_when_value_is_set_clears_it() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(commit_popup_with_reuse_message("HEAD~1"));

    update(&mut model, Message::ShowCommitReuseMessageSelect);

    let Some(PopupContent::Command(PopupContentCommand::Commit(state))) = &model.popup else {
        panic!("Expected commit popup, got: {:?}", model.popup);
    };
    assert_eq!(state.reuse_message, None);
}

// ── Confirming the picker ──────────────────────────────────────────────────────

#[test]
fn test_select_confirm_sets_reuse_message_and_preserves_author() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Reuse message from commit".to_string(),
            vec!["ORIG_HEAD".to_string(), "main".to_string()],
            OnSelect::CommitReuseMessage {
                author: Some("Jane Doe <jane@example.com>".to_string()),
            },
        ),
    )));

    update(&mut model, Message::Select(SelectMessage::Confirm));

    let Some(PopupContent::Command(PopupContentCommand::Commit(state))) = &model.popup else {
        panic!("Expected commit popup, got: {:?}", model.popup);
    };
    assert_eq!(state.reuse_message, Some("ORIG_HEAD".to_string()));
    assert_eq!(
        state.author,
        Some("Jane Doe <jane@example.com>".to_string())
    );
}

#[test]
fn test_author_select_preserves_reuse_message() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(commit_popup_with_reuse_message("ORIG_HEAD"));

    update(&mut model, Message::ShowCommitAuthorSelect);
    update(&mut model, Message::Select(SelectMessage::Confirm));

    let Some(PopupContent::Command(PopupContentCommand::Commit(state))) = &model.popup else {
        panic!("Expected commit popup, got: {:?}", model.popup);
    };
    assert!(state.author.is_some());
    assert_eq!(state.reuse_message, Some("ORIG_HEAD".to_string()));
}

// ── Committing with the argument ───────────────────────────────────────────────

#[test]
fn test_commit_with_reuse_message_reuses_the_message_without_editor() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Reused subject line");
    let first_commit = test_repo.head_hash();

    test_repo
        .write_file_content("file.txt", "modified")
        .stage_files(&["file.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(commit_popup_with_reuse_message(&first_commit));

    update(&mut model, Message::Commit);

    // A new commit exists and its message was copied from the first commit.
    assert_ne!(test_repo.head_hash(), first_commit);
    let output = std::process::Command::new("git")
        .args(["log", "-1", "--format=%s"])
        .current_dir(test_repo.repo_path())
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Reused subject line"
    );
}
