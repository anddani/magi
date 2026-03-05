use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::popup::{InputContext, PopupContent, PopupContentCommand},
    msg::{InputMessage, Message, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

// ── Key binding: 'S' in branch popup shows input ───────────────────────────────

#[test]
fn test_shift_s_in_branch_popup_shows_spinout_input() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Branch));

    let result = handle_key(key(KeyCode::Char('S')), &model);
    assert_eq!(result, Some(Message::ShowSpinoutBranchInput));
}

// ── ShowSpinoutBranchInput shows input popup ───────────────────────────────────

#[test]
fn test_show_spinout_branch_input_sets_input_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(&mut model, Message::ShowSpinoutBranchInput);

    assert_eq!(result, None);
    assert!(
        matches!(
            &model.popup,
            Some(PopupContent::Input(state)) if state.context == InputContext::SpinoutBranch
        ),
        "Expected Input popup with SpinoutBranch context"
    );
}

#[test]
fn test_show_spinout_branch_input_has_expected_title() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(&mut model, Message::ShowSpinoutBranchInput);

    if let Some(PopupContent::Input(state)) = &model.popup {
        assert!(
            state.title().contains("spin-out"),
            "Title should mention spin-out, got: {}",
            state.title()
        );
    } else {
        panic!("Expected Input popup");
    }
}

// ── Confirming input dispatches SpinoutBranch message ─────────────────────────

#[test]
fn test_confirm_input_dispatches_spinout_branch_message() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(&mut model, Message::ShowSpinoutBranchInput);

    update(&mut model, Message::Input(InputMessage::InputChar('f')));
    update(&mut model, Message::Input(InputMessage::InputChar('e')));
    update(&mut model, Message::Input(InputMessage::InputChar('a')));
    update(&mut model, Message::Input(InputMessage::InputChar('t')));

    let result = update(&mut model, Message::Input(InputMessage::Confirm));
    assert_eq!(result, Some(Message::SpinoutBranch("feat".to_string())));
}

// ── SpinoutBranch execution ────────────────────────────────────────────────────

#[test]
fn test_spinout_creates_new_branch_and_stays_on_current() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let current_branch_before = {
        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        repo.head().unwrap().shorthand().unwrap().to_string()
    };

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(&mut model, Message::SpinoutBranch("feature".to_string()));

    assert_eq!(result, Some(Message::Refresh));

    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();

    // Still on the original branch
    let current = repo.head().unwrap().shorthand().unwrap().to_string();
    assert_eq!(
        current, current_branch_before,
        "Should remain on the original branch after spinout"
    );

    // New branch exists
    let branches: Vec<String> = repo
        .branches(Some(git2::BranchType::Local))
        .unwrap()
        .flatten()
        .filter_map(|(b, _)| b.name().ok().flatten().map(|s| s.to_string()))
        .collect();
    assert!(branches.contains(&"feature".to_string()));
}

#[test]
fn test_spinout_returns_error_for_duplicate_branch_name() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    {
        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature", &head, false).unwrap();
    }

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(&mut model, Message::SpinoutBranch("feature".to_string()));

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup for duplicate branch name"
    );
}

