use magi::{
    git::test_repo::TestRepo,
    model::popup::{PopupContent, PopupContentCommand},
    model::select_popup::{OnSelect, SelectPopupState},
    msg::{Message, OptionsSource, PullCommand, ShowSelectPopupConfig, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

// ── ShowSelectPopup::PullElsewhere with no remotes shows error ────────────────

#[test]
fn test_pull_elsewhere_no_remotes_shows_error() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // No remotes → should show an error popup
    let result = update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Pull from".to_string(),
            source: OptionsSource::UpstreamBranches,
            on_select: OnSelect::PullElsewhere,
        }),
    );

    assert_eq!(result, None);
    assert!(matches!(model.popup, Some(PopupContent::Error { .. })));
}

// ── SelectContext::PullElsewhere routes to PullFromElsewhere message ──────────

#[test]
fn test_select_pull_elsewhere_routes_to_pull_command() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Simulate the state after the user has been shown the remote-branch picker
    // and "origin/main" is the selected item.
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Pull from".to_string(),
            vec!["origin/main".to_string(), "origin/dev".to_string()],
            OnSelect::PullElsewhere,
        ),
    )));

    // Confirm the selection (first item "origin/main" is selected by default)
    let result = update(
        &mut model,
        Message::Select(magi::msg::SelectMessage::Confirm),
    );

    assert_eq!(
        result,
        Some(Message::Pull(PullCommand::PullFromElsewhere(
            "origin/main".to_string()
        )))
    );

    // Popup should be dismissed
    assert!(model.popup.is_none());
}
