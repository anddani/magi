use crossterm::event::KeyCode;
use magi::{
    git::{
        log::get_log_entries,
        rebase::{RebaseAction, rebase_in_progress},
        test_repo::TestRepo,
    },
    keys::handle_key,
    model::{
        Line, LineContent, Model, SectionType, ViewMode,
        popup::{ConfirmAction, PopupContent, PopupContentCommand, RebasePopupState},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::{
        CommitSelect, LogType, Message, OptionsSource, RebaseCommand, RebaseTodoMessage,
        SearchMessage, SelectMessage, ShowSelectPopupConfig, update::update,
    },
};

mod utils;
use utils::{
    create_model_from_test_repo, cursor_to_commit, expect_confirm_popup, find_commit_line,
    find_line, key,
};

// ── ShowRebasePopup ────────────────────────────────────────────────────────────

#[test]
fn test_show_rebase_popup_sets_popup_with_branch_name() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowRebasePopup);

    assert_eq!(result, None);
    assert!(matches!(
        &model.popup,
        Some(PopupContent::Command(PopupContentCommand::Rebase(
            RebasePopupState { branch, .. }
        ))) if !branch.is_empty()
    ));
}

#[test]
fn test_show_rebase_popup_captures_current_branch() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let expected_branch = model.git_info.current_branch().unwrap_or_default();

    update(&mut model, Message::ShowRebasePopup);

    if let Some(PopupContent::Command(PopupContentCommand::Rebase(state))) = &model.popup {
        assert_eq!(state.branch, expected_branch);
    } else {
        panic!("Expected Rebase popup");
    }
}

// ── RebaseElsewhere - cursor on commit ────────────────────────────────────────

#[test]
fn test_rebase_elsewhere_on_commit_shows_confirmation() {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("file1.txt", "content1", "First commit")
        .commit_file("file2.txt", "content2", "Second commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Place cursor on a commit line
    cursor_to_commit(&mut model);

    let result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::RebaseElsewhere),
    );

    assert_eq!(result, None);
    assert!(matches!(
        &model.popup,
        Some(PopupContent::Confirm(state))
            if matches!(&state.on_confirm, ConfirmAction::RebaseElsewhere(_))
    ));
}

#[test]
fn test_rebase_elsewhere_confirmation_message_contains_hash() {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("file1.txt", "content1", "First commit")
        .commit_file("file2.txt", "content2", "Second commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let commit_line = find_commit_line(&model).expect("Expected a commit line in the model");
    model.ui_model.cursor_position = commit_line;

    // Get the expected hash
    let expected_hash =
        if let LineContent::Commit(info) = &model.ui_model.lines[commit_line].content {
            info.hash.clone()
        } else {
            panic!("Not a commit line");
        };

    update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::RebaseElsewhere),
    );

    let state = expect_confirm_popup(&model);
    assert!(state.message.contains(&expected_hash));
    assert!(matches!(
        &state.on_confirm,
        ConfirmAction::RebaseElsewhere(hash) if *hash == expected_hash
    ));
}

// ── RebaseElsewhere - cursor NOT on commit ────────────────────────────────────

#[test]
fn test_rebase_elsewhere_not_on_commit_shows_log_pick_view() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Find a non-commit line (section header, empty line, etc.)
    let non_commit_pos = find_line(&model, |c| {
        !matches!(c, LineContent::Commit(_) | LineContent::LogLine(_))
    })
    .expect("Expected at least one non-commit line");

    model.ui_model.cursor_position = non_commit_pos;

    let result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::RebaseElsewhere),
    );

    assert_eq!(result, None);
    assert!(model.popup.is_none(), "No popup expected — using log view");
    assert!(
        matches!(
            model.view_mode,
            ViewMode::Log {
                log_type: LogType::AllReferences,
                picking: true,
                ..
            }
        ),
        "Expected AllReferences log pick view"
    );
    assert_eq!(model.log_pick_on_select, Some(OnSelect::RebaseElsewhere));
}

// ── Keys ──────────────────────────────────────────────────────────────────────

#[test]
fn test_r_key_shows_rebase_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let model = create_model_from_test_repo(&test_repo);

    let result = handle_key(key(KeyCode::Char('r')), &model);
    assert_eq!(result, Some(Message::ShowRebasePopup));
}

