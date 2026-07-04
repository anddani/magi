use crossterm::event::KeyCode;
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::{
        popup::{ApplyPopupState, PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::{Message, OptionsSource, SelectMessage, ShowSelectPopupConfig, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, key};

fn git(workdir: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new("git")
        .arg("-C")
        .arg(workdir)
        .args(args)
        .output()
        .unwrap()
}

fn current_branch(workdir: &std::path::Path) -> String {
    let output = git(workdir, &["rev-parse", "--abbrev-ref", "HEAD"]);
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// ── Key binding — 'd' in apply popup ─────────────────────────────────────────

#[test]
fn test_d_in_apply_popup_no_commits_shows_donate_commit_picker() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('d')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Donate commit".to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::DonateCommitPick,
        }))
    );
}

#[test]
fn test_d_in_apply_popup_with_commits_shows_target_branch_picker() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec!["abc1234".to_string()],
        },
    )));

    let result = handle_key(key(KeyCode::Char('d')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Donate to branch".to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::DonateTargetBranch {
                commits: vec!["abc1234".to_string()],
            },
        }))
    );
}

#[test]
fn test_d_in_apply_popup_with_multiple_commits_embeds_all() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec!["abc1234".to_string(), "def5678".to_string()],
        },
    )));

    let result = handle_key(key(KeyCode::Char('d')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Donate to branch".to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::DonateTargetBranch {
                commits: vec!["abc1234".to_string(), "def5678".to_string()],
            },
        }))
    );
}

// ── d key does nothing in in-progress mode ────────────────────────────────────

#[test]
fn test_d_in_apply_popup_in_progress_does_nothing() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: true,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('d')), &model);
    assert_eq!(result, None);
}

// ── Donate execution — commit moves to target branch ──────────────────────────

#[test]
fn test_donate_commit_to_target_branch() {
    let test_repo = TestRepo::new();

    // Create base commit and a target branch pointing at it
    test_repo.commit_file("base.txt", "base content", "Base commit");
    test_repo.create_branch("target-branch");

    // Add the commit to donate on top of main
    test_repo.commit_file("donated.txt", "donated content", "Commit to donate");
    let donate_hash = test_repo.head_hash();

    let workdir = test_repo.repo.workdir().unwrap().to_path_buf();
    let mut model = create_model_from_test_repo(&test_repo);

    // Execute donate
    let result = update(
        &mut model,
        Message::Donate {
            commits: vec![donate_hash],
            target: "target-branch".to_string(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    // No error popup should be set
    assert!(
        !matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected no error popup, got: {:?}",
        model.popup
    );

    // Donate returns to the original branch
    assert_eq!(current_branch(&workdir), "main");

    // The donated commit is on the target branch
    let target_log = git(&workdir, &["log", "--format=%s", "target-branch"]);
    let target_messages = String::from_utf8_lossy(&target_log.stdout).to_string();
    assert!(target_messages.contains("Commit to donate"));

    // ...and removed from the source branch
    let main_log = git(&workdir, &["log", "--format=%s", "main"]);
    let main_messages = String::from_utf8_lossy(&main_log.stdout).to_string();
    assert!(main_messages.contains("Base commit"));
    assert!(!main_messages.contains("Commit to donate"));
}

// ── Donate execution — invalid commit shows error ─────────────────────────────

#[test]
fn test_donate_invalid_commit_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    // Create a target branch (so LocalBranches picker has something)
    test_repo.create_branch("target-branch");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Donate {
            commits: vec!["deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string()],
            target: "target-branch".to_string(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup for invalid commit hash"
    );
}

#[test]
fn test_donate_nonexistent_target_branch_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    let donate_hash = test_repo.head_hash();

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Donate {
            commits: vec![donate_hash],
            target: "no-such-branch".to_string(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup for nonexistent target branch"
    );
}

// ── Donate with pre-selected commits embedded in OnSelect ─────────────────────

#[test]
fn test_donate_target_branch_on_select_routes_to_donate_message() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    // Create a local branch for the picker
    test_repo.create_branch("feature-branch");

    let mut model = create_model_from_test_repo(&test_repo);

    // Simulate the select popup with DonateTargetBranch on_select
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Donate to branch".to_string(),
            vec!["feature-branch".to_string()],
            OnSelect::DonateTargetBranch {
                commits: vec!["abc1234".to_string()],
            },
        ),
    )));

    // Confirm selection
    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::Donate {
            commits: vec!["abc1234".to_string()],
            target: "feature-branch".to_string(),
        })
    );
}
