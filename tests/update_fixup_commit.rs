use magi::{
    git::{commit::get_recent_commits_for_fixup, test_repo::TestRepo},
    model::popup::{PopupContent, PopupContentCommand, SelectContext},
    msg::{Message, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

#[test]
fn test_show_fixup_commit_select_shows_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"])
        .commit("Second commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowFixupCommitSelect);

    assert_eq!(result, None);
    assert!(matches!(
        model.popup,
        Some(PopupContent::Command(PopupContentCommand::Select(_)))
    ));
    assert_eq!(model.select_context, Some(SelectContext::FixupCommit));
}

#[test]
fn test_fixup_commit_creates_fixup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    // Get the hash of the first user commit
    let commits = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
    let first_commit_ref = commits[0].clone();

    // Make a change and stage it
    test_repo
        .write_file_content("file1.txt", "modified content")
        .stage_files(&["file1.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::FixupCommit(first_commit_ref));

    assert_eq!(result, Some(Message::Refresh));

    // Verify the fixup commit was created
    let commits_after = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
    assert_eq!(commits_after.len(), 3); // Initial + First commit + fixup commit
    assert!(commits_after[0].contains("fixup! First commit"));
}

#[test]
fn test_fixup_commit_without_staged_changes_shows_error() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    // Get the hash of the first user commit
    let commits = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
    let first_commit_ref = commits[0].clone();

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::FixupCommit(first_commit_ref));

    assert_eq!(result, None);
    assert!(matches!(model.popup, Some(PopupContent::Error { .. })));
}

#[test]
fn test_fixup_commit_extracts_hash_from_selection() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    // Make a change and stage it
    test_repo
        .write_file_content("file1.txt", "modified content")
        .stage_files(&["file1.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    // Simulate the format from get_recent_commits_for_fixup: "hash - message"
    let commits = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
    let commit_ref = commits[0].clone(); // Should be in format "abc123 - First commit"

    let result = update(&mut model, Message::FixupCommit(commit_ref));

    assert_eq!(result, Some(Message::Refresh));

    // Verify the fixup commit was created with correct message
    let commits_after = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
    assert!(commits_after[0].contains("fixup! First commit"));
}
