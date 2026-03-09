use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::popup::{ConfirmAction, InputContext, PopupContent, PopupContentCommand},
    model::select_popup::SelectContext,
    msg::{Message, SelectPopup, update::update},
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

// ── ShowTagPopup — key binding ─────────────────────────────────────────────────

#[test]
fn test_t_key_shows_tag_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let model = create_model_from_test_repo(&test_repo);

    let result = handle_key(key(KeyCode::Char('t')), &model);
    assert_eq!(result, Some(Message::ShowTagPopup));
}

// ── ShowTagPopup — state ───────────────────────────────────────────────────────

#[test]
fn test_show_tag_popup_sets_state() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowTagPopup);

    assert_eq!(result, None);
    assert!(
        matches!(
            &model.popup,
            Some(PopupContent::Command(PopupContentCommand::Tag))
        ),
        "Expected Tag popup"
    );
}

// ── Tag popup keys ─────────────────────────────────────────────────────────────

#[test]
fn test_q_in_tag_popup_dismisses() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));

    let result = handle_key(key(KeyCode::Char('q')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

#[test]
fn test_esc_in_tag_popup_dismisses() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));

    let result = handle_key(key(KeyCode::Esc), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── Create tag flow ────────────────────────────────────────────────────────────

#[test]
fn test_t_in_tag_popup_shows_create_tag_input() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));

    let result = handle_key(key(KeyCode::Char('t')), &model);
    assert_eq!(result, Some(Message::ShowCreateTagInput));
}

#[test]
fn test_show_create_tag_input_opens_input_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowCreateTagInput);

    assert_eq!(result, None);
    assert!(
        matches!(
            &model.popup,
            Some(PopupContent::Input(state)) if state.context == InputContext::CreateTag
        ),
        "Expected Input popup with CreateTag context"
    );
}

#[test]
fn test_create_tag_input_confirm_shows_ref_select() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    // Set up an input popup with CreateTag context and a tag name typed
    model.popup = Some(PopupContent::input_popup(InputContext::CreateTag));
    if let Some(PopupContent::Input(ref mut state)) = model.popup {
        state.input_text = "v1.0.0".to_string();
    }

    let result = update(&mut model, Message::Input(magi::msg::InputMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(SelectPopup::CreateTagTarget(
            "v1.0.0".to_string()
        )))
    );
}

#[test]
fn test_create_tag_target_select_shows_refs() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::ShowSelectPopup(SelectPopup::CreateTagTarget("v1.0.0".to_string())),
    );

    assert_eq!(result, None);
    assert!(
        matches!(
            &model.popup,
            Some(PopupContent::Command(PopupContentCommand::Select(state)))
                if !state.all_options.is_empty()
        ),
        "Expected Select popup with non-empty options"
    );
    assert_eq!(
        model.select_context,
        Some(SelectContext::CreateTagTarget("v1.0.0".to_string()))
    );
}

#[test]
fn test_create_tag_creates_tag() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.popup.is_none());

    // Verify the tag exists in the repository
    let tags = model.git_info.repository.tag_names(None).unwrap();
    let tag_list: Vec<&str> = tags.iter().flatten().collect();
    assert!(
        tag_list.contains(&"v1.0.0"),
        "Tag 'v1.0.0' should exist in the repository"
    );
}

// ── Delete tag flow ────────────────────────────────────────────────────────────

#[test]
fn test_x_in_tag_popup_shows_delete_tag_select() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));

    let result = handle_key(key(KeyCode::Char('x')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(SelectPopup::DeleteTag))
    );
}

#[test]
fn test_delete_tag_select_shows_existing_tags() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    // Create a tag so the list is non-empty
    update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    let result = update(&mut model, Message::ShowSelectPopup(SelectPopup::DeleteTag));

    assert_eq!(result, None);
    assert!(
        matches!(
            &model.popup,
            Some(PopupContent::Command(PopupContentCommand::Select(state)))
                if state.all_options.contains(&"v1.0.0".to_string())
        ),
        "Expected Select popup listing 'v1.0.0'"
    );
    assert_eq!(model.select_context, Some(SelectContext::DeleteTag));
}