#[test]
fn test_e_in_rebase_popup_shows_rebase_elsewhere() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('e')), &model);
    assert_eq!(
        result,
        Some(Message::ShowCommitSelect(CommitSelect::RebaseElsewhere))
    );
}

#[test]
fn test_p_in_rebase_popup_with_push_remote_rebases_onto_push_remote() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: Some("origin".to_string()),
            sole_remote: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('p')), &model);
    assert_eq!(
        result,
        Some(Message::Rebase(RebaseCommand::OntoPushRemote(
            "origin".to_string()
        )))
    );
}

#[test]
fn test_p_in_rebase_popup_with_sole_remote_rebases_directly() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: Some("origin".to_string()),
        },
    )));

    let result = handle_key(key(KeyCode::Char('p')), &model);
    assert_eq!(
        result,
        Some(Message::Rebase(RebaseCommand::OntoPushRemote(
            "origin".to_string()
        )))
    );
}

#[test]
fn test_p_in_rebase_popup_without_push_remote_shows_select() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('p')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Rebase onto push remote".to_string(),
            source: OptionsSource::Remotes,
            on_select: OnSelect::RebasePushRemote,
        }))
    );
}

// ── Select confirm → RebasePushRemote ─────────────────────────────────────────

#[test]
fn test_select_confirm_rebase_push_remote_routes_to_rebase_command() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Rebase onto push remote".to_string(),
            vec!["origin".to_string(), "upstream".to_string()],
            OnSelect::RebasePushRemote,
        ),
    )));

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::Rebase(RebaseCommand::OntoPushRemote(
            "origin".to_string()
        )))
    );
}

#[test]
fn test_esc_dismisses_rebase_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        },
    )));

    let result = handle_key(key(KeyCode::Esc), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

#[test]
fn test_q_dismisses_rebase_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('q')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── Select confirm → RebaseElsewhere ─────────────────────────────────────────

#[test]
fn test_select_confirm_rebase_elsewhere_context_returns_rebase_message() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let mut commits = get_log_entries(&repo, &LogType::Current, true, false).unwrap();
    commits.retain(|e| e.is_commit());

    let mut model = create_model_from_test_repo(&test_repo);

    let expected_hash = commits[0].hash.as_ref().unwrap().clone();

    // Set up model in log pick mode (new approach: no popup, log view)
    model.ui_model.lines = commits
        .into_iter()
        .map(|entry| Line {
            content: LineContent::LogLine(entry),
            section: None,
        })
        .collect();
    model.ui_model.cursor_position = 0;
    model.view_mode = ViewMode::Log {
        log_type: LogType::AllReferences,
        picking: true,
        graph: true,
        color: false,
    };
    model.log_pick_on_select = Some(OnSelect::RebaseElsewhere);

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::Rebase(RebaseCommand::Elsewhere(
            expected_hash.clone()
        )))
    );
    assert_eq!(model.view_mode, ViewMode::Status);
    assert!(model.log_pick_on_select.is_none());
}

// ── ShowRebasePopup — not in_progress (normal repo) ──────────────────────────

#[test]
fn test_show_rebase_popup_not_in_progress_for_clean_repo() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(&mut model, Message::ShowRebasePopup);

    if let Some(PopupContent::Command(PopupContentCommand::Rebase(state))) = &model.popup {
        assert!(
            !state.in_progress,
            "Expected in_progress = false for a clean repo"
        );
    } else {
        panic!("Expected Rebase popup");
    }
}

// ── ShowRebasePopup — in_progress when rebase-merge dir exists ────────────────

#[test]
fn test_show_rebase_popup_in_progress_when_rebase_merge_dir_exists() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Simulate a rebase in progress
    test_repo.with_rebase_in_progress();

    update(&mut model, Message::ShowRebasePopup);

    if let Some(PopupContent::Command(PopupContentCommand::Rebase(state))) = &model.popup {
        assert!(
            state.in_progress,
            "Expected in_progress = true when rebase-merge dir exists"
        );
    } else {
        panic!("Expected Rebase popup");
    }
}

// ── rebase_in_progress detection ─────────────────────────────────────────────

#[test]
fn test_rebase_in_progress_false_for_clean_repo() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let model = create_model_from_test_repo(&test_repo);
    assert!(!rebase_in_progress(&model.workdir));
}

// ── Keys in rebase popup when in_progress ─────────────────────────────────────

