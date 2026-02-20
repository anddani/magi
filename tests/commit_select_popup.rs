use magi::{
    git::{log::get_log_entries, test_repo::TestRepo},
    model::popup::{CommitSelectPopupState, PopupContent, PopupContentCommand},
    msg::{FixupType, LogType, Message, SelectMessage, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

/// Helper to get log entries for testing (filters out graph-only entries)
fn get_log_entries_for_test(test_repo: &TestRepo) -> Vec<magi::model::LogEntry> {
    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let mut entries = get_log_entries(&repo, LogType::Current).unwrap();
    entries.retain(|e| e.is_commit());
    entries
}

#[test]
fn test_commit_select_popup_displays_log_entries() {
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

    // Show the commit select popup
    let _result = update(&mut model, Message::ShowFixupCommitSelect(FixupType::Fixup));

    // Verify popup is CommitSelect, not plain Select
    assert!(matches!(
        model.popup,
        Some(PopupContent::Command(PopupContentCommand::CommitSelect(_)))
    ));

    // Extract the state and verify it contains LogEntry objects
    if let Some(PopupContent::Command(PopupContentCommand::CommitSelect(state))) = &model.popup {
        assert!(state.all_commits.len() >= 2);
        // Verify commits have proper structure (hash, message, etc.)
        assert!(state.all_commits[0].hash.is_some());
        assert!(state.all_commits[0].message.is_some());
    } else {
        panic!("Expected CommitSelect popup");
    }
}

#[test]
fn test_commit_select_popup_filters_by_hash() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"])
        .commit("Second commit");

    test_repo
        .write_file_content("file3.txt", "content3")
        .stage_files(&["file3.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    // Show the commit select popup
    let _result = update(&mut model, Message::ShowFixupCommitSelect(FixupType::Fixup));

    // Get initial commit count
    let initial_count =
        if let Some(PopupContent::Command(PopupContentCommand::CommitSelect(state))) = &model.popup
        {
            state.filtered_count()
        } else {
            panic!("Expected CommitSelect popup");
        };

    // Get the first commit's hash prefix
    let hash_prefix = if let Some(PopupContent::Command(PopupContentCommand::CommitSelect(state))) =
        &model.popup
    {
        state.all_commits[0]
            .hash
            .as_ref()
            .unwrap()
            .chars()
            .take(3)
            .collect::<String>()
    } else {
        panic!("Expected CommitSelect popup");
    };

    // Input a few characters from the hash
    for c in hash_prefix.chars() {
        update(&mut model, Message::Select(SelectMessage::InputChar(c)));
    }

    // Verify filtering worked
    if let Some(PopupContent::Command(PopupContentCommand::CommitSelect(state))) = &model.popup {
        let filtered_count = state.filtered_count();
        assert!(
            filtered_count <= initial_count,
            "Filter should reduce or maintain count"
        );
        assert!(filtered_count >= 1, "Filter should find at least one match");
    } else {
        panic!("Expected CommitSelect popup");
    }
}

#[test]
fn test_commit_select_popup_filters_by_message() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("Add feature X");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"])
        .commit("Fix bug Y");

    test_repo
        .write_file_content("file3.txt", "content3")
        .stage_files(&["file3.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    // Show the commit select popup
    let _result = update(&mut model, Message::ShowFixupCommitSelect(FixupType::Fixup));

    // Filter by "feature"
    for c in "feature".chars() {
        update(&mut model, Message::Select(SelectMessage::InputChar(c)));
    }

    // Verify filtering found the "Add feature X" commit
    if let Some(PopupContent::Command(PopupContentCommand::CommitSelect(state))) = &model.popup {
        assert_eq!(state.filtered_count(), 1);
        assert_eq!(
            state.selected_commit().unwrap().message.as_deref(),
            Some("Add feature X")
        );
    } else {
        panic!("Expected CommitSelect popup");
    }
}

#[test]
fn test_commit_select_popup_navigation() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("Commit 1");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"])
        .commit("Commit 2");

    test_repo
        .write_file_content("file3.txt", "content3")
        .stage_files(&["file3.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    // Show the commit select popup
    let _result = update(&mut model, Message::ShowFixupCommitSelect(FixupType::Fixup));

    // Initially at first commit
    if let Some(PopupContent::Command(PopupContentCommand::CommitSelect(state))) = &model.popup {
        assert_eq!(state.selected_index, 0);
    } else {
        panic!("Expected CommitSelect popup");
    }

    // Move down
    update(&mut model, Message::Select(SelectMessage::MoveDown));

    if let Some(PopupContent::Command(PopupContentCommand::CommitSelect(state))) = &model.popup {
        assert_eq!(state.selected_index, 1);
    } else {
        panic!("Expected CommitSelect popup");
    }

    // Move up
    update(&mut model, Message::Select(SelectMessage::MoveUp));

    if let Some(PopupContent::Command(PopupContentCommand::CommitSelect(state))) = &model.popup {
        assert_eq!(state.selected_index, 0);
    } else {
        panic!("Expected CommitSelect popup");
    }
}

#[test]
fn test_commit_select_popup_confirm_returns_hash() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    // Show the commit select popup
    let _result = update(&mut model, Message::ShowFixupCommitSelect(FixupType::Fixup));

    // Get the expected hash
    let expected_hash =
        if let Some(PopupContent::Command(PopupContentCommand::CommitSelect(state))) = &model.popup
        {
            state.selected_commit_hash().unwrap().to_string()
        } else {
            panic!("Expected CommitSelect popup");
        };

    // Confirm selection
    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    // Verify popup is dismissed and correct message is returned
    assert!(model.popup.is_none());
    assert_eq!(
        result,
        Some(Message::FixupCommit(expected_hash, FixupType::Fixup))
    );
}

#[test]
fn test_commit_select_popup_state_new() {
    let commits = get_log_entries_for_test(&TestRepo::new());
    let state = CommitSelectPopupState::new("Test".to_string(), commits.clone());

    assert_eq!(state.title, "Test");
    assert_eq!(state.all_commits, commits);
    assert_eq!(state.filtered_indices.len(), commits.len());
    assert_eq!(state.input_text, "");
    assert_eq!(state.selected_index, 0);
}

#[test]
fn test_commit_select_popup_state_filter() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("Feature X");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"])
        .commit("Bug fix");

    let commits = get_log_entries_for_test(&test_repo);
    let mut state = CommitSelectPopupState::new("Test".to_string(), commits);

    // Filter by "feature"
    state.input_text = "feature".to_string();
    state.update_filter();

    assert_eq!(state.filtered_count(), 1);
    assert_eq!(
        state.selected_commit().unwrap().message.as_deref(),
        Some("Feature X")
    );
}