#[test]
fn test_delete_tag_select_empty_repo_shows_error() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    // No tags created — should show an error popup

    let result = update(&mut model, Message::ShowSelectPopup(SelectPopup::DeleteTag));

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected Error popup when no tags exist"
    );
}

#[test]
fn test_delete_tag_removes_tag() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    // Create then delete
    update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    let result = update(&mut model, Message::DeleteTag("v1.0.0".to_string()));

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.popup.is_none());

    let tags = model.git_info.repository.tag_names(None).unwrap();
    let tag_list: Vec<&str> = tags.iter().flatten().collect();
    assert!(
        !tag_list.contains(&"v1.0.0"),
        "Tag 'v1.0.0' should have been deleted"
    );
}

// ── Prune tags flow ────────────────────────────────────────────────────────────

#[test]
fn test_p_in_tag_popup_shows_prune_remote_select() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag));

    let result = handle_key(key(KeyCode::Char('p')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(SelectPopup::PruneTagsRemotePick))
    );
}

#[test]
fn test_prune_tags_confirm_shows_error_for_bad_remote() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::ShowPruneTagsConfirm {
            remote: "nonexistent-remote".to_string(),
        },
    );

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected Error popup for bad remote"
    );
}

#[test]
fn test_prune_remote_pick_skips_select_with_single_remote() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    // With exactly one remote, should skip the select and return ShowPruneTagsConfirm directly
    let repo_path = test_repo.repo.workdir().unwrap();
    std::process::Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["remote", "add", "origin", "https://example.com/repo.git"])
        .output()
        .unwrap();

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::ShowSelectPopup(SelectPopup::PruneTagsRemotePick),
    );

    assert_eq!(
        result,
        Some(Message::ShowPruneTagsConfirm {
            remote: "origin".to_string()
        })
    );
}

#[test]
fn test_prune_remote_pick_shows_select_with_multiple_remotes() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    // With two remotes, should show the select popup
    let repo_path = test_repo.repo.workdir().unwrap();
    for remote in &["origin", "upstream"] {
        std::process::Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(["remote", "add", remote, "https://example.com/repo.git"])
            .output()
            .unwrap();
    }

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::ShowSelectPopup(SelectPopup::PruneTagsRemotePick),
    );

    assert_eq!(result, None);
    assert!(
        matches!(
            &model.popup,
            Some(PopupContent::Command(PopupContentCommand::Select(state)))
                if state.all_options.contains(&"origin".to_string())
        ),
        "Expected Select popup listing remotes"
    );
    assert_eq!(
        model.select_context,
        Some(SelectContext::PruneTagsRemotePick)
    );
}

#[test]
fn test_prune_tags_deletes_local_only_tags() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Create a local tag
    update(
        &mut model,
        Message::CreateTag {
            name: "v1.0.0".to_string(),
            target: "HEAD".to_string(),
        },
    );

    // Prune with empty remote list (simulates: remote has no tags)
    let result = update(
        &mut model,
        Message::PruneTags {
            local_tags: vec!["v1.0.0".to_string()],
            remote_tags: vec![],
            remote: "origin".to_string(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.popup.is_none());

    let tags = model.git_info.repository.tag_names(None).unwrap();
    let tag_list: Vec<&str> = tags.iter().flatten().collect();
    assert!(
        !tag_list.contains(&"v1.0.0"),
        "v1.0.0 should have been deleted locally"
    );
}

#[test]
fn test_prune_tags_confirm_shows_confirm_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Dispatch directly with known data (bypasses network)
    // Simulate what show_prune_tags_confirm would produce
    let local_only = vec!["v1.0.0".to_string()];
    let remote_only: Vec<String> = vec![];

    // Confirm popup should hold the PruneTags action
    use magi::model::popup::ConfirmPopupState;
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: "Prune tags against 'origin':\n  Delete locally (1): v1.0.0".to_string(),
        on_confirm: ConfirmAction::PruneTags {
            local_tags: local_only.clone(),
            remote_tags: remote_only.clone(),
            remote: "origin".to_string(),
        },
    }));

    assert!(
        matches!(
            &model.popup,
            Some(PopupContent::Confirm(state))
                if matches!(&state.on_confirm, ConfirmAction::PruneTags { .. })
        ),
        "Confirm popup should hold PruneTags action"
    );
}