#[test]
fn test_r_key_in_in_progress_popup_continues_rebase() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: true,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('r')), &model);
    assert_eq!(result, Some(Message::Rebase(RebaseCommand::Continue)));
}

#[test]
fn test_s_key_in_in_progress_popup_skips_rebase() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: true,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('s')), &model);
    assert_eq!(result, Some(Message::Rebase(RebaseCommand::Skip)));
}

#[test]
fn test_a_key_in_in_progress_popup_aborts_rebase() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: true,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        },
    )));

    let result = handle_key(key(KeyCode::Char('a')), &model);
    assert_eq!(result, Some(Message::Rebase(RebaseCommand::Abort)));
}

#[test]
fn test_e_key_has_no_effect_in_in_progress_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: true,
            upstream: None,
            push_remote: None,
            sole_remote: None,
        },
    )));

    // 'e' should not trigger elsewhere in in_progress mode
    let result = handle_key(key(KeyCode::Char('e')), &model);
    assert_eq!(result, None);
}

// ── Interactive rebase — popup key and base selection ─────────────────────────

fn rebase_popup(in_progress: bool) -> PopupContent {
    PopupContent::Command(PopupContentCommand::Rebase(RebasePopupState {
        branch: "main".to_string(),
        in_progress,
        upstream: None,
        push_remote: None,
        sole_remote: None,
    }))
}

#[test]
fn test_i_in_rebase_popup_shows_rebase_interactive_select() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(rebase_popup(false));

    let result = handle_key(key(KeyCode::Char('i')), &model);
    assert_eq!(
        result,
        Some(Message::ShowCommitSelect(CommitSelect::RebaseInteractive))
    );
}

#[test]
fn test_i_key_has_no_effect_in_in_progress_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(rebase_popup(true));

    let result = handle_key(key(KeyCode::Char('i')), &model);
    assert_eq!(result, None);
}

// ── Subset rebase — popup key and two-step selection ──────────────────────────

#[test]
fn test_s_in_rebase_popup_shows_subset_onto_select() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(rebase_popup(false));

    let result = handle_key(key(KeyCode::Char('s')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Rebase subset onto".to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::RebaseSubsetOnto,
        }))
    );
}

#[test]
fn test_select_confirm_rebase_subset_onto_routes_to_commit_select() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Rebase subset onto".to_string(),
            vec!["main".to_string(), "origin/main".to_string()],
            OnSelect::RebaseSubsetOnto,
        ),
    )));

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::ShowCommitSelect(CommitSelect::RebaseSubset {
            newbase: "main".to_string(),
        }))
    );
}

#[test]
fn test_rebase_subset_shows_current_log_pick_view() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::RebaseSubset {
            newbase: "main".to_string(),
        }),
    );

    assert_eq!(result, None);
    assert!(
        matches!(
            model.view_mode,
            ViewMode::Log {
                log_type: LogType::Current,
                picking: true,
                ..
            }
        ),
        "Expected current-branch log pick view"
    );
    assert_eq!(
        model.log_pick_on_select,
        Some(OnSelect::RebaseSubsetStart {
            newbase: "main".to_string(),
        })
    );
}

#[test]
fn test_select_confirm_rebase_subset_start_returns_rebase_message() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let mut commits = get_log_entries(&repo, &LogType::Current, true, false).unwrap();
    commits.retain(|e| e.is_commit());

    let mut model = create_model_from_test_repo(&test_repo);

    let expected_hash = commits[0].hash.as_ref().unwrap().clone();

    model.ui_model.lines = commits
        .into_iter()
        .map(|entry| Line {
            content: LineContent::LogLine(entry),
            section: None,
        })
        .collect();
    model.ui_model.cursor_position = 0;
    model.view_mode = ViewMode::Log {
        log_type: LogType::Current,
        picking: true,
        graph: true,
        color: false,
    };
    model.log_pick_on_select = Some(OnSelect::RebaseSubsetStart {
        newbase: "origin/main".to_string(),
    });

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::Rebase(RebaseCommand::Subset {
            newbase: "origin/main".to_string(),
            start: expected_hash,
        }))
    );
    assert_eq!(model.view_mode, ViewMode::Status);
    assert!(model.log_pick_on_select.is_none());
}

