use crossterm::event::KeyCode;
use magi::{
    git::{log::get_log_entries, test_repo::TestRepo},
    keys::handle_key,
    model::{
        Line, LineContent, ToastStyle, ViewMode,
        arguments::{Argument, Arguments, RevertArgument},
        popup::{PopupContent, PopupContentCommand, RevertPopupState},
    },
    msg::{LogType, Message, RevertCommand, update::update, util::is_external_command},
};

mod utils;
use utils::{create_model_from_test_repo, cursor_to_commit, find_commit_line, find_line, key};

// ── ShowRevertPopup — key binding ──────────────────────────────────────────────

#[test]
fn test_underscore_key_shows_revert_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let model = create_model_from_test_repo(&test_repo);

    // '_' is Shift+hyphen in many terminals; crossterm reports it as Char('_')
    let result = handle_key(key(KeyCode::Char('_')), &model);
    assert_eq!(result, Some(Message::ShowRevertPopup));
}

// ── ShowRevertPopup — cursor on commit ────────────────────────────────────────

#[test]
fn test_show_revert_popup_on_commit_line_selects_hash() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let commit_pos = find_commit_line(&model).expect("Expected a commit line");
    model.ui_model.cursor_position = commit_pos;

    let expected_hash = if let LineContent::Commit(info) = &model.ui_model.lines[commit_pos].content
    {
        info.hash.clone()
    } else {
        panic!("Not a commit line");
    };

    let result = update(&mut model, Message::ShowRevertPopup);

    assert_eq!(result, None);
    if let Some(PopupContent::Command(PopupContentCommand::Revert(state))) = &model.popup {
        assert!(!state.in_progress);
        assert_eq!(state.selected_commits, vec![expected_hash]);
    } else {
        panic!("Expected Revert popup");
    }
}

// ── ShowRevertPopup — cursor NOT on commit ────────────────────────────────────

#[test]
fn test_show_revert_popup_on_non_commit_line_has_empty_commits() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let non_commit_pos = find_line(&model, |c| {
        !matches!(c, LineContent::Commit(_) | LineContent::LogLine(_))
    })
    .expect("Expected a non-commit line");
    model.ui_model.cursor_position = non_commit_pos;

    update(&mut model, Message::ShowRevertPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Revert(state))) = &model.popup {
        assert!(!state.in_progress);
        assert!(state.selected_commits.is_empty());
    } else {
        panic!("Expected Revert popup");
    }
}

// ── ShowRevertPopup — visual selection ───────────────────────────────────────

#[test]
fn test_show_revert_popup_visual_selection_all_commits_collects_all_hashes() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo.commit_file("file2.txt", "content2", "Second commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Collect commit line positions
    let commit_positions: Vec<usize> = model
        .ui_model
        .lines
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            if matches!(&l.content, LineContent::Commit(_)) {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    assert!(
        commit_positions.len() >= 2,
        "Need at least 2 commits for this test"
    );

    let first = *commit_positions.first().unwrap();
    let last = *commit_positions.last().unwrap();

    // Set up visual selection spanning exactly the commit lines
    model.ui_model.visual_mode_anchor = Some(first);
    model.ui_model.cursor_position = last;

    update(&mut model, Message::ShowRevertPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Revert(state))) = &model.popup {
        assert!(!state.in_progress);
        assert_eq!(state.selected_commits.len(), commit_positions.len());
    } else {
        panic!("Expected Revert popup");
    }
}

#[test]
fn test_show_revert_popup_visual_selection_with_non_commit_gives_empty() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Set visual selection that spans from line 0 (likely a non-commit ref/header) to end
    model.ui_model.visual_mode_anchor = Some(0);
    model.ui_model.cursor_position = model.ui_model.lines.len().saturating_sub(1);

    update(&mut model, Message::ShowRevertPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Revert(state))) = &model.popup {
        assert!(!state.in_progress);
        // Selection contains non-commit lines → empty
        assert!(state.selected_commits.is_empty());
    } else {
        panic!("Expected Revert popup");
    }
}

// ── ShowRevertPopup — log view (LogLine) ──────────────────────────────────────

