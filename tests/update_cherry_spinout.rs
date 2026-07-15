use crossterm::event::KeyCode;
use magi::model::InputField;
use magi::{
    git::{log::get_log_entries, test_repo::TestRepo},
    keys::handle_key,
    model::{
        popup::{
            ApplyPopupState, InputContext, InputPopupState, PopupContent, PopupContentCommand,
        },
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::{
        InputMessage, LogType, Message, OptionsSource, SelectMessage, ShowSelectPopupConfig,
        update::update,
    },
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

fn commit_hash(test_repo: &TestRepo, message: &str) -> String {
    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let log = get_log_entries(&repo, &LogType::Current, true, false).unwrap();
    log.iter()
        .find(|e| e.hash.is_some() && e.message.as_deref() == Some(message))
        .and_then(|e| e.hash.clone())
        .unwrap_or_else(|| panic!("Could not find commit '{}'", message))
}

// ── Key binding — 'n' in apply popup ──────────────────────────────────────────

#[test]
fn test_n_in_apply_popup_no_commits_shows_spinout_commit_picker() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('n')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Spinout commit".to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::CherrySpinoutCommitPick,
        }))
    );
}

#[test]
fn test_n_in_apply_popup_with_commits_shows_spinout_root_picker() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec!["abc1234".to_string(), "def5678".to_string()],
        },
    )));

    let result = handle_key(key(KeyCode::Char('n')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Spinout root".to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::CherrySpinoutRootPick {
                commits: vec!["abc1234".to_string(), "def5678".to_string()],
            },
        }))
    );
}

#[test]
fn test_n_in_apply_popup_in_progress_does_nothing() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: true,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('n')), &model);
    assert_eq!(result, None);
}

// ── Select routing — commit pick → root pick → input ─────────────────────────

#[test]
fn test_spinout_commit_pick_routes_to_root_picker() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Spinout commit".to_string(),
            vec!["abc1234".to_string()],
            OnSelect::CherrySpinoutCommitPick,
        ),
    )));

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Spinout root".to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::CherrySpinoutRootPick {
                commits: vec!["abc1234".to_string()],
            },
        }))
    );
}

#[test]
fn test_spinout_root_pick_routes_to_branch_name_input() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Spinout root".to_string(),
            vec!["main".to_string()],
            OnSelect::CherrySpinoutRootPick {
                commits: vec!["abc1234".to_string()],
            },
        ),
    )));

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::ShowCherrySpinoutInput {
            commits: vec!["abc1234".to_string()],
            root: "main".to_string(),
        })
    );
}

#[test]
fn test_spinout_input_confirm_routes_to_cherry_spinout() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);
    let mut state = InputPopupState::new(InputContext::CherrySpinout {
        commits: vec!["abc1234".to_string()],
        root: "main".to_string(),
    });
    state.input = InputField::from_text("topic");
    model.popup = Some(PopupContent::Input(state));

    let result = update(&mut model, Message::Input(InputMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::CherrySpinout {
            commits: vec!["abc1234".to_string()],
            branch: "topic".to_string(),
            root: "main".to_string(),
        })
    );
}

// ── Execution ─────────────────────────────────────────────────────────────────

#[test]
fn test_cherry_spinout_moves_commit_and_stays_on_current_branch() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("a.txt", "content a", "Commit A");
    test_repo.commit_file("b.txt", "content b", "Commit B");

    let workdir = test_repo.repo.workdir().unwrap().to_path_buf();
    let hash_a = commit_hash(&test_repo, "Commit A");
    let hash_b = commit_hash(&test_repo, "Commit B");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::CherrySpinout {
            commits: vec![hash_b.clone()],
            branch: "topic".to_string(),
            root: hash_a.clone(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(
        !matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected no error popup, got: {:?}",
        model.popup
    );

    // Spinout stays on the original branch (unlike spinoff)
    assert_eq!(current_branch(&workdir), "main");

    // The spun-out commit is on the new branch
    let topic_log = git(&workdir, &["log", "--format=%s", "topic"]);
    let topic_messages = String::from_utf8_lossy(&topic_log.stdout).to_string();
    assert!(topic_messages.contains("Commit B"));

    // ...and removed from the source branch
    let main_log = git(&workdir, &["log", "--format=%s", "main"]);
    let main_messages = String::from_utf8_lossy(&main_log.stdout).to_string();
    assert!(main_messages.contains("Commit A"));
    assert!(!main_messages.contains("Commit B"));
}

#[test]
fn test_cherry_spinout_multiple_commits() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("a.txt", "content a", "Commit A");
    test_repo.commit_file("b.txt", "content b", "Commit B");
    test_repo.commit_file("c.txt", "content c", "Commit C");

    let workdir = test_repo.repo.workdir().unwrap().to_path_buf();
    let hash_a = commit_hash(&test_repo, "Commit A");
    let hash_b = commit_hash(&test_repo, "Commit B");
    let hash_c = commit_hash(&test_repo, "Commit C");

    let mut model = create_model_from_test_repo(&test_repo);

    // Commits are ordered oldest first
    let result = update(
        &mut model,
        Message::CherrySpinout {
            commits: vec![hash_b, hash_c],
            branch: "topic".to_string(),
            root: hash_a,
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(
        !matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected no error popup, got: {:?}",
        model.popup
    );

    assert_eq!(current_branch(&workdir), "main");

    let topic_log = git(&workdir, &["log", "--format=%s", "topic"]);
    let topic_messages = String::from_utf8_lossy(&topic_log.stdout).to_string();
    assert!(topic_messages.contains("Commit B"));
    assert!(topic_messages.contains("Commit C"));

    let main_log = git(&workdir, &["log", "--format=%s", "main"]);
    let main_messages = String::from_utf8_lossy(&main_log.stdout).to_string();
    assert!(main_messages.contains("Commit A"));
    assert!(!main_messages.contains("Commit B"));
    assert!(!main_messages.contains("Commit C"));
}

#[test]
fn test_cherry_spinout_duplicate_branch_name_shows_error() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("a.txt", "content a", "Commit A");

    let workdir = test_repo.repo.workdir().unwrap().to_path_buf();
    let hash_a = commit_hash(&test_repo, "Commit A");

    // Create a branch with the name we will try to spin out to
    test_repo.create_branch("existing");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::CherrySpinout {
            commits: vec![hash_a],
            branch: "existing".to_string(),
            root: "main".to_string(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup for duplicate branch name"
    );

    // Still on the original branch
    assert_eq!(current_branch(&workdir), "main");
}