#[test]
fn test_rebase_interactive_on_commit_opens_todo_editor_directly() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    cursor_to_commit(&mut model);

    let commit_line = find_commit_line(&model).unwrap();
    let expected_hash =
        if let LineContent::Commit(info) = &model.ui_model.lines[commit_line].content {
            info.hash.clone()
        } else {
            panic!("Not a commit line");
        };

    let result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::RebaseInteractive),
    );

    // No confirmation — the todo editor opens for the commit directly
    assert_eq!(result, Some(Message::ShowRebaseTodo(expected_hash)));
    assert!(model.popup.is_none());
}

#[test]
fn test_rebase_interactive_not_on_commit_shows_current_log_pick_view() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let non_commit_pos = find_line(&model, |c| {
        !matches!(c, LineContent::Commit(_) | LineContent::LogLine(_))
    })
    .expect("Expected at least one non-commit line");
    model.ui_model.cursor_position = non_commit_pos;

    let result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::RebaseInteractive),
    );

    assert_eq!(result, None);
    assert!(
        matches!(
            model.view_mode,
            ViewMode::Log {
                log_type: LogType::Current,
                picking: true,
                ..
            }
        ),
        "Expected current-branch log pick view"
    );
    assert_eq!(model.log_pick_on_select, Some(OnSelect::RebaseInteractive));
}

// ── Interactive rebase — todo editor state ────────────────────────────────────

/// Creates a repo with "Initial commit", "Commit A", "Commit B" and opens the
/// todo editor with Commit A as base (so the todo lists A then B).
fn model_with_todo_editor() -> (TestRepo, Model) {
    let test_repo = TestRepo::new();
    test_repo.commit_file("a.txt", "a", "Commit A");
    let base = test_repo.head_hash();
    test_repo.commit_file("b.txt", "b", "Commit B");

    let mut model = create_model_from_test_repo(&test_repo);
    update(&mut model, Message::ShowRebaseTodo(base));
    (test_repo, model)
}

fn todo_actions(model: &Model) -> Vec<RebaseAction> {
    model
        .rebase_todo
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .map(|e| e.action)
        .collect()
}

#[test]
fn test_show_rebase_todo_opens_editor() {
    let (_test_repo, model) = model_with_todo_editor();

    assert_eq!(model.view_mode, ViewMode::RebaseTodo);
    assert_eq!(model.ui_model.cursor_position, 0);
    assert!(matches!(
        &model.ui_model.lines[0].content,
        LineContent::RebaseTodoLine(entry) if entry.message == "Commit A"
    ));
    assert!(matches!(
        &model.ui_model.lines[1].content,
        LineContent::RebaseTodoLine(entry) if entry.message == "Commit B"
    ));
    assert_eq!(todo_actions(&model), vec![RebaseAction::Pick; 2]);

    // Keybinding hints follow the entries, one line per key
    assert!(matches!(
        &model.ui_model.lines[2].content,
        LineContent::EmptyLine
    ));
    let hint_keys: Vec<&str> = model
        .ui_model
        .lines
        .iter()
        .filter_map(|l| match &l.content {
            LineContent::RebaseTodoHint { key, .. } => Some(*key),
            _ => None,
        })
        .collect();
    assert_eq!(
        hint_keys,
        vec!["p", "r", "e", "s", "f", "d", "K/J", "u", "RET", "R", "q"]
    );

    // Confirm rebase and Abort are grouped, separated by an empty line
    let last = model.ui_model.lines.len() - 1;
    assert!(matches!(
        &model.ui_model.lines[last].content,
        LineContent::RebaseTodoHint { key: "q", .. }
    ));
    assert!(matches!(
        &model.ui_model.lines[last - 1].content,
        LineContent::RebaseTodoHint { key: "R", .. }
    ));
    assert!(matches!(
        &model.ui_model.lines[last - 2].content,
        LineContent::EmptyLine
    ));
}

#[test]
fn test_rebase_todo_actions_on_hint_lines_are_inert() {
    let (_test_repo, mut model) = model_with_todo_editor();

    // Move the cursor past the entries onto the hint block
    model.ui_model.cursor_position = 3;
    update(
        &mut model,
        Message::RebaseTodo(RebaseTodoMessage::SetAction(RebaseAction::Drop)),
    );
    update(
        &mut model,
        Message::RebaseTodo(RebaseTodoMessage::MoveEntryUp),
    );

    assert_eq!(todo_actions(&model), vec![RebaseAction::Pick; 2]);
    assert!(model.toast.is_none());
    assert_eq!(model.ui_model.cursor_position, 3);
}

