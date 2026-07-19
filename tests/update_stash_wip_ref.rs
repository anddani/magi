use crossterm::event::KeyCode;
use magi::{
    git::{git_cmd, test_repo::TestRepo},
    keys::handle_key,
    model::popup::{PopupContent, PopupContentCommand},
    msg::{Message, StashCommand, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, key};

fn show_ref(test_repo: &TestRepo, rev: &str, file: &str) -> Result<String, String> {
    let output = git_cmd(test_repo.repo_path(), &["show", &format!("{rev}:{file}")])
        .output()
        .unwrap();
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

// ── Stash popup keys ───────────────────────────────────────────────────────────

#[test]
fn test_r_in_stash_popup_triggers_wip_commit() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = handle_key(key(KeyCode::Char('r')), &model);
    assert_eq!(result, Some(Message::Stash(StashCommand::ToWipRef)));
}

// ── Wip commit execution ───────────────────────────────────────────────────────

#[test]
fn test_wip_commit_records_changes_and_keeps_them() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo
        .write_file_content("file1.txt", "staged")
        .stage_files(&["file1.txt"])
        .write_file_content("file1.txt", "unstaged");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = update(&mut model, Message::Stash(StashCommand::ToWipRef));

    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(model.popup, None);

    // The wip refs record the index and working tree states
    assert_eq!(
        show_ref(&test_repo, "refs/wip/index/refs/heads/main", "file1.txt"),
        Ok("staged".to_string())
    );
    assert_eq!(
        show_ref(&test_repo, "refs/wip/wtree/refs/heads/main", "file1.txt"),
        Ok("unstaged".to_string())
    );

    // Nothing is reset — the index and working tree are untouched
    let output = git_cmd(test_repo.repo_path(), &["show", ":file1.txt"])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout), "staged");
    let content = std::fs::read_to_string(test_repo.repo_path().join("file1.txt")).unwrap();
    assert_eq!(content, "unstaged");
}

#[test]
fn test_wip_commit_does_not_create_a_stash() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo.write_file_content("file1.txt", "modified");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    update(&mut model, Message::Stash(StashCommand::ToWipRef));

    let output = git_cmd(test_repo.repo_path(), &["stash", "list"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
}

#[test]
fn test_wip_commit_with_clean_tree_succeeds_without_wip_commit() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = update(&mut model, Message::Stash(StashCommand::ToWipRef));

    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(model.popup, None);

    // The wip refs are started at the branch tip's tree with no wip commit
    assert_eq!(
        show_ref(&test_repo, "refs/wip/wtree/refs/heads/main", "file1.txt"),
        Ok("content1".to_string())
    );
}