#[test]
fn test_show_revert_popup_on_log_line_selects_hash() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Populate the model with log-view lines (as ShowLog does)
    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let log_lines: Vec<Line> = get_log_entries(&repo, &LogType::Current, true, false)
        .unwrap()
        .into_iter()
        .map(|entry| Line {
            content: LineContent::LogLine(entry),
            section: None,
        })
        .collect();

    // Find a log line that has a hash
    let log_commit_pos = log_lines
        .iter()
        .position(|l| matches!(&l.content, LineContent::LogLine(e) if e.hash.is_some()))
        .expect("Expected at least one log line with a hash");

    let expected_hash = if let LineContent::LogLine(entry) = &log_lines[log_commit_pos].content {
        entry.hash.clone().unwrap()
    } else {
        panic!("Not a log line");
    };

    model.ui_model.lines = log_lines;
    model.view_mode = ViewMode::Log {
        log_type: LogType::Current,
        picking: false,
        graph: true,
        color: false,
    };
    model.ui_model.cursor_position = log_commit_pos;

    update(&mut model, Message::ShowRevertPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Revert(state))) = &model.popup {
        assert!(!state.in_progress);
        assert_eq!(state.selected_commits, vec![expected_hash]);
    } else {
        panic!("Expected Revert popup with selected commit from log view");
    }
}

// ── Revert popup keys ─────────────────────────────────────────────────────────

#[test]
fn test_underscore_in_revert_popup_with_commits_triggers_revert() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec!["abc1234".to_string()],
            mainline: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('_')), &model);
    assert_eq!(
        result,
        Some(Message::Revert(RevertCommand::Commits {
            hashes: vec!["abc1234".to_string()],
            mainline: None,
        }))
    );
}

#[test]
fn test_underscore_in_revert_popup_without_commits_does_nothing() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec![],
            mainline: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('_')), &model);
    assert_eq!(result, None);
}

#[test]
fn test_underscore_in_revert_popup_in_progress_triggers_continue() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: true,
            selected_commits: vec![],
            mainline: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('_')), &model);
    assert_eq!(result, Some(Message::Revert(RevertCommand::Continue)));
}

#[test]
fn test_s_in_revert_popup_in_progress_triggers_skip() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: true,
            selected_commits: vec![],
            mainline: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('s')), &model);
    assert_eq!(result, Some(Message::Revert(RevertCommand::Skip)));
}

#[test]
fn test_a_in_revert_popup_in_progress_triggers_abort() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: true,
            selected_commits: vec![],
            mainline: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('a')), &model);
    assert_eq!(result, Some(Message::Revert(RevertCommand::Abort)));
}

#[test]
fn test_s_in_revert_popup_not_in_progress_does_nothing() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec![],
            mainline: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('s')), &model);
    assert_eq!(result, None);
}

#[test]
fn test_q_dismisses_revert_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec![],
            mainline: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('q')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

#[test]
fn test_esc_dismisses_revert_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec![],
            mainline: None,
        },
    )));

    let result = handle_key(key(KeyCode::Esc), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── v key — revert no commit ──────────────────────────────────────────────────

#[test]
fn test_v_in_revert_popup_with_commits_triggers_no_commit_revert() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec!["abc1234".to_string()],
            mainline: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('v')), &model);
    assert_eq!(
        result,
        Some(Message::Revert(RevertCommand::NoCommit {
            hashes: vec!["abc1234".to_string()],
            mainline: None,
        }))
    );
}

#[test]
fn test_v_in_revert_popup_without_commits_does_nothing() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec![],
            mainline: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('v')), &model);
    assert_eq!(result, None);
}

#[test]
fn test_v_in_revert_popup_in_progress_does_nothing() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: true,
            selected_commits: vec![],
            mainline: None,
        },
    )));

    // 'v' is not a recognised key in in-progress mode
    let result = handle_key(key(KeyCode::Char('v')), &model);
    assert_eq!(result, None);
}

// ── -e / -E arguments ─────────────────────────────────────────────────────────

fn revert_popup_model(test_repo: &TestRepo) -> magi::model::Model {
    let mut model = create_model_from_test_repo(test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec![],
            mainline: None,
        },
    )));
    model
}

#[test]
fn test_show_revert_popup_enables_edit_argument_by_default() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    cursor_to_commit(&mut model);

    update(&mut model, Message::ShowRevertPopup);

    match &model.arguments {
        Some(Arguments::RevertArguments(args)) => {
            assert!(args.contains(&RevertArgument::Edit));
            assert!(!args.contains(&RevertArgument::NoEdit));
        }
        _ => panic!("Expected RevertArguments with --edit enabled by default"),
    }
}