#[test]
fn test_rebase_todo_set_action_updates_line_and_advances_cursor() {
    let (_test_repo, mut model) = model_with_todo_editor();

    update(
        &mut model,
        Message::RebaseTodo(RebaseTodoMessage::SetAction(RebaseAction::Reword)),
    );

    assert_eq!(
        todo_actions(&model),
        vec![RebaseAction::Reword, RebaseAction::Pick]
    );
    assert!(matches!(
        &model.ui_model.lines[0].content,
        LineContent::RebaseTodoLine(entry) if entry.action == RebaseAction::Reword
    ));
    // Auto-advanced to the next line
    assert_eq!(model.ui_model.cursor_position, 1);
}

#[test]
fn test_rebase_todo_squash_on_first_line_is_rejected_with_toast() {
    let (_test_repo, mut model) = model_with_todo_editor();

    update(
        &mut model,
        Message::RebaseTodo(RebaseTodoMessage::SetAction(RebaseAction::Squash)),
    );

    assert_eq!(todo_actions(&model), vec![RebaseAction::Pick; 2]);
    assert_eq!(model.ui_model.cursor_position, 0);
    assert!(model.toast.is_some(), "Expected a warning toast");
}

#[test]
fn test_rebase_todo_move_entry_down_swaps_lines_and_follows_cursor() {
    let (_test_repo, mut model) = model_with_todo_editor();

    update(
        &mut model,
        Message::RebaseTodo(RebaseTodoMessage::MoveEntryDown),
    );

    assert!(matches!(
        &model.ui_model.lines[0].content,
        LineContent::RebaseTodoLine(entry) if entry.message == "Commit B"
    ));
    assert!(matches!(
        &model.ui_model.lines[1].content,
        LineContent::RebaseTodoLine(entry) if entry.message == "Commit A"
    ));
    assert_eq!(model.ui_model.cursor_position, 1);
}

#[test]
fn test_rebase_todo_undo_restores_previous_state() {
    let (_test_repo, mut model) = model_with_todo_editor();

    update(
        &mut model,
        Message::RebaseTodo(RebaseTodoMessage::SetAction(RebaseAction::Drop)),
    );
    assert_eq!(
        todo_actions(&model),
        vec![RebaseAction::Drop, RebaseAction::Pick]
    );

    update(&mut model, Message::RebaseTodo(RebaseTodoMessage::Undo));
    assert_eq!(todo_actions(&model), vec![RebaseAction::Pick; 2]);
}

#[test]
fn test_rebase_todo_abort_returns_to_status_without_rebasing() {
    let (test_repo, mut model) = model_with_todo_editor();

    let result = update(&mut model, Message::RebaseTodo(RebaseTodoMessage::Abort));

    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(model.view_mode, ViewMode::Status);
    assert!(model.rebase_todo.is_none());
    assert!(!rebase_in_progress(test_repo.repo_path()));
}

// ── Interactive rebase — todo editor keys ─────────────────────────────────────

