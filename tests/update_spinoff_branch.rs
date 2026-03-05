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

// ── Key binding: 's' in branch popup shows input ───────────────────────────────

#[test]
fn test_s_in_branch_popup_shows_spinoff_input() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Branch));

    let result = handle_key(key(KeyCode::Char('s')), &model);
    assert_eq!(result, Some(Message::ShowSpinoffBranchInput));
}

// ── ShowSpinoffBranchInput shows input popup ───────────────────────────────────

#[test]
fn test_show_spinoff_branch_input_sets_input_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(&mut model, Message::ShowSpinoffBranchInput);

    assert_eq!(result, None);
    assert!(
        matches!(
            &model.popup,
            Some(PopupContent::Input(state)) if state.context == InputContext::SpinoffBranch
        ),
        "Expected Input popup with SpinoffBranch context"
    );
}

#[test]
fn test_show_spinoff_branch_input_has_expected_title() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(&mut model, Message::ShowSpinoffBranchInput);

    if let Some(PopupContent::Input(state)) = &model.popup {
        assert!(
            state.title().contains("spin-off"),
            "Title should mention spin-off, got: {}",
            state.title()
        );
    } else {
        panic!("Expected Input popup");
    }
}

// ── Confirming input dispatches SpinoffBranch message ─────────────────────────

#[test]
fn test_confirm_input_dispatches_spinoff_branch_message() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(&mut model, Message::ShowSpinoffBranchInput);

    // Type a branch name
    update(&mut model, Message::Input(InputMessage::InputChar('f')));
    update(&mut model, Message::Input(InputMessage::InputChar('e')));
    update(&mut model, Message::Input(InputMessage::InputChar('a')));
    update(&mut model, Message::Input(InputMessage::InputChar('t')));

    let result = update(&mut model, Message::Input(InputMessage::Confirm));
    assert_eq!(result, Some(Message::SpinoffBranch("feat".to_string())));
}

#[test]
fn test_confirm_empty_input_keeps_popup_open() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(&mut model, Message::ShowSpinoffBranchInput);

    // Confirm without typing anything
    let result = update(&mut model, Message::Input(InputMessage::Confirm));
    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Input(_))),
        "Popup should remain open when input is empty"
    );
}

// ── SpinoffBranch execution ────────────────────────────────────────────────────

#[test]
fn test_spinoff_creates_new_branch_and_checks_it_out() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(&mut model, Message::SpinoffBranch("feature".to_string()));

    assert_eq!(result, Some(Message::Refresh));

    // Verify the new branch exists and we're on it
    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let head = repo.head().unwrap();
    let current = head.shorthand().unwrap();
    assert_eq!(current, "feature");

    let branches: Vec<String> = repo
        .branches(Some(git2::BranchType::Local))
        .unwrap()
        .flatten()
        .filter_map(|(b, _)| b.name().ok().flatten().map(|s| s.to_string()))
        .collect();
    assert!(branches.contains(&"feature".to_string()));
}

#[test]
fn test_spinoff_returns_error_for_duplicate_branch_name() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    // Create the branch first
    {
        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature", &head, false).unwrap();
    }

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(&mut model, Message::SpinoffBranch("feature".to_string()));

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup for duplicate branch name"
    );
}

#[test]
fn test_spinoff_with_upstream_resets_old_branch() {
    use std::process::Command;

    // Set up: a "remote" repo and a local clone with tracked branch
    let remote_repo = TestRepo::new();
    remote_repo
        .write_file_content("file.txt", "initial")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let remote_path = remote_repo.repo_path().to_path_buf();

    // Clone the remote
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

    // Make commits on the local branch (unpushed)
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

    // Get the hash of the original upstream commit (where main was before the unpushed commit)
    let upstream_commit_output = Command::new("git")
        .args([
            "-C",
            local_path.to_str().unwrap(),
            "merge-base",
            "HEAD",
            "origin/main",
        ])
        .output()
        .unwrap();
    let upstream_merge_base = String::from_utf8_lossy(&upstream_commit_output.stdout)
        .trim()
        .to_string();

    // Get current main HEAD (with the unpushed commit)
    let main_head_before = Command::new("git")
        .args(["-C", local_path.to_str().unwrap(), "rev-parse", "HEAD"])
        .output()
        .unwrap();
    let main_head_before = String::from_utf8_lossy(&main_head_before.stdout)
        .trim()
        .to_string();

    // Create model from local repo
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

    // Perform spinoff
    let result = update(&mut model, Message::SpinoffBranch("feature".to_string()));
    assert_eq!(result, Some(Message::Refresh));

    // Verify new branch "feature" points to what was the main HEAD
    let local_repo = git2::Repository::open(local_path).unwrap();
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
        feature_commit, main_head_before,
        "feature branch should point to the unpushed commit"
    );

    // Verify old main branch was reset to the upstream merge-base
    let main_branch = local_repo
        .find_branch("main", git2::BranchType::Local)
        .unwrap();
    let main_commit = main_branch.get().peel_to_commit().unwrap().id().to_string();
    assert_eq!(
        main_commit, upstream_merge_base,
        "main branch should be reset to the upstream merge-base"
    );
}

#[test]
fn test_spinoff_without_upstream_does_not_reset_old_branch() {
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
    let result = update(&mut model, Message::SpinoffBranch("feature".to_string()));

    assert_eq!(result, Some(Message::Refresh));

    // Old branch should NOT have been reset (no upstream)
    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let old_branch = repo
        .find_branch(&current_branch, git2::BranchType::Local)
        .unwrap();
    let old_commit = old_branch.get().peel_to_commit().unwrap().id().to_string();
    assert_eq!(
        old_commit, head_before,
        "old branch should remain unchanged when there is no upstream"
    );
}