#[test]
fn test_e_in_arg_mode_toggles_edit_argument() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = revert_popup_model(&test_repo);
    model.arg_mode = true;

    let result = handle_key(key(KeyCode::Char('e')), &model);
    assert_eq!(
        result,
        Some(Message::ToggleArgument(Argument::Revert(
            RevertArgument::Edit
        )))
    );
}

#[test]
fn test_toggle_edit_argument_disables_default() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    cursor_to_commit(&mut model);
    update(&mut model, Message::ShowRevertPopup);
    model.arg_mode = true;

    update(
        &mut model,
        Message::ToggleArgument(Argument::Revert(RevertArgument::Edit)),
    );

    match &model.arguments {
        Some(Arguments::RevertArguments(args)) => {
            assert!(!args.contains(&RevertArgument::Edit));
        }
        _ => panic!("Expected RevertArguments"),
    }
    assert!(!model.arg_mode);
}

#[test]
fn test_revert_commits_with_edit_returns_with_editor_command() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo.commit_file("file1.txt", "content2", "Second commit");

    let mut model = create_model_from_test_repo(&test_repo);
    cursor_to_commit(&mut model);
    update(&mut model, Message::ShowRevertPopup);

    let hash = test_repo.repo.head().unwrap().target().unwrap().to_string();
    let result = update(
        &mut model,
        Message::Revert(RevertCommand::Commits {
            hashes: vec![hash.clone()],
            mainline: None,
        }),
    );

    let expected = Message::Revert(RevertCommand::WithEditor {
        args: vec!["revert".to_string(), "--edit".to_string(), hash],
    });
    assert_eq!(result, Some(expected));
    // The editor command must run with the TUI suspended, not in a PTY
    assert!(is_external_command(&result.unwrap()));
    assert!(model.pty_state.is_none());
}

#[test]
fn test_revert_commits_with_no_edit_runs_in_pty() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo.commit_file("file1.txt", "content2", "Second commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::RevertArguments(
        [RevertArgument::NoEdit].into_iter().collect(),
    ));

    let hash = test_repo.repo.head().unwrap().target().unwrap().to_string();
    let result = update(
        &mut model,
        Message::Revert(RevertCommand::Commits {
            hashes: vec![hash],
            mainline: None,
        }),
    );

    assert_eq!(result, None);
    assert!(model.pty_state.is_some());
}

#[test]
fn test_revert_commits_with_both_edit_flags_favors_no_edit() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo.commit_file("file1.txt", "content2", "Second commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::RevertArguments(
        [RevertArgument::Edit, RevertArgument::NoEdit]
            .into_iter()
            .collect(),
    ));

    let hash = test_repo.repo.head().unwrap().target().unwrap().to_string();
    let result = update(
        &mut model,
        Message::Revert(RevertCommand::Commits {
            hashes: vec![hash],
            mainline: None,
        }),
    );

    // --no-edit is passed last, so git skips the editor: run in a PTY
    assert_eq!(result, None);
    assert!(model.pty_state.is_some());
}

#[test]
fn test_revert_with_editor_creates_revert_commit() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "one", "First commit");
    test_repo.commit_file("file1.txt", "two", "Second commit");
    // Use a no-op editor so the default revert message is kept
    test_repo
        .repo
        .config()
        .unwrap()
        .set_str("core.editor", "true")
        .unwrap();

    let mut model = create_model_from_test_repo(&test_repo);

    let hash = test_repo.repo.head().unwrap().target().unwrap().to_string();
    let result = update(
        &mut model,
        Message::Revert(RevertCommand::WithEditor {
            args: vec!["revert".to_string(), "--edit".to_string(), hash],
        }),
    );

    assert_eq!(result, Some(Message::Refresh));
    let toast = model.toast.expect("Expected a toast after reverting");
    assert_eq!(toast.style, ToastStyle::Success);
    assert!(
        toast
            .message
            .starts_with("Revert: Revert \"Second commit\""),
        "unexpected toast message: {}",
        toast.message
    );

    let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
    let summary = head.summary().unwrap().unwrap();
    assert!(summary.starts_with("Revert \"Second commit\""));
    let content = std::fs::read_to_string(test_repo.repo_path().join("file1.txt")).unwrap();
    assert_eq!(content, "one");
}