#[test]
fn test_keys_in_rebase_todo_view() {
    let (_test_repo, model) = model_with_todo_editor();

    let cases = [
        (
            KeyCode::Char('p'),
            RebaseTodoMessage::SetAction(RebaseAction::Pick),
        ),
        (
            KeyCode::Char('r'),
            RebaseTodoMessage::SetAction(RebaseAction::Reword),
        ),
        (
            KeyCode::Char('e'),
            RebaseTodoMessage::SetAction(RebaseAction::Edit),
        ),
        (
            KeyCode::Char('s'),
            RebaseTodoMessage::SetAction(RebaseAction::Squash),
        ),
        (
            KeyCode::Char('f'),
            RebaseTodoMessage::SetAction(RebaseAction::Fixup),
        ),
        (
            KeyCode::Char('d'),
            RebaseTodoMessage::SetAction(RebaseAction::Drop),
        ),
        (KeyCode::Char('u'), RebaseTodoMessage::Undo),
        (KeyCode::Char('q'), RebaseTodoMessage::Abort),
        (KeyCode::Esc, RebaseTodoMessage::Abort),
    ];
    for (code, expected) in cases {
        assert_eq!(
            handle_key(key(code), &model),
            Some(Message::RebaseTodo(expected)),
            "key {:?}",
            code
        );
    }

    assert_eq!(
        handle_key(utils::shift_key(KeyCode::Char('K')), &model),
        Some(Message::RebaseTodo(RebaseTodoMessage::MoveEntryUp))
    );
    assert_eq!(
        handle_key(utils::shift_key(KeyCode::Char('J')), &model),
        Some(Message::RebaseTodo(RebaseTodoMessage::MoveEntryDown))
    );
    assert_eq!(
        handle_key(key(KeyCode::Enter), &model),
        Some(Message::ShowPreview)
    );
    assert_eq!(
        handle_key(utils::shift_key(KeyCode::Char('R')), &model),
        Some(Message::Rebase(RebaseCommand::ExecuteInteractive))
    );
    assert_eq!(
        handle_key(key(KeyCode::Char(':')), &model),
        Some(Message::RebaseTodo(RebaseTodoMessage::CommandStart))
    );
    // Navigation still works
    assert_eq!(
        handle_key(key(KeyCode::Char('j')), &model),
        Some(Message::Navigation(magi::msg::NavigationAction::MoveDown))
    );
    // Search works like in other views
    assert_eq!(
        handle_key(key(KeyCode::Char('/')), &model),
        Some(Message::EnterSearchMode)
    );
    assert_eq!(
        handle_key(key(KeyCode::Char('n')), &model),
        Some(Message::Search(magi::msg::SearchMessage::Next))
    );
    assert_eq!(
        handle_key(utils::shift_key(KeyCode::Char('N')), &model),
        Some(Message::Search(magi::msg::SearchMessage::Prev))
    );
    // Keys that would open popups in the status view are inert here
    assert_eq!(handle_key(key(KeyCode::Char('c')), &model), None);
}

// ── Interactive rebase — search in the todo editor ───────────────────────────

#[test]
fn test_rebase_todo_search_moves_cursor_to_match() {
    let (_test_repo, mut model) = model_with_todo_editor();

    update(&mut model, Message::EnterSearchMode);
    for c in "Commit B".chars() {
        update(
            &mut model,
            Message::Search(SearchMessage::Edit(magi::model::EditOp::Insert(c))),
        );
    }
    update(&mut model, Message::Search(SearchMessage::Confirm));

    assert_eq!(model.view_mode, ViewMode::RebaseTodo);
    assert_eq!(model.ui_model.cursor_position, 1);
    assert_eq!(model.ui_model.search_query.as_str(), "Commit B");

    // Cancelling the search keeps the editor state intact
    update(&mut model, Message::Search(SearchMessage::Cancel));
    assert!(model.ui_model.search_query.is_empty());
    assert!(model.rebase_todo.is_some());
}

// ── Interactive rebase — vim-style command line ──────────────────────────────

/// Type the given command-line text through the real key handler.
fn type_command(model: &mut Model, text: &str) {
    for c in text.chars() {
        let msg = handle_key(key(KeyCode::Char(c)), model).expect("key should map to a message");
        update(model, msg);
    }
}

fn command_buffer(model: &Model) -> Option<&str> {
    model
        .rebase_todo
        .as_ref()
        .and_then(|s| s.command_input.as_deref())
}

#[test]
fn test_colon_wq_maps_to_execute_interactive() {
    let (_test_repo, mut model) = model_with_todo_editor();

    type_command(&mut model, ":wq");
    assert_eq!(command_buffer(&model), Some("wq"));

    // Action keys go into the buffer instead of editing entries
    assert_eq!(todo_actions(&model), vec![RebaseAction::Pick; 2]);

    assert_eq!(
        handle_key(key(KeyCode::Enter), &model),
        Some(Message::Rebase(RebaseCommand::ExecuteInteractive))
    );
}

#[test]
fn test_colon_x_maps_to_execute_interactive() {
    let (_test_repo, mut model) = model_with_todo_editor();

    type_command(&mut model, ":x");
    assert_eq!(
        handle_key(key(KeyCode::Enter), &model),
        Some(Message::Rebase(RebaseCommand::ExecuteInteractive))
    );
}

#[test]
fn test_colon_q_maps_to_abort() {
    let (_test_repo, mut model) = model_with_todo_editor();

    type_command(&mut model, ":q");
    assert_eq!(
        handle_key(key(KeyCode::Enter), &model),
        Some(Message::RebaseTodo(RebaseTodoMessage::Abort))
    );
}

