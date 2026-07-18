use magi::model::EditOp;
use magi::{
    git::{git_cmd, test_repo::TestRepo},
    model::popup::{InputContext, PopupContent},
    msg::{Message, StashCommand, StashType, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

fn stash_messages(test_repo: &TestRepo) -> Vec<String> {
    let output = git_cmd(test_repo.repo_path(), &["stash", "list", "--format=%gs"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.to_string())
        .collect()
}

#[test]
fn test_show_stash_worktree_input_opens_input_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowStashInput(StashType::Worktree));

    assert_eq!(result, None);

    assert!(matches!(
        model.popup,
        Some(PopupContent::Input(ref state))
            if state.title() == "Stash worktree message"
            && matches!(state.context, InputContext::Stash(StashType::Worktree))
    ));
}

#[test]
fn test_confirm_stash_worktree_input_with_message_triggers_stash_worktree() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    update(&mut model, Message::ShowStashInput(StashType::Worktree));

    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::Edit(EditOp::Insert('w'))),
    );
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::Edit(EditOp::Insert('i'))),
    );
    update(
        &mut model,
        Message::Input(magi::msg::InputMessage::Edit(EditOp::Insert('p'))),
    );

    let result = update(&mut model, Message::Input(magi::msg::InputMessage::Confirm));

    assert_eq!(model.popup, None);
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::Push(
            StashType::Worktree,
            "wip".to_string()
        )))
    );
}

#[test]
fn test_confirm_stash_worktree_input_empty_triggers_stash_worktree_with_empty_message() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);

    update(&mut model, Message::ShowStashInput(StashType::Worktree));

    let result = update(&mut model, Message::Input(magi::msg::InputMessage::Confirm));

    assert_eq!(model.popup, None);
    assert_eq!(
        result,
        Some(Message::Stash(StashCommand::Push(
            StashType::Worktree,
            String::new()
        )))
    );
}

// ── Stash worktree execution ───────────────────────────────────────────────────

#[test]
fn test_stash_worktree_stashes_unstaged_and_keeps_index() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "committed", "First commit");
    test_repo
        .write_file_content("file.txt", "staged")
        .stage_files(&["file.txt"])
        .write_file_content("file.txt", "unstaged");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Stash(StashCommand::Push(StashType::Worktree, "wip".to_string())),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(model.popup, None);
    assert_eq!(stash_messages(&test_repo), vec!["On main: wip".to_string()]);

    // The staged change is kept and the unstaged change is gone from the worktree
    let output = git_cmd(test_repo.repo_path(), &["show", ":file.txt"])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout), "staged");
    let content = std::fs::read_to_string(test_repo.repo_path().join("file.txt")).unwrap();
    assert_eq!(content, "staged");
}

#[test]
fn test_stash_worktree_with_no_unstaged_changes_shows_error() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "committed", "First commit");
    test_repo
        .write_file_content("file.txt", "staged")
        .stage_files(&["file.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Stash(StashCommand::Push(StashType::Worktree, "wip".to_string())),
    );

    assert_eq!(result, None);
    assert_eq!(
        model.popup,
        Some(PopupContent::Error {
            message: "No unstaged changes to save".to_string()
        })
    );
    assert!(stash_messages(&test_repo).is_empty());
}