#[test]
fn test_spinout_with_upstream_resets_current_branch_in_place() {
    use std::process::Command;

    let remote_repo = TestRepo::new();
    remote_repo
        .write_file_content("file.txt", "initial")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let remote_path = remote_repo.repo_path().to_path_buf();

    let local_dir = tempfile::tempdir().unwrap();
    let local_path = local_dir.path();
    Command::new("git")
        .args([
            "clone",
            remote_path.to_str().unwrap(),
            local_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to clone");

    Command::new("git")
        .args([
            "-C",
            local_path.to_str().unwrap(),
            "config",
            "user.email",
            "test@test.com",
        ])
        .output()
        .unwrap();
    Command::new("git")
        .args([
            "-C",
            local_path.to_str().unwrap(),
            "config",
            "user.name",
            "Test",
        ])
        .output()
        .unwrap();

    std::fs::write(local_path.join("file.txt"), "second").unwrap();
    Command::new("git")
        .args(["-C", local_path.to_str().unwrap(), "add", "file.txt"])
        .output()
        .unwrap();
    Command::new("git")
        .args([
            "-C",
            local_path.to_str().unwrap(),
            "commit",
            "-m",
            "Unpushed commit",
        ])
        .output()
        .unwrap();

    let unpushed_head = Command::new("git")
        .args(["-C", local_path.to_str().unwrap(), "rev-parse", "HEAD"])
        .output()
        .unwrap();
    let unpushed_head = String::from_utf8_lossy(&unpushed_head.stdout)
        .trim()
        .to_string();

    let upstream_merge_base_output = Command::new("git")
        .args([
            "-C",
            local_path.to_str().unwrap(),
            "merge-base",
            "HEAD",
            "origin/main",
        ])
        .output()
        .unwrap();
    let upstream_merge_base = String::from_utf8_lossy(&upstream_merge_base_output.stdout)
        .trim()
        .to_string();

    let git_info = magi::git::GitInfo::new_from_path(local_path).unwrap();
    let lines = git_info.get_lines().unwrap();
    let mut model = magi::model::Model {
        git_info,
        workdir: local_path.to_path_buf(),
        running_state: magi::model::RunningState::Running,
        ui_model: magi::model::UiModel {
            lines,
            ..Default::default()
        },
        theme: magi::config::Theme::default(),
        popup: None,
        toast: None,
        select_result: None,
        select_context: None,
        pty_state: None,
        arg_mode: false,
        pending_g: false,
        arguments: None,
        open_pr_branch: None,
        view_mode: magi::model::ViewMode::Status,
        cursor_reposition_context: None,
    };

    let result = update(&mut model, Message::SpinoutBranch("feature".to_string()));
    assert_eq!(result, Some(Message::Refresh));

    let local_repo = git2::Repository::open(local_path).unwrap();

    // Still on main
    let current = local_repo.head().unwrap().shorthand().unwrap().to_string();
    assert_eq!(current, "main", "Should remain on main after spinout");

    // feature branch points to the unpushed commit
    let feature_branch = local_repo
        .find_branch("feature", git2::BranchType::Local)
        .unwrap();
    let feature_commit = feature_branch
        .get()
        .peel_to_commit()
        .unwrap()
        .id()
        .to_string();
    assert_eq!(
        feature_commit, unpushed_head,
        "feature branch should point to the unpushed commit"
    );

    // main (current branch) was reset to the upstream merge-base
    let main_branch = local_repo
        .find_branch("main", git2::BranchType::Local)
        .unwrap();
    let main_commit = main_branch.get().peel_to_commit().unwrap().id().to_string();
    assert_eq!(
        main_commit, upstream_merge_base,
        "current branch should be reset to the upstream merge-base"
    );
}

#[test]
fn test_spinout_without_upstream_does_not_reset_current_branch() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "initial")
        .stage_files(&["file.txt"])
        .commit("Initial commit");
    test_repo
        .write_file_content("file.txt", "second")
        .stage_files(&["file.txt"])
        .commit("Second commit");

    let head_before = {
        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        repo.head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string()
    };

    let current_branch = {
        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        repo.head().unwrap().shorthand().unwrap().to_string()
    };

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(&mut model, Message::SpinoutBranch("feature".to_string()));

    assert_eq!(result, Some(Message::Refresh));

    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();

    // Current branch unchanged
    let old_branch = repo
        .find_branch(&current_branch, git2::BranchType::Local)
        .unwrap();
    let old_commit = old_branch.get().peel_to_commit().unwrap().id().to_string();
    assert_eq!(
        old_commit, head_before,
        "current branch should remain unchanged when there is no upstream"
    );

    // Still on current branch
    let current = repo.head().unwrap().shorthand().unwrap().to_string();
    assert_eq!(current, current_branch);
}