#[test]
fn test_unknown_command_shows_toast_and_leaves_command_mode() {
    let (_test_repo, mut model) = model_with_todo_editor();

    type_command(&mut model, ":foo");
    let msg = handle_key(key(KeyCode::Enter), &model).unwrap();
    assert_eq!(msg, Message::RebaseTodo(RebaseTodoMessage::CommandInvalid));

    update(&mut model, msg);
    assert_eq!(command_buffer(&model), None);
    let toast = model.toast.as_ref().expect("Expected a warning toast");
    assert!(toast.message.contains(":foo"));
    assert_eq!(model.view_mode, ViewMode::RebaseTodo);
}

#[test]
fn test_command_mode_escape_cancels() {
    let (_test_repo, mut model) = model_with_todo_editor();

    type_command(&mut model, ":wq");
    let msg = handle_key(key(KeyCode::Esc), &model).unwrap();
    update(&mut model, msg);

    assert_eq!(command_buffer(&model), None);
    assert_eq!(model.view_mode, ViewMode::RebaseTodo);
}

#[test]
fn test_command_mode_backspace_edits_and_exits_when_empty() {
    let (_test_repo, mut model) = model_with_todo_editor();

    type_command(&mut model, ":w");

    let backspace = |model: &mut Model| {
        let msg = handle_key(key(KeyCode::Backspace), model).unwrap();
        update(model, msg);
    };

    backspace(&mut model);
    assert_eq!(command_buffer(&model), Some(""));

    // Backspacing past the ':' leaves command mode, like in vim
    backspace(&mut model);
    assert_eq!(command_buffer(&model), None);

    // Keys act on entries again
    assert_eq!(
        handle_key(key(KeyCode::Char('d')), &model),
        Some(Message::RebaseTodo(RebaseTodoMessage::SetAction(
            RebaseAction::Drop
        )))
    );
}

// ── Interactive rebase — commit preview from the todo editor ─────────────────

#[test]
fn test_rebase_todo_preview_commit_and_return() {
    let (_test_repo, mut model) = model_with_todo_editor();
    let lines_before = model.ui_model.lines.len();

    // Preview the entry under the cursor
    update(&mut model, Message::ShowPreview);
    assert_eq!(model.view_mode, ViewMode::Preview);
    assert!(model.ui_model.lines.iter().any(
        |l| matches!(&l.content, LineContent::PreviewLine { content, .. }
                if content.contains("Commit A"))
    ));

    // Exiting the preview restores the todo editor
    update(&mut model, Message::ExitPreview);
    assert_eq!(model.view_mode, ViewMode::RebaseTodo);
    assert_eq!(model.ui_model.lines.len(), lines_before);
    assert!(model.rebase_todo.is_some());
}

#[test]
fn test_rebase_todo_preview_on_hint_line_is_inert() {
    let (_test_repo, mut model) = model_with_todo_editor();

    model.ui_model.cursor_position = 3;
    update(&mut model, Message::ShowPreview);

    assert_eq!(model.view_mode, ViewMode::RebaseTodo);
}

// ── Interactive rebase — end-to-end execution ─────────────────────────────────

#[test]
fn test_execute_interactive_rebase_drops_commit() {
    let (test_repo, mut model) = model_with_todo_editor();

    // Move cursor to Commit B and drop it
    model.ui_model.cursor_position = 1;
    update(
        &mut model,
        Message::RebaseTodo(RebaseTodoMessage::SetAction(RebaseAction::Drop)),
    );

    let result = update(
        &mut model,
        Message::Rebase(RebaseCommand::ExecuteInteractive),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(model.view_mode, ViewMode::Status);
    assert!(model.rebase_todo.is_none());
    assert!(!rebase_in_progress(test_repo.repo_path()));

    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head_commit.summary().unwrap(), Some("Commit A"));
}

// ── Rebasing section shown in status view when rebase in progress ─────────────

#[test]
fn test_rebasing_section_shown_when_rebase_merge_dir_and_stopped_sha_exist() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Simulate a stopped rebase
    test_repo.with_rebase_in_progress();

    // Refresh to pick up the new state
    update(&mut model, Message::Refresh);

    let has_rebasing_section = model
        .ui_model
        .lines
        .iter()
        .any(|l| l.section == Some(SectionType::Rebasing));
    assert!(
        has_rebasing_section,
        "Expected a Rebasing section in the UI"
    );

    let has_rebasing_entry = model.ui_model.lines.iter().any(|l| {
        matches!(
            &l.content,
            LineContent::RebasingEntry {
                is_current: true,
                ..
            }
        )
    });
    assert!(
        has_rebasing_entry,
        "Expected a current rebasing entry in the UI"
    );
}

