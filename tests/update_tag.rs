use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::popup::{InputContext, PopupContent, PopupContentCommand, TagPopupState},
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
            Some(PopupContent::Command(PopupContentCommand::Tag(
                TagPopupState {}
            )))
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
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag(
        TagPopupState {},
    )));

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
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag(
        TagPopupState {},
    )));

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
    model.popup = Some(PopupContent::Command(PopupContentCommand::Tag(
        TagPopupState {},
    )));

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
