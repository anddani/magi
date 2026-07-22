use crossterm::event::KeyCode;
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::{LineContent, ViewMode, popup::PopupContent, popup::PopupContentCommand},
    msg::{LogType, Message, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, key};

// ── Stash popup keys ───────────────────────────────────────────────────────────

#[test]
fn test_l_in_stash_popup_shows_stash_list() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = handle_key(key(KeyCode::Char('l')), &model);
    assert_eq!(result, Some(Message::ShowLog(LogType::Stashes)));
}

// ── Stash list execution ───────────────────────────────────────────────────────

#[test]
fn test_show_stash_list_displays_stashes() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "first")
        .create_stash("First stash");
    test_repo
        .write_file_content("file.txt", "second")
        .create_stash("Second stash");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = update(&mut model, Message::ShowLog(LogType::Stashes));

    assert_eq!(result, None);
    assert_eq!(model.popup, None);
    assert!(matches!(
        model.view_mode,
        ViewMode::Log {
            log_type: LogType::Stashes,
            ..
        }
    ));

    let messages: Vec<String> = model
        .ui_model
        .lines
        .iter()
        .filter_map(|line| match &line.content {
            LineContent::LogLine(entry) => entry.message.clone(),
            _ => None,
        })
        .collect();
    assert_eq!(messages.len(), 2);
    assert!(messages[0].contains("Second stash"));
    assert!(messages[1].contains("First stash"));
}

#[test]
fn test_show_stash_list_without_stashes_shows_empty_view() {
    let test_repo = TestRepo::new();

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Stash));

    let result = update(&mut model, Message::ShowLog(LogType::Stashes));

    assert_eq!(result, None);
    assert_eq!(model.popup, None);
    assert!(matches!(
        model.view_mode,
        ViewMode::Log {
            log_type: LogType::Stashes,
            ..
        }
    ));
    assert!(model.ui_model.lines.is_empty());
}
