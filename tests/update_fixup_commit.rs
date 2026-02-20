use magi::{
    git::{commit::get_recent_commits_for_fixup, test_repo::TestRepo},
    model::{
        Toast,
        popup::{CommitSelectPopupState, PopupContent, PopupContentCommand, SelectContext},
    },
    msg::{FixupType, Message, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

#[test]
fn test_show_fixup_commit_select_without_staged_changes_shows_toast() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowFixupCommitSelect(FixupType::Fixup));

    assert_eq!(result, Some(Message::DismissPopup));
    assert!(model.popup.is_none());
    assert!(matches!(model.toast, Some(Toast { .. })));
    if let Some(toast) = model.toast {
        assert_eq!(toast.message, "Nothing staged to fixup");
    }
}

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

    // Stage some changes to prepare for fixup
    test_repo
        .write_file_content("file1.txt", "modified content")
        .stage_files(&["file1.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowFixupCommitSelect(FixupType::Fixup));

    assert_eq!(result, None);
    assert!(matches!(
        model.popup,
        Some(PopupContent::Command(PopupContentCommand::CommitSelect(_)))
    ));
    assert_eq!(
        model.select_context,
        Some(SelectContext::FixupCommit(FixupType::Fixup))
    );
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
    let first_commit_hash = commits[0].hash.as_ref().unwrap().to_string();

    // Make a change and stage it
    test_repo
        .write_file_content("file1.txt", "modified content")
        .stage_files(&["file1.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::FixupCommit(first_commit_hash, FixupType::Fixup),
    );

    assert_eq!(result, Some(Message::Refresh));

    // Verify the fixup commit was created
    let commits_after = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
    assert_eq!(commits_after.len(), 3); // Initial + First commit + fixup commit
    assert_eq!(
        commits_after[0].message.as_deref(),
        Some("fixup! First commit")
    );
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
    let first_commit_hash = commits[0].hash.as_ref().unwrap().to_string();

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::FixupCommit(first_commit_hash, FixupType::Fixup),
    );

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

    // Get the hash from the commit entry
    let commits = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
    let commit_hash = commits[0].hash.as_ref().unwrap().to_string();

    let result = update(
        &mut model,
        Message::FixupCommit(commit_hash, FixupType::Fixup),
    );

    assert_eq!(result, Some(Message::Refresh));

    // Verify the fixup commit was created with correct message
    let commits_after = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
    assert_eq!(
        commits_after[0].message.as_deref(),
        Some("fixup! First commit")
    );
}

#[test]
fn test_show_squash_commit_select_without_staged_changes_shows_toast() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::ShowFixupCommitSelect(FixupType::Squash),
    );

    assert_eq!(result, Some(Message::DismissPopup));
    assert!(model.popup.is_none());
    assert!(matches!(model.toast, Some(Toast { .. })));
    if let Some(toast) = model.toast {
        assert_eq!(toast.message, "Nothing staged to fixup");
    }
}

#[test]
fn test_show_squash_commit_select_shows_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"])
        .commit("Second commit");

    // Stage some changes to prepare for squash
    test_repo
        .write_file_content("file1.txt", "modified content")
        .stage_files(&["file1.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::ShowFixupCommitSelect(FixupType::Squash),
    );

    assert_eq!(result, None);
    assert!(matches!(
        model.popup,
        Some(PopupContent::Command(PopupContentCommand::Select(_)))
    ));
    assert_eq!(
        model.select_context,
        Some(SelectContext::FixupCommit(FixupType::Squash))
    );
}

#[test]
fn test_squash_commit_creates_squash() {
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

    let result = update(
        &mut model,
        Message::FixupCommit(first_commit_ref, FixupType::Squash),
    );

    assert_eq!(result, Some(Message::Refresh));

    // Verify the squash commit was created
    let commits_after = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
    assert_eq!(commits_after.len(), 3); // Initial + First commit + squash commit
    assert!(commits_after[0].contains("squash! First commit"));
}

#[test]
fn test_squash_commit_without_staged_changes_shows_error() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    // Get the hash of the first user commit
    let commits = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
    let first_commit_ref = commits[0].clone();

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::FixupCommit(first_commit_ref, FixupType::Squash),
    );

    assert_eq!(result, None);
    assert!(matches!(model.popup, Some(PopupContent::Error { .. })));
}

#[test]
fn test_squash_commit_extracts_hash_from_selection() {
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

    let result = update(
        &mut model,
        Message::FixupCommit(commit_ref, FixupType::Squash),
    );

    assert_eq!(result, Some(Message::Refresh));

    // Verify the squash commit was created with correct message
    let commits_after = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
    assert!(commits_after[0].contains("squash! First commit"));
}