// ── Modify commit — popup key and selection ───────────────────────────────────

#[test]
fn test_m_in_rebase_popup_shows_modify_commit_select() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(rebase_popup(false));

    let result = handle_key(key(KeyCode::Char('m')), &model);
    assert_eq!(
        result,
        Some(Message::ShowCommitSelect(CommitSelect::ModifyCommit))
    );
}

#[test]
fn test_m_key_has_no_effect_in_in_progress_popup() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(rebase_popup(true));

    let result = handle_key(key(KeyCode::Char('m')), &model);
    assert_eq!(result, None);
}

#[test]
fn test_modify_commit_on_commit_shows_confirmation() {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("file1.txt", "content1", "First commit")
        .commit_file("file2.txt", "content2", "Second commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let commit_line = find_commit_line(&model).expect("Expected a commit line in the model");
    model.ui_model.cursor_position = commit_line;

    let expected_hash =
        if let LineContent::Commit(info) = &model.ui_model.lines[commit_line].content {
            info.hash.clone()
        } else {
            panic!("Not a commit line");
        };

    let result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::ModifyCommit),
    );

    assert_eq!(result, None);
    let state = expect_confirm_popup(&model);
    assert!(state.message.contains(&expected_hash));
    assert!(matches!(
        &state.on_confirm,
        ConfirmAction::ModifyCommit(hash) if *hash == expected_hash
    ));
}

#[test]
fn test_modify_commit_confirmation_routes_to_rebase_command() {
    let test_repo = TestRepo::new();
    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Confirm(
        magi::model::popup::ConfirmPopupState {
            message: "Modify commit abc1234?".to_string(),
            on_confirm: ConfirmAction::ModifyCommit("abc1234".to_string()),
        },
    ));

    let result = handle_key(key(KeyCode::Char('y')), &model);
    assert_eq!(
        result,
        Some(Message::Rebase(RebaseCommand::ModifyCommit(
            "abc1234".to_string()
        )))
    );
}

#[test]
fn test_modify_commit_not_on_commit_shows_current_log_pick_view() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let non_commit_pos = find_line(&model, |c| {
        !matches!(c, LineContent::Commit(_) | LineContent::LogLine(_))
    })
    .expect("Expected at least one non-commit line");
    model.ui_model.cursor_position = non_commit_pos;

    let result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::ModifyCommit),
    );

    assert_eq!(result, None);
    assert!(
        matches!(
            model.view_mode,
            ViewMode::Log {
                log_type: LogType::Current,
                picking: true,
                ..
            }
        ),
        "Expected current-branch log pick view"
    );
    assert_eq!(model.log_pick_on_select, Some(OnSelect::ModifyCommit));
}

#[test]
fn test_select_confirm_modify_commit_returns_rebase_message() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let mut commits = get_log_entries(&repo, &LogType::Current, true, false).unwrap();
    commits.retain(|e| e.is_commit());

    let mut model = create_model_from_test_repo(&test_repo);

    let expected_hash = commits[0].hash.as_ref().unwrap().clone();

    model.ui_model.lines = commits
        .into_iter()
        .map(|entry| Line {
            content: LineContent::LogLine(entry),
            section: None,
        })
        .collect();
    model.ui_model.cursor_position = 0;
    model.view_mode = ViewMode::Log {
        log_type: LogType::Current,
        picking: true,
        graph: true,
        color: false,
    };
    model.log_pick_on_select = Some(OnSelect::ModifyCommit);

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::Rebase(RebaseCommand::ModifyCommit(expected_hash)))
    );
    assert_eq!(model.view_mode, ViewMode::Status);
    assert!(model.log_pick_on_select.is_none());
}

#[test]
fn test_modify_commit_update_stops_rebase_at_commit() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    let target = test_repo.head_hash();
    test_repo.commit_file("file2.txt", "content2", "Second commit");

    let mut model = create_model_from_test_repo(&test_repo);

    update(
        &mut model,
        Message::Rebase(RebaseCommand::ModifyCommit(target)),
    );

    assert!(rebase_in_progress(test_repo.repo_path()));
    assert!(model.toast.is_some(), "Expected a toast after modify");
}
