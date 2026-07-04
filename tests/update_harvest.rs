use crossterm::event::KeyCode;
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::{
        popup::{ApplyPopupState, PopupContent, PopupContentCommand},
        select_popup::OnSelect,
    },
    msg::{Message, OptionsSource, ShowSelectPopupConfig, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, key};

// ── Key binding — 'h' in apply popup ─────────────────────────────────────────

#[test]
fn test_h_in_apply_popup_no_commits_shows_harvest_commit_picker() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('h')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Harvest commit".to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::HarvestCommitPick,
        }))
    );
}

#[test]
fn test_h_in_apply_popup_with_commits_shows_source_branch_picker() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec!["abc1234".to_string()],
        },
    )));

    let result = handle_key(key(KeyCode::Char('h')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Harvest from branch".to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::HarvestSourceBranch {
                commits: vec!["abc1234".to_string()],
            },
        }))
    );
}

#[test]
fn test_h_in_apply_popup_with_multiple_commits_embeds_all() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec!["abc1234".to_string(), "def5678".to_string()],
        },
    )));

    let result = handle_key(key(KeyCode::Char('h')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Harvest from branch".to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::HarvestSourceBranch {
                commits: vec!["abc1234".to_string(), "def5678".to_string()],
            },
        }))
    );
}

// ── h key does nothing in in-progress mode ────────────────────────────────────

#[test]
fn test_h_in_apply_popup_in_progress_does_nothing() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: true,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('h')), &model);
    assert_eq!(result, None);
}

// ── Harvest execution — commits at tip of source branch ───────────────────────

#[test]
fn test_harvest_commits_at_tip_of_source_branch() {
    let test_repo = TestRepo::new();

    // Create base commit
    test_repo.commit_file("base.txt", "base content", "Base commit");

    // Create source branch and add commits to harvest
    let workdir = test_repo.repo.workdir().unwrap().to_path_buf();
    std::process::Command::new("git")
        .arg("-C")
        .arg(&workdir)
        .args(["checkout", "-b", "source-branch"])
        .output()
        .unwrap();

    test_repo.commit_file("harvested.txt", "harvested content", "Commit to harvest");

    // Get the hash of the commit to harvest (the tip of source-branch)
    let harvest_hash = test_repo.head_hash();

    // Switch back to main
    std::process::Command::new("git")
        .arg("-C")
        .arg(&workdir)
        .args(["checkout", "main"])
        .output()
        .unwrap();

    let mut model = create_model_from_test_repo(&test_repo);

    // Execute harvest
    let result = update(
        &mut model,
        Message::Harvest {
            commits: vec![harvest_hash.clone()],
            source: "source-branch".to_string(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    // No error popup should be set
    assert!(
        !matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected no error popup, got: {:?}",
        model.popup
    );

    // The harvested file should now exist on main
    assert!(
        workdir.join("harvested.txt").exists(),
        "Harvested file should exist on current branch"
    );
}

// ── Harvest execution — invalid commit shows error ────────────────────────────

#[test]
fn test_harvest_invalid_commit_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    // Create a source branch (so LocalBranches picker has something)
    test_repo.create_branch("source-branch");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::Harvest {
            commits: vec!["deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string()],
            source: "source-branch".to_string(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup for invalid commit hash"
    );
}

// ── Harvest with pre-selected commits embedded in OnSelect ───────────────────

#[test]
fn test_harvest_source_branch_on_select_routes_to_harvest_message() {
    use magi::model::popup::{PopupContent, PopupContentCommand};
    use magi::model::select_popup::SelectPopupState;
    use magi::msg::update::update as msg_update;

    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    // Create a local branch for the picker
    test_repo.create_branch("feature-branch");

    let mut model = create_model_from_test_repo(&test_repo);

    // Simulate the select popup with HarvestSourceBranch on_select
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Harvest from branch".to_string(),
            vec!["feature-branch".to_string()],
            OnSelect::HarvestSourceBranch {
                commits: vec!["abc1234".to_string()],
            },
        ),
    )));

    // Confirm selection
    let result = msg_update(
        &mut model,
        magi::msg::Message::Select(magi::msg::SelectMessage::Confirm),
    );

    assert_eq!(
        result,
        Some(Message::Harvest {
            commits: vec!["abc1234".to_string()],
            source: "feature-branch".to_string(),
        })
    );
}
