use crossterm::event::KeyCode;
use magi::{
    git::{git_cmd, test_repo::TestRepo},
    keys::handle_key,
    model::popup::{PopupContent, PopupContentCommand},
    msg::{Message, StashCommand, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, key};

fn stash_messages(test_repo: &TestRepo) -> Vec<String> {
    let output = git_cmd(test_repo.repo_path(), &["stash", "list", "--format=%gs"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.to_string())
        .collect()
}

// ── Stash popup keys ───────────────────────────────────────────────────────────

#[test]
fn test_upper_z_in_stash_popup_triggers_snapshot() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = handle_key(key(KeyCode::Char('Z')), &model);
    assert_eq!(result, Some(Message::Stash(StashCommand::Snapshot)));
}

// ── Snapshot execution ─────────────────────────────────────────────────────────

#[test]
fn test_snapshot_creates_stash_and_keeps_working_tree() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo.write_file_content("file1.txt", "modified");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = update(&mut model, Message::Stash(StashCommand::Snapshot));

    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(model.popup, None);

    // The snapshot is recorded as a stash with the "WIP on <branch>" message
    let messages = stash_messages(&test_repo);
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("WIP on"));

    // The working tree is untouched — the modification is still there
    let content = std::fs::read_to_string(test_repo.repo_path().join("file1.txt")).unwrap();
    assert_eq!(content, "modified");
}

#[test]
fn test_snapshot_includes_staged_changes() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo
        .write_file_content("file1.txt", "staged change")
        .stage_files(&["file1.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::Stash(StashCommand::Snapshot));

    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(stash_messages(&test_repo).len(), 1);

    // The index is untouched — the change is still staged
    let output = git_cmd(test_repo.repo_path(), &["diff", "--cached", "--name-only"])
        .output()
        .unwrap();
    let staged = String::from_utf8_lossy(&output.stdout);
    assert!(staged.contains("file1.txt"));
}

#[test]
fn test_snapshot_with_no_changes_shows_error() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = update(&mut model, Message::Stash(StashCommand::Snapshot));

    assert_eq!(result, None);
    assert_eq!(
        model.popup,
        Some(PopupContent::Error {
            message: "No local changes to save".to_string()
        })
    );
    assert!(stash_messages(&test_repo).is_empty());
}

// ── Snapshot index ─────────────────────────────────────────────────────────────

#[test]
fn test_upper_i_in_stash_popup_triggers_index_snapshot() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = handle_key(key(KeyCode::Char('I')), &model);
    assert_eq!(result, Some(Message::Stash(StashCommand::SnapshotIndex)));
}

#[test]
fn test_index_snapshot_creates_stash_and_keeps_index() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo
        .write_file_content("file1.txt", "staged change")
        .stage_files(&["file1.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = update(&mut model, Message::Stash(StashCommand::SnapshotIndex));

    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(model.popup, None);

    let messages = stash_messages(&test_repo);
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("WIP on"));

    // The index is untouched — the change is still staged
    let output = git_cmd(test_repo.repo_path(), &["diff", "--cached", "--name-only"])
        .output()
        .unwrap();
    let staged = String::from_utf8_lossy(&output.stdout);
    assert!(staged.contains("file1.txt"));
}

#[test]
fn test_index_snapshot_with_nothing_staged_shows_error() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo.write_file_content("file1.txt", "unstaged only");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = update(&mut model, Message::Stash(StashCommand::SnapshotIndex));

    assert_eq!(result, None);
    assert_eq!(
        model.popup,
        Some(PopupContent::Error {
            message: "No staged changes to save".to_string()
        })
    );
    assert!(stash_messages(&test_repo).is_empty());
}

// ── Snapshot worktree ──────────────────────────────────────────────────────────

#[test]
fn test_upper_w_in_stash_popup_triggers_worktree_snapshot() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = handle_key(key(KeyCode::Char('W')), &model);
    assert_eq!(result, Some(Message::Stash(StashCommand::SnapshotWorktree)));
}

#[test]
fn test_worktree_snapshot_creates_stash_and_keeps_worktree() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo.write_file_content("file1.txt", "unstaged change");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = update(&mut model, Message::Stash(StashCommand::SnapshotWorktree));

    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(model.popup, None);

    let messages = stash_messages(&test_repo);
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("WIP on"));

    // The working tree is untouched — the modification is still there
    let content = std::fs::read_to_string(test_repo.repo_path().join("file1.txt")).unwrap();
    assert_eq!(content, "unstaged change");
}

#[test]
fn test_worktree_snapshot_with_nothing_unstaged_shows_error() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo
        .write_file_content("file1.txt", "staged only")
        .stage_files(&["file1.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = update(&mut model, Message::Stash(StashCommand::SnapshotWorktree));

    assert_eq!(result, None);
    assert_eq!(
        model.popup,
        Some(PopupContent::Error {
            message: "No unstaged changes to save".to_string()
        })
    );
    assert!(stash_messages(&test_repo).is_empty());
}
